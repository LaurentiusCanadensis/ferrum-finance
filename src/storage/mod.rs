//! Storage module for FerrumFinance
//!
//! This module provides comprehensive database storage functionality including:
//! - Connection management and pooling
//! - Repository pattern implementation
//! - Transaction management and rollback support
//! - Comprehensive error handling
//! - Multi-currency data handling
//! - Performance optimizations and indexing

use std::fmt;
use thiserror::Error;

// Public modules
pub mod database;
pub mod repositories;

// Re-export commonly used types
pub use database::{Database, DatabaseManager, DatabaseConfig};
pub use repositories::{
    AccountRepository, TransactionRepository, LoanRepository, ExchangeRateRepository,
    Repository, RepositoryManager
};

/// Comprehensive error types for storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database connection error: {message}")]
    ConnectionError { message: String },

    #[error("Database initialization failed: {message}")]
    InitializationError { message: String },

    #[error("Migration failed: {message}")]
    MigrationError { message: String },

    #[error("Transaction error: {message}")]
    TransactionError { message: String },

    #[error("Query execution failed: {message}")]
    QueryError { message: String },

    #[error("Data not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: String },

    #[error("Data already exists: {entity_type} with id {id}")]
    AlreadyExists { entity_type: String, id: String },

    #[error("Constraint violation: {message}")]
    ConstraintViolation { message: String },

    #[error("Foreign key constraint violation: {table} references {referenced_table}")]
    ForeignKeyViolation { table: String, referenced_table: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    #[error("Data integrity error: {message}")]
    DataIntegrityError { message: String },

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: rust_decimal::Decimal, available: rust_decimal::Decimal },

    #[error("Currency conversion error: {message}")]
    CurrencyConversionError { message: String },

    #[error("Invalid amount: {amount} for currency {currency}")]
    InvalidAmount { amount: rust_decimal::Decimal, currency: String },

    #[error("Account not active: {account_id}")]
    AccountNotActive { account_id: String },

    #[error("Transaction limit exceeded: {limit} for account {account_id}")]
    TransactionLimitExceeded { limit: rust_decimal::Decimal, account_id: String },

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Concurrency error: resource modified by another process")]
    ConcurrencyError,

    #[error("Backup error: {message}")]
    BackupError { message: String },

    #[error("Restore error: {message}")]
    RestoreError { message: String },

    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl StorageError {
    /// Creates a connection error
    pub fn connection_error(message: impl Into<String>) -> Self {
        Self::ConnectionError {
            message: message.into(),
        }
    }

    /// Creates a query error
    pub fn query_error(message: impl Into<String>) -> Self {
        Self::QueryError {
            message: message.into(),
        }
    }

    /// Creates a not found error
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Creates a validation error
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Creates a constraint violation error
    pub fn constraint_violation(message: impl Into<String>) -> Self {
        Self::ConstraintViolation {
            message: message.into(),
        }
    }

    /// Creates a data integrity error
    pub fn data_integrity_error(message: impl Into<String>) -> Self {
        Self::DataIntegrityError {
            message: message.into(),
        }
    }

    /// Checks if this is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Checks if this is a constraint violation
    pub fn is_constraint_violation(&self) -> bool {
        matches!(self, Self::ConstraintViolation { .. } | Self::ForeignKeyViolation { .. })
    }

    /// Checks if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::ValidationError { .. })
    }
}

/// Conversion from rusqlite errors
impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(sqlite_error, message) => {
                match sqlite_error.code {
                    rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY => {
                        StorageError::ForeignKeyViolation {
                            table: "unknown".to_string(),
                            referenced_table: "unknown".to_string(),
                        }
                    }
                    rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE |
                    rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY => {
                        StorageError::ConstraintViolation {
                            message: message.unwrap_or_else(|| "Unique constraint violation".to_string()),
                        }
                    }
                    rusqlite::ffi::SQLITE_CONSTRAINT => {
                        StorageError::ConstraintViolation {
                            message: message.unwrap_or_else(|| "Constraint violation".to_string()),
                        }
                    }
                    _ => StorageError::QueryError {
                        message: format!("SQLite error: {}",
                            message.unwrap_or_else(|| "Unknown database error".to_string())),
                    }
                }
            }
            rusqlite::Error::QueryReturnedNoRows => {
                StorageError::NotFound {
                    entity_type: "Record".to_string(),
                    id: "unknown".to_string(),
                }
            }
            _ => StorageError::QueryError {
                message: format!("Database error: {}", error),
            }
        }
    }
}

/// Conversion from serde_json errors
impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        if error.is_syntax() || error.is_data() {
            StorageError::DeserializationError {
                message: format!("JSON deserialization failed: {}", error),
            }
        } else {
            StorageError::SerializationError {
                message: format!("JSON serialization failed: {}", error),
            }
        }
    }
}

/// Conversion from UUID errors
impl From<uuid::Error> for StorageError {
    fn from(error: uuid::Error) -> Self {
        StorageError::ValidationError {
            field: "id".to_string(),
            message: format!("Invalid UUID: {}", error),
        }
    }
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Pagination parameters for database queries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub offset: u32,
}

impl Pagination {
    /// Creates a new pagination with page and per_page
    pub fn new(page: u32, per_page: u32) -> Self {
        let offset = page.saturating_sub(1) * per_page;
        Self {
            page,
            per_page,
            offset,
        }
    }

    /// Creates pagination for the first page
    pub fn first_page(per_page: u32) -> Self {
        Self::new(1, per_page)
    }

    /// Gets the SQL LIMIT clause
    pub fn limit(&self) -> u32 {
        self.per_page
    }

    /// Gets the SQL OFFSET clause
    pub fn offset(&self) -> u32 {
        self.offset
    }
}

/// Query ordering for database results
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "ASC"),
            SortOrder::Descending => write!(f, "DESC"),
        }
    }
}

/// Query filtering and sorting options
#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub pagination: Option<Pagination>,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
}

impl QueryOptions {
    /// Creates new query options
    pub fn new() -> Self {
        Self {
            pagination: None,
            sort_by: None,
            sort_order: SortOrder::Ascending,
        }
    }

    /// Sets pagination
    pub fn with_pagination(mut self, pagination: Pagination) -> Self {
        self.pagination = Some(pagination);
        self
    }

    /// Sets sorting
    pub fn with_sort(mut self, sort_by: String, sort_order: SortOrder) -> Self {
        self.sort_by = Some(sort_by);
        self.sort_order = sort_order;
        self
    }

    /// Adds ORDER BY clause to SQL if sorting is specified
    pub fn apply_order_by(&self, sql: &mut String) {
        if let Some(ref sort_by) = self.sort_by {
            sql.push_str(&format!(" ORDER BY {} {}", sort_by, self.sort_order));
        }
    }

    /// Adds LIMIT and OFFSET clauses to SQL if pagination is specified
    pub fn apply_pagination(&self, sql: &mut String) {
        if let Some(ref pagination) = self.pagination {
            sql.push_str(&format!(" LIMIT {} OFFSET {}", pagination.limit(), pagination.offset()));
        }
    }

    /// Applies both ordering and pagination to SQL
    pub fn apply_to_sql(&self, sql: &mut String) {
        self.apply_order_by(sql);
        self.apply_pagination(sql);
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Database transaction wrapper providing RAII-style transaction management
pub struct DatabaseTransaction<'conn> {
    pub transaction: rusqlite::Transaction<'conn>,
    committed: bool,
}

impl<'conn> DatabaseTransaction<'conn> {
    /// Creates a new database transaction
    pub fn new(transaction: rusqlite::Transaction<'conn>) -> Self {
        Self {
            transaction,
            committed: false,
        }
    }

    /// Commits the transaction
    pub fn commit(mut self) -> StorageResult<()> {
        self.transaction.commit()?;
        self.committed = true;
        Ok(())
    }

    /// Rolls back the transaction (called automatically on drop if not committed)
    pub fn rollback(mut self) -> StorageResult<()> {
        self.transaction.rollback()?;
        Ok(())
    }

    /// Executes a SQL statement within the transaction
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> StorageResult<usize> {
        Ok(self.transaction.execute(sql, params)?)
    }

    /// Prepares a SQL statement within the transaction
    pub fn prepare(&self, sql: &str) -> StorageResult<rusqlite::Statement> {
        Ok(self.transaction.prepare(sql)?)
    }

    /// Queries a single row within the transaction
    pub fn query_row<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> StorageResult<T>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        Ok(self.transaction.query_row(sql, params, f)?)
    }
}

impl<'conn> Drop for DatabaseTransaction<'conn> {
    fn drop(&mut self) {
        if !self.committed {
            // Automatically rollback if not committed
            let _ = self.transaction.rollback();
        }
    }
}

/// Audit trail entry for tracking data changes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: uuid::Uuid,
    pub table_name: String,
    pub record_id: String,
    pub operation: AuditOperation,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub changed_by: Option<String>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub reason: Option<String>,
}

/// Types of audit operations
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuditOperation {
    Insert,
    Update,
    Delete,
}

impl fmt::Display for AuditOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditOperation::Insert => write!(f, "INSERT"),
            AuditOperation::Update => write!(f, "UPDATE"),
            AuditOperation::Delete => write!(f, "DELETE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(1, 10);
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.offset(), 0);

        let pagination = Pagination::new(3, 20);
        assert_eq!(pagination.offset(), 40);

        let pagination = Pagination::first_page(15);
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.offset(), 0);
    }

    #[test]
    fn test_query_options() {
        let mut sql = "SELECT * FROM accounts".to_string();
        let options = QueryOptions::new()
            .with_sort("name".to_string(), SortOrder::Descending)
            .with_pagination(Pagination::new(2, 10));

        options.apply_to_sql(&mut sql);
        assert!(sql.contains("ORDER BY name DESC"));
        assert!(sql.contains("LIMIT 10 OFFSET 10"));
    }

    #[test]
    fn test_storage_error_constructors() {
        let error = StorageError::not_found("Account", "123");
        assert!(error.is_not_found());

        let error = StorageError::validation_error("name", "required field");
        assert!(error.is_validation_error());

        let error = StorageError::constraint_violation("unique constraint");
        assert!(error.is_constraint_violation());
    }
}