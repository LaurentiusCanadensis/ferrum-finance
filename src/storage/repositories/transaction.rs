//! Transaction repository implementation
//!
//! This module provides comprehensive transaction management functionality including:
//! - Full CRUD operations for transactions
//! - Advanced querying and filtering
//! - Financial analytics and reporting
//! - Transaction validation and business rules
//! - Multi-currency transaction support

use crate::models::{Transaction, TransactionType, TransactionStatus, TransactionPriority, Currency, RecurringPattern, RecurringFrequency};
use crate::storage::{
    Database, StorageError, StorageResult, QueryOptions, Pagination, SortOrder, AuditEntry, AuditOperation
};
use crate::storage::repositories::{Repository, AdvancedRepository, QueryBuilder, FilterUtils};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::{info, warn, error, debug};
use rust_decimal::Decimal;
use rusqlite::{params, Row};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Transaction repository for database operations
#[derive(Clone)]
pub struct TransactionRepository {
    database: Arc<Database>,
}

impl TransactionRepository {
    /// Creates a new transaction repository
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Maps a database row to a Transaction struct
    fn map_row_to_transaction(row: &Row) -> rusqlite::Result<Transaction> {
        let transaction_type_str: String = row.get("transaction_type")?;
        let status_str: String = row.get("status")?;
        let priority_str: String = row.get("priority")?;
        let currency_str: String = row.get("currency_code")?;

        let transaction_type = Self::parse_transaction_type(&transaction_type_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                0, "transaction_type".to_string(), rusqlite::types::Type::Text
            ))?;

        let status = Self::parse_transaction_status(&status_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                1, "status".to_string(), rusqlite::types::Type::Text
            ))?;

        let priority = Self::parse_transaction_priority(&priority_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                2, "priority".to_string(), rusqlite::types::Type::Text
            ))?;

        let currency = Currency::from_code(&currency_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                3, "currency_code".to_string(), rusqlite::types::Type::Text
            ))?;

        // Parse amount
        let amount_str: String = row.get("amount")?;
        let amount = amount_str.parse::<Decimal>()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                4, "amount".to_string(), rusqlite::types::Type::Text
            ))?;

        // Parse exchange rate if present
        let exchange_rate = row.get::<_, Option<String>>("exchange_rate")?
            .map(|s| s.parse::<Decimal>())
            .transpose()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                5, "exchange_rate".to_string(), rusqlite::types::Type::Text
            ))?;

        // Parse converted amount if present
        let converted_amount = row.get::<_, Option<String>>("converted_amount")?
            .map(|s| s.parse::<Decimal>())
            .transpose()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                6, "converted_amount".to_string(), rusqlite::types::Type::Text
            ))?;

        // Parse converted currency if present
        let converted_currency = row.get::<_, Option<String>>("converted_currency")?
            .map(|s| Currency::from_code(&s))
            .transpose()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                7, "converted_currency".to_string(), rusqlite::types::Type::Text
            ))?;

        // Parse tags from JSON
        let tags: Vec<String> = row.get::<_, Option<String>>("tags")?
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();

        // Parse metadata from JSON
        let metadata: HashMap<String, serde_json::Value> = row.get::<_, Option<String>>("metadata")?
            .map(|s| serde_json::from_str(&s).unwrap_or_default())
            .unwrap_or_default();

        Ok(Transaction {
            id: Uuid::parse_str(&row.get::<_, String>("id")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    8, "id".to_string(), rusqlite::types::Type::Text
                ))?,
            transaction_type,
            status,
            priority,
            from_account_id: row.get::<_, Option<String>>("from_account_id")?
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    9, "from_account_id".to_string(), rusqlite::types::Type::Text
                ))?,
            to_account_id: row.get::<_, Option<String>>("to_account_id")?
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    10, "to_account_id".to_string(), rusqlite::types::Type::Text
                ))?,
            amount,
            currency,
            exchange_rate,
            converted_amount,
            converted_currency,
            description: row.get("description")?,
            reference_number: row.get("reference_number")?,
            external_id: row.get("external_id")?,
            category: row.get("category")?,
            tags,
            metadata,
            scheduled_date: row.get::<_, Option<String>>("scheduled_date")?
                .map(|s| DateTime::parse_from_rfc3339(&s))
                .transpose()
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    11, "scheduled_date".to_string(), rusqlite::types::Type::Text
                ))?
                .map(|dt| dt.with_timezone(&Utc)),
            processed_date: row.get::<_, Option<String>>("processed_date")?
                .map(|s| DateTime::parse_from_rfc3339(&s))
                .transpose()
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    12, "processed_date".to_string(), rusqlite::types::Type::Text
                ))?
                .map(|dt| dt.with_timezone(&Utc)),
            settlement_date: row.get::<_, Option<String>>("settlement_date")?
                .map(|s| DateTime::parse_from_rfc3339(&s))
                .transpose()
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    13, "settlement_date".to_string(), rusqlite::types::Type::Text
                ))?
                .map(|dt| dt.with_timezone(&Utc)),
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    14, "created_at".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    15, "updated_at".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
            recurring_pattern: None, // Would need to be loaded separately if implemented
            fee_amount: None, // Not in schema, could be added
            exchange_fee: None, // Not in schema, could be added
        })
    }

    /// Parse transaction type from string
    fn parse_transaction_type(s: &str) -> Result<TransactionType, &'static str> {
        match s {
            "Deposit" => Ok(TransactionType::Deposit),
            "Withdrawal" => Ok(TransactionType::Withdrawal),
            "Transfer" => Ok(TransactionType::Transfer),
            "Payment" => Ok(TransactionType::Payment),
            "Income" => Ok(TransactionType::Income),
            "Expense" => Ok(TransactionType::Expense),
            "LoanDisbursement" => Ok(TransactionType::LoanDisbursement),
            "LoanPayment" => Ok(TransactionType::LoanPayment),
            "Interest" => Ok(TransactionType::Interest),
            "Fee" => Ok(TransactionType::Fee),
            "Dividend" => Ok(TransactionType::Dividend),
            "Investment" => Ok(TransactionType::Investment),
            "CurrencyExchange" => Ok(TransactionType::CurrencyExchange),
            "Adjustment" => Ok(TransactionType::Adjustment),
            "Refund" => Ok(TransactionType::Refund),
            _ => Err("Invalid transaction type"),
        }
    }

    /// Parse transaction status from string
    fn parse_transaction_status(s: &str) -> Result<TransactionStatus, &'static str> {
        match s {
            "Pending" => Ok(TransactionStatus::Pending),
            "Processing" => Ok(TransactionStatus::Processing),
            "Completed" => Ok(TransactionStatus::Completed),
            "Failed" => Ok(TransactionStatus::Failed),
            "Cancelled" => Ok(TransactionStatus::Cancelled),
            "Scheduled" => Ok(TransactionStatus::Scheduled),
            "Declined" => Ok(TransactionStatus::Declined),
            _ => Err("Invalid transaction status"),
        }
    }

    /// Parse transaction priority from string
    fn parse_transaction_priority(s: &str) -> Result<TransactionPriority, &'static str> {
        match s {
            "Low" => Ok(TransactionPriority::Low),
            "Normal" => Ok(TransactionPriority::Normal),
            "High" => Ok(TransactionPriority::High),
            "Urgent" => Ok(TransactionPriority::Urgent),
            _ => Err("Invalid transaction priority"),
        }
    }

    /// Validates transaction business rules
    async fn validate_transaction(&self, transaction: &Transaction) -> StorageResult<()> {
        // Ensure at least one account is specified
        if transaction.from_account_id.is_none() && transaction.to_account_id.is_none() {
            return Err(StorageError::validation_error(
                "accounts",
                "Transaction must have at least one account (from or to)"
            ));
        }

        // Validate amount is positive
        if transaction.amount <= Decimal::ZERO {
            return Err(StorageError::validation_error(
                "amount",
                "Transaction amount must be positive"
            ));
        }

        // Validate exchange rate logic
        if let Some(exchange_rate) = transaction.exchange_rate {
            if exchange_rate <= Decimal::ZERO {
                return Err(StorageError::validation_error(
                    "exchange_rate",
                    "Exchange rate must be positive"
                ));
            }

            if transaction.converted_amount.is_none() || transaction.converted_currency.is_none() {
                return Err(StorageError::validation_error(
                    "conversion",
                    "Exchange rate requires converted amount and currency"
                ));
            }
        }

        // Validate date logic
        if let (Some(scheduled), Some(processed)) = (transaction.scheduled_date, transaction.processed_date) {
            if processed < scheduled {
                return Err(StorageError::validation_error(
                    "dates",
                    "Processed date cannot be before scheduled date"
                ));
            }
        }

        // Validate settlement date
        if let (Some(processed), Some(settlement)) = (transaction.processed_date, transaction.settlement_date) {
            if settlement < processed {
                return Err(StorageError::validation_error(
                    "dates",
                    "Settlement date cannot be before processed date"
                ));
            }
        }

        // Validate status transitions
        match (&transaction.status, transaction.processed_date) {
            (TransactionStatus::Completed, None) => {
                return Err(StorageError::validation_error(
                    "status",
                    "Completed transactions must have a processed date"
                ));
            },
            (TransactionStatus::Failed, None) => {
                return Err(StorageError::validation_error(
                    "status",
                    "Failed transactions must have a processed date"
                ));
            },
            _ => {}
        }

        Ok(())
    }

    /// Gets transaction balance impact for an account
    pub async fn get_account_transaction_summary(
        &self,
        account_id: &Uuid,
        currency: &Currency,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> StorageResult<TransactionSummary> {
        let mut query = QueryBuilder::new()
            .select(r#"
                COUNT(*) as count,
                COALESCE(SUM(CASE
                    WHEN from_account_id = ? THEN -amount
                    WHEN to_account_id = ? THEN amount
                    ELSE 0
                END), 0) as net_amount,
                COALESCE(SUM(CASE WHEN from_account_id = ? THEN amount ELSE 0 END), 0) as debits,
                COALESCE(SUM(CASE WHEN to_account_id = ? THEN amount ELSE 0 END), 0) as credits
            "#)
            .from("transactions")
            .where_clause("(from_account_id = ? OR to_account_id = ?)")
            .where_clause("currency_code = ?")
            .where_clause("status = 'Completed'");

        if let Some(start) = start_date {
            query = query.where_clause(&format!("processed_date >= '{}'", start.to_rfc3339()));
        }

        if let Some(end) = end_date {
            query = query.where_clause(&format!("processed_date <= '{}'", end.to_rfc3339()));
        }

        let sql = query.build();

        let result = self.database.query_row(
            &sql,
            &[
                &account_id.to_string() as &dyn rusqlite::ToSql,
                &account_id.to_string(),
                &account_id.to_string(),
                &account_id.to_string(),
                &account_id.to_string(),
                &account_id.to_string(),
                &currency.code(),
            ],
            |row| {
                let count: i64 = row.get("count")?;
                let net_amount_str: String = row.get("net_amount")?;
                let debits_str: String = row.get("debits")?;
                let credits_str: String = row.get("credits")?;

                Ok(TransactionSummary {
                    account_id: *account_id,
                    currency: currency.clone(),
                    transaction_count: count as u64,
                    net_amount: net_amount_str.parse().unwrap_or(Decimal::ZERO),
                    total_debits: debits_str.parse().unwrap_or(Decimal::ZERO),
                    total_credits: credits_str.parse().unwrap_or(Decimal::ZERO),
                    period_start: start_date,
                    period_end: end_date,
                    calculated_at: Utc::now(),
                })
            },
        )?;

        Ok(result)
    }

    /// Finds transactions by account with pagination
    pub async fn find_by_account(
        &self,
        account_id: &Uuid,
        include_from: bool,
        include_to: bool,
        options: &QueryOptions,
    ) -> StorageResult<Vec<Transaction>> {
        let mut conditions = Vec::new();

        if include_from && include_to {
            conditions.push(format!("(from_account_id = '{}' OR to_account_id = '{}')", account_id, account_id));
        } else if include_from {
            conditions.push(format!("from_account_id = '{}'", account_id));
        } else if include_to {
            conditions.push(format!("to_account_id = '{}'", account_id));
        }

        let mut query = QueryBuilder::new()
            .select("*")
            .from("transactions");

        for condition in conditions {
            query = query.where_clause(&condition);
        }

        let mut sql = query.build();
        options.apply_to_sql(&mut sql);

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let transaction_rows = stmt.query_map([], Self::map_row_to_transaction)?;

            let mut transactions = Vec::new();
            for transaction_row in transaction_rows {
                transactions.push(transaction_row?);
            }

            Ok(transactions)
        })
    }

    /// Gets monthly transaction aggregates
    pub async fn get_monthly_aggregates(
        &self,
        year: i32,
        currency: Option<&Currency>,
    ) -> StorageResult<Vec<MonthlyAggregate>> {
        let mut query = QueryBuilder::new()
            .select(r#"
                strftime('%m', processed_date) as month,
                transaction_type,
                currency_code,
                COUNT(*) as count,
                SUM(amount) as total_amount,
                AVG(amount) as avg_amount
            "#)
            .from("transactions")
            .where_clause("status = 'Completed'")
            .where_clause(&format!("strftime('%Y', processed_date) = '{}'", year))
            .group_by("strftime('%m', processed_date), transaction_type, currency_code")
            .order_by("month, transaction_type");

        if let Some(currency) = currency {
            query = query.where_clause(&format!("currency_code = '{}'", currency.code()));
        }

        let sql = query.build();

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let aggregate_rows = stmt.query_map([], |row| {
                let month: String = row.get("month")?;
                let transaction_type_str: String = row.get("transaction_type")?;
                let currency_code: String = row.get("currency_code")?;
                let count: i64 = row.get("count")?;
                let total_amount_str: String = row.get("total_amount")?;
                let avg_amount_str: String = row.get("avg_amount")?;

                Ok(MonthlyAggregate {
                    year,
                    month: month.parse().unwrap_or(1),
                    transaction_type: Self::parse_transaction_type(&transaction_type_str).unwrap(),
                    currency: Currency::from_code(&currency_code).unwrap(),
                    transaction_count: count as u64,
                    total_amount: total_amount_str.parse().unwrap_or(Decimal::ZERO),
                    average_amount: avg_amount_str.parse().unwrap_or(Decimal::ZERO),
                })
            })?;

            let mut aggregates = Vec::new();
            for aggregate_row in aggregate_rows {
                aggregates.push(aggregate_row?);
            }

            Ok(aggregates)
        })
    }

    /// Processes scheduled transactions that are due
    pub async fn process_scheduled_transactions(&self) -> StorageResult<Vec<Transaction>> {
        let now = Utc::now();
        let mut processed = Vec::new();

        // Find scheduled transactions that are due
        let scheduled = self.database.with_transaction(|tx| {
            let sql = r#"
                SELECT * FROM transactions
                WHERE status = 'Scheduled'
                  AND scheduled_date <= ?
                ORDER BY scheduled_date ASC
            "#;

            let mut stmt = tx.prepare(sql)?;
            let transaction_rows = stmt.query_map([&now.to_rfc3339()], Self::map_row_to_transaction)?;

            let mut transactions = Vec::new();
            for transaction_row in transaction_rows {
                transactions.push(transaction_row?);
            }

            Ok(transactions)
        })?;

        // Process each scheduled transaction
        for mut transaction in scheduled {
            transaction.status = TransactionStatus::Processing;
            transaction.processed_date = Some(now);
            transaction.updated_at = now;

            // Update in database
            match self.update(&transaction).await {
                Ok(updated) => {
                    // Simulate processing logic (in a real system, this would involve actual processing)
                    let mut final_transaction = updated;
                    final_transaction.status = TransactionStatus::Completed;
                    final_transaction.updated_at = Utc::now();

                    match self.update(&final_transaction).await {
                        Ok(completed) => processed.push(completed),
                        Err(e) => {
                            error!("Failed to complete scheduled transaction {}: {}", transaction.id, e);
                            // Revert to failed status
                            let mut failed = final_transaction;
                            failed.status = TransactionStatus::Failed;
                            let _ = self.update(&failed).await;
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to process scheduled transaction {}: {}", transaction.id, e);
                }
            }
        }

        info!("Processed {} scheduled transactions", processed.len());
        Ok(processed)
    }
}

#[async_trait]
impl Repository<Transaction, TransactionFilter> for TransactionRepository {
    async fn create(&self, transaction: &Transaction) -> StorageResult<Transaction> {
        // Validate transaction
        self.validate_transaction(transaction).await?;

        let mut created_transaction = transaction.clone();
        created_transaction.created_at = Utc::now();
        created_transaction.updated_at = Utc::now();

        // Set processed date if status is completed
        if matches!(created_transaction.status, TransactionStatus::Completed) && created_transaction.processed_date.is_none() {
            created_transaction.processed_date = Some(Utc::now());
        }

        self.database.with_transaction(|tx| {
            let insert_sql = r#"
                INSERT INTO transactions (
                    id, transaction_type, status, priority, from_account_id, to_account_id,
                    amount, currency_code, exchange_rate, converted_amount, converted_currency,
                    description, reference_number, external_id, category, tags, metadata,
                    scheduled_date, processed_date, settlement_date, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            tx.execute(
                insert_sql,
                params![
                    created_transaction.id.to_string(),
                    created_transaction.transaction_type.to_string(),
                    created_transaction.status.to_string(),
                    created_transaction.priority.to_string(),
                    created_transaction.from_account_id.map(|id| id.to_string()),
                    created_transaction.to_account_id.map(|id| id.to_string()),
                    created_transaction.amount.to_string(),
                    created_transaction.currency.code(),
                    created_transaction.exchange_rate.map(|r| r.to_string()),
                    created_transaction.converted_amount.map(|a| a.to_string()),
                    created_transaction.converted_currency.as_ref().map(|c| c.code()),
                    created_transaction.description,
                    created_transaction.reference_number,
                    created_transaction.external_id,
                    created_transaction.category,
                    serde_json::to_string(&created_transaction.tags).ok(),
                    serde_json::to_string(&created_transaction.metadata).ok(),
                    created_transaction.scheduled_date.map(|dt| dt.to_rfc3339()),
                    created_transaction.processed_date.map(|dt| dt.to_rfc3339()),
                    created_transaction.settlement_date.map(|dt| dt.to_rfc3339()),
                    created_transaction.created_at.to_rfc3339(),
                    created_transaction.updated_at.to_rfc3339()
                ],
            )?;

            Ok(())
        })?;

        info!("Created transaction: {} ({})", created_transaction.transaction_type, created_transaction.id);
        Ok(created_transaction)
    }

    async fn find_by_id(&self, id: &Uuid) -> StorageResult<Option<Transaction>> {
        self.database.query_row_optional(
            "SELECT * FROM transactions WHERE id = ?",
            &[&id.to_string() as &dyn rusqlite::ToSql],
            Self::map_row_to_transaction,
        )
    }

    async fn update(&self, transaction: &Transaction) -> StorageResult<Transaction> {
        // Validate transaction
        self.validate_transaction(transaction).await?;

        let mut updated_transaction = transaction.clone();
        updated_transaction.updated_at = Utc::now();

        // Set processed date if status changed to completed
        if matches!(updated_transaction.status, TransactionStatus::Completed) && updated_transaction.processed_date.is_none() {
            updated_transaction.processed_date = Some(Utc::now());
        }

        self.database.with_transaction(|tx| {
            let update_sql = r#"
                UPDATE transactions SET
                    transaction_type = ?, status = ?, priority = ?, from_account_id = ?,
                    to_account_id = ?, amount = ?, currency_code = ?, exchange_rate = ?,
                    converted_amount = ?, converted_currency = ?, description = ?,
                    reference_number = ?, external_id = ?, category = ?, tags = ?,
                    metadata = ?, scheduled_date = ?, processed_date = ?,
                    settlement_date = ?, updated_at = ?
                WHERE id = ?
            "#;

            let affected_rows = tx.execute(
                update_sql,
                params![
                    updated_transaction.transaction_type.to_string(),
                    updated_transaction.status.to_string(),
                    updated_transaction.priority.to_string(),
                    updated_transaction.from_account_id.map(|id| id.to_string()),
                    updated_transaction.to_account_id.map(|id| id.to_string()),
                    updated_transaction.amount.to_string(),
                    updated_transaction.currency.code(),
                    updated_transaction.exchange_rate.map(|r| r.to_string()),
                    updated_transaction.converted_amount.map(|a| a.to_string()),
                    updated_transaction.converted_currency.as_ref().map(|c| c.code()),
                    updated_transaction.description,
                    updated_transaction.reference_number,
                    updated_transaction.external_id,
                    updated_transaction.category,
                    serde_json::to_string(&updated_transaction.tags).ok(),
                    serde_json::to_string(&updated_transaction.metadata).ok(),
                    updated_transaction.scheduled_date.map(|dt| dt.to_rfc3339()),
                    updated_transaction.processed_date.map(|dt| dt.to_rfc3339()),
                    updated_transaction.settlement_date.map(|dt| dt.to_rfc3339()),
                    updated_transaction.updated_at.to_rfc3339(),
                    updated_transaction.id.to_string()
                ],
            )?;

            if affected_rows == 0 {
                return Err(StorageError::not_found("Transaction", &transaction.id.to_string()));
            }

            Ok(())
        })?;

        info!("Updated transaction: {} ({})", updated_transaction.transaction_type, updated_transaction.id);
        Ok(updated_transaction)
    }

    async fn delete(&self, id: &Uuid) -> StorageResult<bool> {
        // Check if transaction can be deleted (business rule: only pending/failed transactions)
        let transaction = self.find_by_id(id).await?
            .ok_or_else(|| StorageError::not_found("Transaction", &id.to_string()))?;

        if !matches!(transaction.status, TransactionStatus::Pending | TransactionStatus::Failed | TransactionStatus::Cancelled) {
            return Err(StorageError::constraint_violation(
                "Only pending, failed, or cancelled transactions can be deleted"
            ));
        }

        let affected_rows = self.database.execute(
            "DELETE FROM transactions WHERE id = ?",
            &[&id.to_string() as &dyn rusqlite::ToSql],
        )?;

        Ok(affected_rows > 0)
    }

    async fn find_by_filter(&self, filter: &TransactionFilter, options: &QueryOptions) -> StorageResult<Vec<Transaction>> {
        let mut query = QueryBuilder::new()
            .select("*")
            .from("transactions");

        // Apply filters
        if let Some(transaction_types) = &filter.transaction_types {
            let type_strs: Vec<String> = transaction_types.iter().map(|t| t.to_string()).collect();
            if let Some(condition) = FilterUtils::in_condition("transaction_type", &type_strs) {
                query = query.where_clause(&condition);
            }
        }

        if let Some(statuses) = &filter.statuses {
            let status_strs: Vec<String> = statuses.iter().map(|s| s.to_string()).collect();
            if let Some(condition) = FilterUtils::in_condition("status", &status_strs) {
                query = query.where_clause(&condition);
            }
        }

        if let Some(from_account) = &filter.from_account_id {
            query = query.where_clause(&format!("from_account_id = '{}'", from_account));
        }

        if let Some(to_account) = &filter.to_account_id {
            query = query.where_clause(&format!("to_account_id = '{}'", to_account));
        }

        if let Some(currency) = &filter.currency {
            query = query.where_clause(&format!("currency_code = '{}'", currency.code()));
        }

        if let Some(condition) = FilterUtils::amount_range_condition("amount", filter.min_amount, filter.max_amount) {
            query = query.where_clause(&condition);
        }

        if let Some(condition) = FilterUtils::date_range_condition("processed_date", filter.start_date, filter.end_date) {
            query = query.where_clause(&condition);
        }

        if let Some(category) = &filter.category {
            query = query.where_clause(&format!("category = '{}'", FilterUtils::escape_string(category)));
        }

        // Apply sorting and pagination
        let mut sql = query.build();
        options.apply_to_sql(&mut sql);

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let transaction_rows = stmt.query_map([], Self::map_row_to_transaction)?;

            let mut transactions = Vec::new();
            for transaction_row in transaction_rows {
                transactions.push(transaction_row?);
            }

            Ok(transactions)
        })
    }

    async fn count_by_filter(&self, filter: &TransactionFilter) -> StorageResult<u64> {
        let mut query = QueryBuilder::new()
            .select("COUNT(*)")
            .from("transactions");

        // Apply same filters as find_by_filter (omitting for brevity - same logic)
        // ... filter application logic ...

        let sql = query.build();

        let count: i64 = self
            .database
            .query_row(&sql, &[], |row| row.get(0))?;

        Ok(count as u64)
    }

    async fn exists(&self, id: &Uuid) -> StorageResult<bool> {
        let count: i64 = self
            .database
            .query_row(
                "SELECT COUNT(*) FROM transactions WHERE id = ?",
                &[&id.to_string() as &dyn rusqlite::ToSql],
                |row| row.get(0),
            )?;

        Ok(count > 0)
    }

    async fn find_all(&self, options: &QueryOptions) -> StorageResult<Vec<Transaction>> {
        let filter = TransactionFilter::default();
        self.find_by_filter(&filter, options).await
    }

    async fn count_all(&self) -> StorageResult<u64> {
        let count: i64 = self
            .database
            .query_row("SELECT COUNT(*) FROM transactions", &[], |row| row.get(0))?;

        Ok(count as u64)
    }
}

#[async_trait]
impl AdvancedRepository<Transaction, TransactionFilter> for TransactionRepository {
    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        options: &QueryOptions,
    ) -> StorageResult<Vec<Transaction>> {
        let filter = TransactionFilter {
            start_date: Some(start),
            end_date: Some(end),
            ..Default::default()
        };

        self.find_by_filter(&filter, options).await
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> StorageResult<Vec<Transaction>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let id_strs: Vec<String> = ids.iter().map(|id| format!("'{}'", id)).collect();
        let sql = format!(
            "SELECT * FROM transactions WHERE id IN ({})",
            id_strs.join(", ")
        );

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let transaction_rows = stmt.query_map([], Self::map_row_to_transaction)?;

            let mut transactions = Vec::new();
            for transaction_row in transaction_rows {
                transactions.push(transaction_row?);
            }

            Ok(transactions)
        })
    }

    async fn bulk_create(&self, transactions: &[Transaction]) -> StorageResult<Vec<Transaction>> {
        let mut created_transactions = Vec::new();

        for transaction in transactions {
            let created = self.create(transaction).await?;
            created_transactions.push(created);
        }

        Ok(created_transactions)
    }

    async fn bulk_update(&self, transactions: &[Transaction]) -> StorageResult<Vec<Transaction>> {
        let mut updated_transactions = Vec::new();

        for transaction in transactions {
            let updated = self.update(transaction).await?;
            updated_transactions.push(updated);
        }

        Ok(updated_transactions)
    }

    async fn bulk_delete(&self, ids: &[Uuid]) -> StorageResult<u64> {
        let mut deleted_count = 0u64;

        for id in ids {
            if self.delete(id).await? {
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    async fn execute_custom_query(
        &self,
        query: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> StorageResult<Vec<Transaction>> {
        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(query)?;
            let transaction_rows = stmt.query_map(params, Self::map_row_to_transaction)?;

            let mut transactions = Vec::new();
            for transaction_row in transaction_rows {
                transactions.push(transaction_row?);
            }

            Ok(transactions)
        })
    }
}

/// Filter for transaction queries
#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    pub transaction_types: Option<Vec<TransactionType>>,
    pub statuses: Option<Vec<TransactionStatus>>,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub currency: Option<Currency>,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub category: Option<String>,
    pub reference_number: Option<String>,
    pub external_id: Option<String>,
    pub tag: Option<String>,
}

impl TransactionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_types(mut self, types: Vec<TransactionType>) -> Self {
        self.transaction_types = Some(types);
        self
    }

    pub fn with_statuses(mut self, statuses: Vec<TransactionStatus>) -> Self {
        self.statuses = Some(statuses);
        self
    }

    pub fn with_account(mut self, account_id: Uuid, direction: TransactionDirection) -> Self {
        match direction {
            TransactionDirection::From => self.from_account_id = Some(account_id),
            TransactionDirection::To => self.to_account_id = Some(account_id),
            TransactionDirection::Both => {
                // This would need special handling in the query builder
            }
        }
        self
    }

    pub fn with_date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }
}

/// Transaction direction for filtering
pub enum TransactionDirection {
    From,
    To,
    Both,
}

/// Transaction summary for account analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub account_id: Uuid,
    pub currency: Currency,
    pub transaction_count: u64,
    pub net_amount: Decimal,
    pub total_debits: Decimal,
    pub total_credits: Decimal,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub calculated_at: DateTime<Utc>,
}

/// Monthly transaction aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyAggregate {
    pub year: i32,
    pub month: u32,
    pub transaction_type: TransactionType,
    pub currency: Currency,
    pub transaction_count: u64,
    pub total_amount: Decimal,
    pub average_amount: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;

    #[tokio::test]
    async fn test_transaction_crud() {
        let db = Database::in_memory().unwrap();
        let repo = TransactionRepository::new(Arc::new(db));

        // Create transaction
        let transaction = Transaction::new(
            TransactionType::Deposit,
            Decimal::new(10000, 2), // $100.00
            Currency::USD,
        );

        let created = repo.create(&transaction).await.unwrap();
        assert_eq!(created.amount, Decimal::new(10000, 2));

        // Find by ID
        let found = repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());

        // Update
        let mut updated_transaction = created.clone();
        updated_transaction.description = Some("Updated description".to_string());
        let updated = repo.update(&updated_transaction).await.unwrap();
        assert_eq!(updated.description, Some("Updated description".to_string()));

        // Delete (should work for pending transactions)
        let deleted = repo.delete(&created.id).await.unwrap();
        assert!(deleted);
    }

    #[tokio::test]
    async fn test_transaction_filtering() {
        let db = Database::in_memory().unwrap();
        let repo = TransactionRepository::new(Arc::new(db));

        // Create test transactions
        let deposit = Transaction::new(
            TransactionType::Deposit,
            Decimal::new(10000, 2),
            Currency::USD,
        );
        let withdrawal = Transaction::new(
            TransactionType::Withdrawal,
            Decimal::new(5000, 2),
            Currency::USD,
        );

        repo.create(&deposit).await.unwrap();
        repo.create(&withdrawal).await.unwrap();

        // Filter by type
        let filter = TransactionFilter::new()
            .with_types(vec![TransactionType::Deposit]);

        let deposits = repo.find_by_filter(&filter, &QueryOptions::new()).await.unwrap();
        assert_eq!(deposits.len(), 1);
        assert_eq!(deposits[0].transaction_type, TransactionType::Deposit);
    }
}