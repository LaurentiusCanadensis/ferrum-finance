//! Database module for FerrumFinance
//!
//! This module provides comprehensive database management functionality including:
//! - SQLite connection management and pooling
//! - Schema creation and migration handling
//! - Transaction management with rollback support
//! - Database initialization and configuration
//! - Performance optimizations and indexing

use crate::storage::{StorageError, StorageResult, DatabaseTransaction, AuditEntry, AuditOperation};
use chrono::{DateTime, Utc};
use log::{info, warn, error, debug};
use rusqlite::{Connection, Transaction, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Database configuration settings
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_path: PathBuf,
    pub enable_wal_mode: bool,
    pub enable_foreign_keys: bool,
    pub cache_size_kb: i32,
    pub busy_timeout_ms: u32,
    pub enable_audit_trail: bool,
    pub backup_retention_days: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("ferrum_finance.db"),
            enable_wal_mode: true,
            enable_foreign_keys: true,
            cache_size_kb: 10240, // 10MB cache
            busy_timeout_ms: 30000, // 30 seconds
            enable_audit_trail: true,
            backup_retention_days: 30,
        }
    }
}

impl DatabaseConfig {
    /// Creates a new database configuration
    pub fn new(database_path: impl AsRef<Path>) -> Self {
        Self {
            database_path: database_path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Sets WAL mode enabled/disabled
    pub fn with_wal_mode(mut self, enabled: bool) -> Self {
        self.enable_wal_mode = enabled;
        self
    }

    /// Sets foreign keys enabled/disabled
    pub fn with_foreign_keys(mut self, enabled: bool) -> Self {
        self.enable_foreign_keys = enabled;
        self
    }

    /// Sets cache size in KB
    pub fn with_cache_size(mut self, size_kb: i32) -> Self {
        self.cache_size_kb = size_kb;
        self
    }

    /// Sets busy timeout in milliseconds
    pub fn with_busy_timeout(mut self, timeout_ms: u32) -> Self {
        self.busy_timeout_ms = timeout_ms;
        self
    }

    /// Sets audit trail enabled/disabled
    pub fn with_audit_trail(mut self, enabled: bool) -> Self {
        self.enable_audit_trail = enabled;
        self
    }
}

/// Database connection wrapper with enhanced functionality
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
}

impl Database {
    /// Opens a database connection with the given configuration
    pub fn open(config: DatabaseConfig) -> StorageResult<Self> {
        info!("Opening database at: {:?}", config.database_path);

        // Ensure parent directory exists
        if let Some(parent) = config.database_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::connection_error(format!("Failed to create database directory: {}", e))
            })?;
        }

        let connection = Connection::open(&config.database_path).map_err(|e| {
            StorageError::connection_error(format!("Failed to open database: {}", e))
        })?;

        let db = Self {
            connection: Arc::new(Mutex::new(connection)),
            config,
        };

        // Configure the database
        db.configure()?;

        // Initialize schema
        db.initialize_schema()?;

        info!("Database opened and initialized successfully");
        Ok(db)
    }

    /// Creates an in-memory database for testing
    pub fn in_memory() -> StorageResult<Self> {
        let config = DatabaseConfig {
            database_path: PathBuf::from(":memory:"),
            ..Default::default()
        };

        Self::open(config)
    }

    /// Configures the database connection with optimizations
    fn configure(&self) -> StorageResult<()> {
        let conn = self.connection.lock().unwrap();

        // Enable foreign key constraints
        if self.config.enable_foreign_keys {
            conn.execute("PRAGMA foreign_keys = ON", [])?;
            debug!("Foreign key constraints enabled");
        }

        // Set WAL mode for better concurrency
        if self.config.enable_wal_mode {
            conn.execute("PRAGMA journal_mode = WAL", [])?;
            debug!("WAL mode enabled");
        }

        // Set cache size for better performance
        conn.execute(
            &format!("PRAGMA cache_size = -{}", self.config.cache_size_kb),
            [],
        )?;

        // Set busy timeout
        conn.busy_timeout(std::time::Duration::from_millis(
            self.config.busy_timeout_ms as u64,
        ))?;

        // Enable query optimization
        conn.execute("PRAGMA optimize", [])?;

        info!("Database configured successfully");
        Ok(())
    }

    /// Initializes the database schema
    fn initialize_schema(&self) -> StorageResult<()> {
        info!("Initializing database schema");

        let conn = self.connection.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        // Create schema version table
        tx.execute(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                description TEXT NOT NULL
            )
            "#,
            [],
        )?;

        // Check current schema version
        let current_version: i32 = tx
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        debug!("Current schema version: {}", current_version);

        // Apply migrations
        self.apply_migrations(&tx, current_version)?;

        tx.commit()?;
        info!("Schema initialization completed");

        Ok(())
    }

    /// Applies database migrations
    fn apply_migrations(&self, tx: &Transaction, current_version: i32) -> StorageResult<()> {
        let migrations = self.get_migrations();

        for (version, description, sql) in migrations.iter() {
            if *version > current_version {
                info!("Applying migration {}: {}", version, description);

                // Execute migration SQL
                tx.execute_batch(sql)?;

                // Record migration
                tx.execute(
                    "INSERT INTO schema_migrations (version, description) VALUES (?, ?)",
                    [version as &dyn rusqlite::ToSql, description as &dyn rusqlite::ToSql],
                )?;

                info!("Migration {} applied successfully", version);
            }
        }

        Ok(())
    }

    /// Returns all database migrations
    fn get_migrations(&self) -> Vec<(i32, &'static str, &'static str)> {
        vec![
            (1, "Create core financial tables", include_str!("../sql/001_create_core_tables.sql")),
            (2, "Create indexes for performance", include_str!("../sql/002_create_indexes.sql")),
            (3, "Create audit trail tables", include_str!("../sql/003_create_audit_tables.sql")),
            (4, "Create views and functions", include_str!("../sql/004_create_views.sql")),
        ]
    }

    /// Begins a new database transaction
    pub fn begin_transaction(&self) -> StorageResult<DatabaseTransaction> {
        let conn = self.connection.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        Ok(DatabaseTransaction::new(tx))
    }

    /// Executes a closure within a transaction with automatic rollback on error
    pub fn with_transaction<F, R>(&self, f: F) -> StorageResult<R>
    where
        F: FnOnce(&DatabaseTransaction) -> StorageResult<R>,
    {
        let tx = self.begin_transaction()?;
        match f(&tx) {
            Ok(result) => {
                tx.commit()?;
                Ok(result)
            }
            Err(error) => {
                tx.rollback()?;
                Err(error)
            }
        }
    }

    /// Executes a SQL statement
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> StorageResult<usize> {
        let conn = self.connection.lock().unwrap();
        Ok(conn.execute(sql, params)?)
    }

    /// Queries a single row
    pub fn query_row<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        f: F,
    ) -> StorageResult<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let conn = self.connection.lock().unwrap();
        Ok(conn.query_row(sql, params, f)?)
    }

    /// Queries an optional single row
    pub fn query_row_optional<T, F>(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
        f: F,
    ) -> StorageResult<Option<T>>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let conn = self.connection.lock().unwrap();
        Ok(conn.query_row(sql, params, f).optional()?)
    }

    /// Prepares a SQL statement
    pub fn prepare(&self, sql: &str) -> StorageResult<rusqlite::Statement> {
        let conn = self.connection.lock().unwrap();
        Ok(conn.prepare(sql)?)
    }

    /// Records an audit entry if audit trail is enabled
    pub fn record_audit_entry(&self, entry: &AuditEntry) -> StorageResult<()> {
        if !self.config.enable_audit_trail {
            return Ok(());
        }

        self.execute(
            r#"
            INSERT INTO audit_trail (
                id, table_name, record_id, operation, old_values, new_values,
                changed_by, changed_at, reason
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            &[
                &entry.id.to_string() as &dyn rusqlite::ToSql,
                &entry.table_name,
                &entry.record_id,
                &entry.operation.to_string(),
                &entry.old_values.as_ref().map(|v| serde_json::to_string(v).unwrap()),
                &entry.new_values.as_ref().map(|v| serde_json::to_string(v).unwrap()),
                &entry.changed_by,
                &entry.changed_at.to_rfc3339(),
                &entry.reason,
            ],
        )?;

        Ok(())
    }

    /// Creates a backup of the database
    pub fn create_backup(&self, backup_path: impl AsRef<Path>) -> StorageResult<()> {
        let conn = self.connection.lock().unwrap();

        // Use SQLite backup API
        let backup_conn = Connection::open(&backup_path)?;
        let backup = rusqlite::backup::Backup::new(&*conn, &backup_conn)?;
        backup.run_to_completion(5, std::time::Duration::from_millis(250), None)?;

        info!("Database backup created at: {:?}", backup_path.as_ref());
        Ok(())
    }

    /// Restores database from a backup
    pub fn restore_from_backup(&self, backup_path: impl AsRef<Path>) -> StorageResult<()> {
        if !backup_path.as_ref().exists() {
            return Err(StorageError::RestoreError {
                message: format!("Backup file not found: {:?}", backup_path.as_ref()),
            });
        }

        let conn = self.connection.lock().unwrap();
        let backup_conn = Connection::open(&backup_path)?;

        let backup = rusqlite::backup::Backup::new(&backup_conn, &*conn)?;
        backup.run_to_completion(5, std::time::Duration::from_millis(250), None)?;

        info!("Database restored from backup: {:?}", backup_path.as_ref());
        Ok(())
    }

    /// Analyzes the database and updates statistics
    pub fn analyze(&self) -> StorageResult<()> {
        self.execute("ANALYZE", &[])?;
        info!("Database analysis completed");
        Ok(())
    }

    /// Vacuums the database to reclaim space
    pub fn vacuum(&self) -> StorageResult<()> {
        self.execute("VACUUM", &[])?;
        info!("Database vacuum completed");
        Ok(())
    }

    /// Gets database statistics
    pub fn get_statistics(&self) -> StorageResult<DatabaseStatistics> {
        let page_count: i64 = self.query_row("PRAGMA page_count", &[], |row| row.get(0))?;
        let page_size: i64 = self.query_row("PRAGMA page_size", &[], |row| row.get(0))?;
        let freelist_count: i64 = self.query_row("PRAGMA freelist_count", &[], |row| row.get(0))?;

        let database_size = page_count * page_size;
        let free_space = freelist_count * page_size;
        let used_space = database_size - free_space;

        // Get table information
        let mut tables = HashMap::new();
        {
            let conn = self.connection.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
            )?;

            let table_names: Result<Vec<String>, rusqlite::Error> = stmt
                .query_map([], |row| row.get(0))?
                .collect();

            for table_name in table_names? {
                let count: i64 = conn.query_row(
                    &format!("SELECT COUNT(*) FROM {}", table_name),
                    [],
                    |row| row.get(0),
                )?;
                tables.insert(table_name, count as u64);
            }
        }

        Ok(DatabaseStatistics {
            database_size: database_size as u64,
            used_space: used_space as u64,
            free_space: free_space as u64,
            page_count: page_count as u64,
            page_size: page_size as u32,
            table_counts: tables,
        })
    }

    /// Closes the database connection
    pub fn close(self) -> StorageResult<()> {
        // The connection will be closed when Arc is dropped
        info!("Database connection closed");
        Ok(())
    }
}

/// Database statistics information
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub database_size: u64,
    pub used_space: u64,
    pub free_space: u64,
    pub page_count: u64,
    pub page_size: u32,
    pub table_counts: HashMap<String, u64>,
}

/// Database manager for handling multiple database operations
pub struct DatabaseManager {
    database: Arc<Database>,
}

impl DatabaseManager {
    /// Creates a new database manager
    pub fn new(database: Database) -> Self {
        Self {
            database: Arc::new(database),
        }
    }

    /// Gets a reference to the database
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Performs database maintenance tasks
    pub fn maintenance(&self) -> StorageResult<()> {
        info!("Starting database maintenance");

        // Analyze database
        self.database.analyze()?;

        // Clean up audit trail if enabled
        if self.database.config.enable_audit_trail {
            self.cleanup_audit_trail()?;
        }

        info!("Database maintenance completed");
        Ok(())
    }

    /// Cleans up old audit trail entries
    fn cleanup_audit_trail(&self) -> StorageResult<()> {
        let retention_days = self.database.config.backup_retention_days;
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);

        let deleted = self.database.execute(
            "DELETE FROM audit_trail WHERE changed_at < ?",
            &[&cutoff_date.to_rfc3339() as &dyn rusqlite::ToSql],
        )?;

        if deleted > 0 {
            info!("Cleaned up {} old audit trail entries", deleted);
        }

        Ok(())
    }

    /// Creates automated backups
    pub fn create_automated_backup(&self) -> StorageResult<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("ferrum_finance_backup_{}.db", timestamp);

        let backup_path = self.database.config.database_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("backups")
            .join(backup_filename);

        // Ensure backup directory exists
        if let Some(parent) = backup_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::BackupError {
                    message: format!("Failed to create backup directory: {}", e),
                }
            })?;
        }

        self.database.create_backup(&backup_path)?;
        Ok(())
    }
}

// SQL migration files would be created in a separate sql directory
// For now, we'll include the SQL inline as constants

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_creation() {
        let db = Database::in_memory().unwrap();
        let stats = db.get_statistics().unwrap();
        assert!(stats.database_size > 0);
    }

    #[test]
    fn test_transaction_rollback() {
        let db = Database::in_memory().unwrap();

        let result = db.with_transaction(|_tx| {
            // Simulate an error
            Err(StorageError::validation_error("test", "test error"))
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::new("test.db")
            .with_wal_mode(false)
            .with_cache_size(2048);

        assert!(!config.enable_wal_mode);
        assert_eq!(config.cache_size_kb, 2048);
    }

    #[test]
    fn test_backup_and_restore() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let backup_path = temp_dir.path().join("backup.db");

        let config = DatabaseConfig::new(&db_path);
        let db = Database::open(config).unwrap();

        // Create backup
        db.create_backup(&backup_path).unwrap();
        assert!(backup_path.exists());

        // Restore from backup
        db.restore_from_backup(&backup_path).unwrap();
    }
}