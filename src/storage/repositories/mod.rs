//! Repository module for FerrumFinance storage layer
//!
//! This module implements the Repository pattern for data access, providing:
//! - Abstract repository traits for common operations
//! - Concrete repository implementations for each entity
//! - Advanced querying and filtering capabilities
//! - Transaction safety and error handling
//! - Multi-currency support and business logic

use crate::storage::{StorageResult, QueryOptions, Pagination, SortOrder};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

// Repository implementations
pub mod account;
pub mod transaction;
pub mod loan;
pub mod exchange_rate;

// Re-export repository implementations
pub use account::{AccountRepository, AccountFilter, AccountSummary};
pub use transaction::{TransactionRepository, TransactionFilter, TransactionSummary};
pub use loan::{LoanRepository, LoanFilter, LoanSummary};
pub use exchange_rate::{ExchangeRateRepository, ExchangeRateFilter};

/// Base repository trait defining common CRUD operations
#[async_trait]
pub trait Repository<T, F>
where
    T: Clone + Send + Sync,
    F: Clone + Send + Sync,
{
    /// Creates a new entity
    async fn create(&self, entity: &T) -> StorageResult<T>;

    /// Finds an entity by its ID
    async fn find_by_id(&self, id: &Uuid) -> StorageResult<Option<T>>;

    /// Updates an existing entity
    async fn update(&self, entity: &T) -> StorageResult<T>;

    /// Deletes an entity by its ID
    async fn delete(&self, id: &Uuid) -> StorageResult<bool>;

    /// Finds entities matching the given filter
    async fn find_by_filter(&self, filter: &F, options: &QueryOptions) -> StorageResult<Vec<T>>;

    /// Counts entities matching the given filter
    async fn count_by_filter(&self, filter: &F) -> StorageResult<u64>;

    /// Checks if an entity exists by ID
    async fn exists(&self, id: &Uuid) -> StorageResult<bool>;

    /// Finds all entities with pagination
    async fn find_all(&self, options: &QueryOptions) -> StorageResult<Vec<T>>;

    /// Gets the total count of entities
    async fn count_all(&self) -> StorageResult<u64>;
}

/// Advanced repository trait for entities requiring complex queries
#[async_trait]
pub trait AdvancedRepository<T, F>: Repository<T, F>
where
    T: Clone + Send + Sync,
    F: Clone + Send + Sync,
{
    /// Finds entities within a date range
    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        options: &QueryOptions,
    ) -> StorageResult<Vec<T>>;

    /// Finds entities by multiple IDs
    async fn find_by_ids(&self, ids: &[Uuid]) -> StorageResult<Vec<T>>;

    /// Performs a bulk insert operation
    async fn bulk_create(&self, entities: &[T]) -> StorageResult<Vec<T>>;

    /// Performs a bulk update operation
    async fn bulk_update(&self, entities: &[T]) -> StorageResult<Vec<T>>;

    /// Performs a bulk delete operation
    async fn bulk_delete(&self, ids: &[Uuid]) -> StorageResult<u64>;

    /// Executes a custom query with parameters
    async fn execute_custom_query(
        &self,
        query: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> StorageResult<Vec<T>>;
}

/// Repository manager for coordinating multiple repositories
pub struct RepositoryManager {
    pub accounts: AccountRepository,
    pub transactions: TransactionRepository,
    pub loans: LoanRepository,
    pub exchange_rates: ExchangeRateRepository,
}

impl RepositoryManager {
    /// Creates a new repository manager
    pub fn new(
        accounts: AccountRepository,
        transactions: TransactionRepository,
        loans: LoanRepository,
        exchange_rates: ExchangeRateRepository,
    ) -> Self {
        Self {
            accounts,
            transactions,
            loans,
            exchange_rates,
        }
    }

    /// Validates that all required entities exist for a transaction
    pub async fn validate_transaction_entities(
        &self,
        from_account_id: Option<&Uuid>,
        to_account_id: Option<&Uuid>,
        currency_code: &str,
    ) -> StorageResult<()> {
        // Validate from account
        if let Some(from_id) = from_account_id {
            if !self.accounts.exists(from_id).await? {
                return Err(crate::storage::StorageError::not_found("Account", &from_id.to_string()));
            }
        }

        // Validate to account
        if let Some(to_id) = to_account_id {
            if !self.accounts.exists(to_id).await? {
                return Err(crate::storage::StorageError::not_found("Account", &to_id.to_string()));
            }
        }

        // Validate currency exists (would need currency repository)
        // This is a placeholder - in a real implementation, you'd check if currency exists

        Ok(())
    }

    /// Gets a comprehensive financial summary
    pub async fn get_financial_summary(
        &self,
        currency_code: &str,
    ) -> StorageResult<FinancialSummary> {
        // Get account summaries
        let account_filter = AccountFilter::default();
        let accounts = self.accounts.find_by_filter(&account_filter, &QueryOptions::default()).await?;

        // Get transaction summaries for the last 30 days
        let end_date = Utc::now();
        let start_date = end_date - chrono::Duration::days(30);
        let transactions = self.transactions.find_by_date_range(start_date, end_date, &QueryOptions::default()).await?;

        // Get loan summaries
        let loan_filter = LoanFilter::default();
        let loans = self.loans.find_by_filter(&loan_filter, &QueryOptions::default()).await?;

        // Calculate totals (simplified - would need actual balance calculations)
        let total_accounts = accounts.len() as u64;
        let total_transactions = transactions.len() as u64;
        let total_loans = loans.len() as u64;

        Ok(FinancialSummary {
            currency_code: currency_code.to_string(),
            total_accounts,
            total_transactions,
            total_loans,
            summary_date: Utc::now(),
            // Add more detailed calculations here
        })
    }

    /// Performs database health checks across all repositories
    pub async fn perform_health_check(&self) -> StorageResult<HealthCheckResult> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check account integrity
        let account_count = self.accounts.count_all().await?;
        if account_count == 0 {
            warnings.push("No accounts found in the system".to_string());
        }

        // Check for orphaned transactions
        // This would require a more complex query to find transactions with invalid account references

        // Check for overdue loans
        let loan_filter = LoanFilter {
            status: Some(vec!["Active".to_string()]),
            ..Default::default()
        };
        let active_loans = self.loans.find_by_filter(&loan_filter, &QueryOptions::default()).await?;
        // Check for overdue payments (would need additional logic)

        Ok(HealthCheckResult {
            status: if issues.is_empty() { "Healthy".to_string() } else { "Issues Found".to_string() },
            issues,
            warnings,
            check_time: Utc::now(),
        })
    }
}

/// Financial summary data structure
#[derive(Debug, Clone)]
pub struct FinancialSummary {
    pub currency_code: String,
    pub total_accounts: u64,
    pub total_transactions: u64,
    pub total_loans: u64,
    pub summary_date: DateTime<Utc>,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: String,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub check_time: DateTime<Utc>,
}

/// Common query builder for constructing dynamic SQL queries
pub struct QueryBuilder {
    select_clause: String,
    from_clause: String,
    where_clauses: Vec<String>,
    join_clauses: Vec<String>,
    group_by_clause: Option<String>,
    having_clause: Option<String>,
    order_by_clause: Option<String>,
    limit_clause: Option<String>,
}

impl QueryBuilder {
    /// Creates a new query builder
    pub fn new() -> Self {
        Self {
            select_clause: "*".to_string(),
            from_clause: String::new(),
            where_clauses: Vec::new(),
            join_clauses: Vec::new(),
            group_by_clause: None,
            having_clause: None,
            order_by_clause: None,
            limit_clause: None,
        }
    }

    /// Sets the SELECT clause
    pub fn select(mut self, fields: &str) -> Self {
        self.select_clause = fields.to_string();
        self
    }

    /// Sets the FROM clause
    pub fn from(mut self, table: &str) -> Self {
        self.from_clause = table.to_string();
        self
    }

    /// Adds a WHERE clause
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    /// Adds a JOIN clause
    pub fn join(mut self, join_clause: &str) -> Self {
        self.join_clauses.push(join_clause.to_string());
        self
    }

    /// Sets the GROUP BY clause
    pub fn group_by(mut self, fields: &str) -> Self {
        self.group_by_clause = Some(fields.to_string());
        self
    }

    /// Sets the HAVING clause
    pub fn having(mut self, condition: &str) -> Self {
        self.having_clause = Some(condition.to_string());
        self
    }

    /// Sets the ORDER BY clause
    pub fn order_by(mut self, fields: &str) -> Self {
        self.order_by_clause = Some(fields.to_string());
        self
    }

    /// Sets the LIMIT clause
    pub fn limit(mut self, limit: u32, offset: Option<u32>) -> Self {
        if let Some(offset) = offset {
            self.limit_clause = Some(format!("LIMIT {} OFFSET {}", limit, offset));
        } else {
            self.limit_clause = Some(format!("LIMIT {}", limit));
        }
        self
    }

    /// Builds the final SQL query
    pub fn build(self) -> String {
        let mut query = format!("SELECT {} FROM {}", self.select_clause, self.from_clause);

        // Add JOINs
        for join in &self.join_clauses {
            query.push_str(&format!(" {}", join));
        }

        // Add WHERE clauses
        if !self.where_clauses.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_clauses.join(" AND ")));
        }

        // Add GROUP BY
        if let Some(group_by) = &self.group_by_clause {
            query.push_str(&format!(" GROUP BY {}", group_by));
        }

        // Add HAVING
        if let Some(having) = &self.having_clause {
            query.push_str(&format!(" HAVING {}", having));
        }

        // Add ORDER BY
        if let Some(order_by) = &self.order_by_clause {
            query.push_str(&format!(" ORDER BY {}", order_by));
        }

        // Add LIMIT
        if let Some(limit) = &self.limit_clause {
            query.push_str(&format!(" {}", limit));
        }

        query
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Common filter utilities
pub struct FilterUtils;

impl FilterUtils {
    /// Converts a date range to SQL condition
    pub fn date_range_condition(
        field: &str,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Option<String> {
        match (start, end) {
            (Some(start), Some(end)) => {
                Some(format!("{} BETWEEN '{}' AND '{}'", field, start.to_rfc3339(), end.to_rfc3339()))
            }
            (Some(start), None) => {
                Some(format!("{} >= '{}'", field, start.to_rfc3339()))
            }
            (None, Some(end)) => {
                Some(format!("{} <= '{}'", field, end.to_rfc3339()))
            }
            (None, None) => None,
        }
    }

    /// Converts a list of values to SQL IN condition
    pub fn in_condition(field: &str, values: &[String]) -> Option<String> {
        if values.is_empty() {
            None
        } else {
            let quoted_values: Vec<String> = values.iter().map(|v| format!("'{}'", v)).collect();
            Some(format!("{} IN ({})", field, quoted_values.join(", ")))
        }
    }

    /// Converts an amount range to SQL condition
    pub fn amount_range_condition(
        field: &str,
        min_amount: Option<Decimal>,
        max_amount: Option<Decimal>,
    ) -> Option<String> {
        match (min_amount, max_amount) {
            (Some(min), Some(max)) => {
                Some(format!("{} BETWEEN {} AND {}", field, min, max))
            }
            (Some(min), None) => {
                Some(format!("{} >= {}", field, min))
            }
            (None, Some(max)) => {
                Some(format!("{} <= {}", field, max))
            }
            (None, None) => None,
        }
    }

    /// Escapes SQL strings to prevent injection
    pub fn escape_string(input: &str) -> String {
        input.replace("'", "''")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new()
            .select("id, name, balance")
            .from("accounts")
            .where_clause("status = 'Active'")
            .where_clause("balance > 0")
            .order_by("name ASC")
            .limit(10, Some(0))
            .build();

        assert!(query.contains("SELECT id, name, balance"));
        assert!(query.contains("FROM accounts"));
        assert!(query.contains("WHERE status = 'Active' AND balance > 0"));
        assert!(query.contains("ORDER BY name ASC"));
        assert!(query.contains("LIMIT 10 OFFSET 0"));
    }

    #[test]
    fn test_filter_utils_date_range() {
        let start = Utc::now();
        let end = start + chrono::Duration::hours(1);

        let condition = FilterUtils::date_range_condition("created_at", Some(start), Some(end));
        assert!(condition.is_some());
        assert!(condition.unwrap().contains("BETWEEN"));
    }

    #[test]
    fn test_filter_utils_in_condition() {
        let values = vec!["Active".to_string(), "Suspended".to_string()];
        let condition = FilterUtils::in_condition("status", &values);

        assert!(condition.is_some());
        assert!(condition.unwrap().contains("status IN ('Active', 'Suspended')"));
    }

    #[test]
    fn test_filter_utils_amount_range() {
        let min = Decimal::new(100, 2); // 1.00
        let max = Decimal::new(10000, 2); // 100.00

        let condition = FilterUtils::amount_range_condition("amount", Some(min), Some(max));
        assert!(condition.is_some());
        assert!(condition.unwrap().contains("BETWEEN"));
    }
}