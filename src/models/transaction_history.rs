//! Transaction history module for FerrumFinance
//!
//! This module provides historical transaction management including audit trails,
//! transaction filtering, search capabilities, and historical data analysis.

use crate::models::{
    currency::Currency,
    transaction::{Transaction, TransactionType, TransactionStatus},
};
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeSet};
use std::fmt::{Display, Formatter, Result as FmtResult};
use thiserror::Error;
use uuid::Uuid;

/// Historical transaction record with audit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistoryEntry {
    /// The transaction at this point in history
    pub transaction: Transaction,
    /// When this history entry was created
    pub recorded_at: DateTime<Utc>,
    /// What operation caused this history entry
    pub operation: HistoryOperation,
    /// User who performed the operation
    pub performed_by: Option<String>,
    /// Previous version of the transaction (for updates)
    pub previous_version: Option<Box<Transaction>>,
    /// Reason for the change
    pub change_reason: Option<String>,
    /// Additional context or notes
    pub notes: Option<String>,
}

/// Type of operation that created a history entry
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HistoryOperation {
    /// Transaction was created
    Created,
    /// Transaction was updated/modified
    Updated,
    /// Transaction was cancelled
    Cancelled,
    /// Transaction status was changed
    StatusChanged,
    /// Transaction was deleted (soft delete)
    Deleted,
    /// Transaction was restored from deletion
    Restored,
    /// Automated system operation
    SystemOperation,
}

/// Criteria for filtering transaction history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryFilter {
    /// Filter by transaction IDs
    pub transaction_ids: Option<Vec<Uuid>>,
    /// Filter by account IDs
    pub account_ids: Option<Vec<Uuid>>,
    /// Filter by transaction types
    pub transaction_types: Option<Vec<TransactionType>>,
    /// Filter by transaction statuses
    pub statuses: Option<Vec<TransactionStatus>>,
    /// Filter by currencies
    pub currencies: Option<Vec<Currency>>,
    /// Filter by date range (effective date of transaction)
    pub date_range: Option<DateRange>,
    /// Filter by history operation types
    pub operations: Option<Vec<HistoryOperation>>,
    /// Filter by amount range
    pub amount_range: Option<AmountRange>,
    /// Filter by categories
    pub categories: Option<Vec<String>>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Filter by user who performed operations
    pub performed_by: Option<Vec<String>>,
    /// Text search in descriptions and notes
    pub text_search: Option<String>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Skip number of results (for pagination)
    pub offset: Option<usize>,
}

/// Date range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (inclusive)
    pub start: DateTime<Utc>,
    /// End date (inclusive)
    pub end: DateTime<Utc>,
}

/// Amount range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountRange {
    /// Minimum amount (inclusive)
    pub min: Decimal,
    /// Maximum amount (inclusive)
    pub max: Decimal,
    /// Currency for the amount range
    pub currency: Currency,
}

/// Sort options for transaction history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    /// Sort by transaction creation date (newest first)
    CreatedDateDesc,
    /// Sort by transaction creation date (oldest first)
    CreatedDateAsc,
    /// Sort by effective date (newest first)
    EffectiveDateDesc,
    /// Sort by effective date (oldest first)
    EffectiveDateAsc,
    /// Sort by amount (highest first)
    AmountDesc,
    /// Sort by amount (lowest first)
    AmountAsc,
    /// Sort by history recorded date (newest first)
    RecordedDateDesc,
    /// Sort by history recorded date (oldest first)
    RecordedDateAsc,
}

/// Error types for transaction history operations
#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Transaction history not found for ID: {id}")]
    NotFound { id: Uuid },
    #[error("Invalid date range: start ({start}) must be before end ({end})")]
    InvalidDateRange { start: DateTime<Utc>, end: DateTime<Utc> },
    #[error("Invalid amount range: min ({min}) must be less than or equal to max ({max})")]
    InvalidAmountRange { min: Decimal, max: Decimal },
    #[error("Search query too broad: {message}")]
    SearchTooBroad { message: String },
    #[error("History operation error: {message}")]
    OperationError { message: String },
}

/// Statistics about transaction history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatistics {
    /// Total number of history entries
    pub total_entries: usize,
    /// Number of unique transactions
    pub unique_transactions: usize,
    /// Entries by operation type
    pub operations_count: HashMap<HistoryOperation, usize>,
    /// Date range of all entries
    pub date_range: Option<DateRange>,
    /// Most active users
    pub top_users: Vec<(String, usize)>,
    /// Most frequently updated transactions
    pub most_updated_transactions: Vec<(Uuid, usize)>,
}

/// Transaction history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistory {
    /// All history entries indexed by transaction ID
    entries: HashMap<Uuid, Vec<TransactionHistoryEntry>>,
    /// Index by recorded date for efficient time-based queries
    date_index: BTreeSet<(DateTime<Utc>, Uuid)>,
    /// Index by operation type
    operation_index: HashMap<HistoryOperation, BTreeSet<(DateTime<Utc>, Uuid)>>,
    /// Index by user
    user_index: HashMap<String, BTreeSet<(DateTime<Utc>, Uuid)>>,
    /// Statistics cache
    statistics: Option<HistoryStatistics>,
    /// When statistics were last calculated
    statistics_updated: Option<DateTime<Utc>>,
}

impl TransactionHistory {
    /// Creates a new transaction history manager
    pub fn new() -> Self {
        TransactionHistory {
            entries: HashMap::new(),
            date_index: BTreeSet::new(),
            operation_index: HashMap::new(),
            user_index: HashMap::new(),
            statistics: None,
            statistics_updated: None,
        }
    }

    /// Records a new transaction creation
    pub fn record_creation(
        &mut self,
        transaction: Transaction,
        performed_by: Option<String>,
        notes: Option<String>,
    ) {
        let entry = TransactionHistoryEntry {
            transaction: transaction.clone(),
            recorded_at: Utc::now(),
            operation: HistoryOperation::Created,
            performed_by: performed_by.clone(),
            previous_version: None,
            change_reason: Some("Transaction created".to_string()),
            notes,
        };

        self.add_entry(entry);
    }

    /// Records a transaction update
    pub fn record_update(
        &mut self,
        previous_transaction: Transaction,
        updated_transaction: Transaction,
        performed_by: Option<String>,
        change_reason: Option<String>,
        notes: Option<String>,
    ) {
        let entry = TransactionHistoryEntry {
            transaction: updated_transaction.clone(),
            recorded_at: Utc::now(),
            operation: HistoryOperation::Updated,
            performed_by,
            previous_version: Some(Box::new(previous_transaction)),
            change_reason,
            notes,
        };

        self.add_entry(entry);
    }

    /// Records a status change
    pub fn record_status_change(
        &mut self,
        transaction: Transaction,
        previous_status: TransactionStatus,
        performed_by: Option<String>,
        notes: Option<String>,
    ) {
        let entry = TransactionHistoryEntry {
            transaction: transaction.clone(),
            recorded_at: Utc::now(),
            operation: HistoryOperation::StatusChanged,
            performed_by,
            previous_version: None,
            change_reason: Some(format!("Status changed from {:?} to {:?}", previous_status, transaction.status)),
            notes,
        };

        self.add_entry(entry);
    }

    /// Records a transaction cancellation
    pub fn record_cancellation(
        &mut self,
        transaction: Transaction,
        performed_by: Option<String>,
        reason: Option<String>,
        notes: Option<String>,
    ) {
        let entry = TransactionHistoryEntry {
            transaction: transaction.clone(),
            recorded_at: Utc::now(),
            operation: HistoryOperation::Cancelled,
            performed_by,
            previous_version: None,
            change_reason: reason,
            notes,
        };

        self.add_entry(entry);
    }

    /// Adds a history entry to all indexes
    fn add_entry(&mut self, entry: TransactionHistoryEntry) {
        let transaction_id = entry.transaction.id;
        let recorded_at = entry.recorded_at;
        let operation = entry.operation.clone();
        let performed_by = entry.performed_by.clone();

        // Add to main entries
        self.entries
            .entry(transaction_id)
            .or_insert_with(Vec::new)
            .push(entry);

        // Add to date index
        self.date_index.insert((recorded_at, transaction_id));

        // Add to operation index
        self.operation_index
            .entry(operation)
            .or_insert_with(BTreeSet::new)
            .insert((recorded_at, transaction_id));

        // Add to user index
        if let Some(user) = performed_by {
            self.user_index
                .entry(user)
                .or_insert_with(BTreeSet::new)
                .insert((recorded_at, transaction_id));
        }

        // Invalidate statistics cache
        self.statistics = None;
        self.statistics_updated = None;
    }

    /// Gets the complete history for a specific transaction
    pub fn get_transaction_history(&self, transaction_id: &Uuid) -> Option<&Vec<TransactionHistoryEntry>> {
        self.entries.get(transaction_id)
    }

    /// Gets the latest version of a transaction from history
    pub fn get_latest_transaction(&self, transaction_id: &Uuid) -> Option<&Transaction> {
        self.entries.get(transaction_id)?
            .last()
            .map(|entry| &entry.transaction)
    }

    /// Filters transaction history based on criteria
    pub fn filter_history(&self, filter: &HistoryFilter, sort: SortOrder) -> Vec<&TransactionHistoryEntry> {
        let mut results = Vec::new();

        // Start with all entries
        for entries_list in self.entries.values() {
            for entry in entries_list {
                if self.matches_filter(entry, filter) {
                    results.push(entry);
                }
            }
        }

        // Sort results
        self.sort_entries(&mut results, sort);

        // Apply pagination
        if let Some(offset) = filter.offset {
            if offset < results.len() {
                results = results.into_iter().skip(offset).collect();
            } else {
                results.clear();
            }
        }

        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        results
    }

    /// Searches transaction history with text query
    pub fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<&TransactionHistoryEntry>, HistoryError> {
        if query.len() < 3 {
            return Err(HistoryError::SearchTooBroad {
                message: "Search query must be at least 3 characters".to_string(),
            });
        }

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for entries_list in self.entries.values() {
            for entry in entries_list {
                // Search in description
                if let Some(desc) = &entry.transaction.description {
                    if desc.to_lowercase().contains(&query_lower) {
                        results.push(entry);
                        continue;
                    }
                }

                // Search in notes
                if let Some(notes) = &entry.notes {
                    if notes.to_lowercase().contains(&query_lower) {
                        results.push(entry);
                        continue;
                    }
                }

                // Search in change reason
                if let Some(reason) = &entry.change_reason {
                    if reason.to_lowercase().contains(&query_lower) {
                        results.push(entry);
                        continue;
                    }
                }

                // Search in external reference
                if let Some(ref_id) = &entry.transaction.external_reference {
                    if ref_id.to_lowercase().contains(&query_lower) {
                        results.push(entry);
                        continue;
                    }
                }

                // Search in category
                if let Some(category) = &entry.transaction.category {
                    if category.to_lowercase().contains(&query_lower) {
                        results.push(entry);
                        continue;
                    }
                }
            }
        }

        // Sort by relevance (most recent first)
        results.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));

        if let Some(limit) = limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Gets transactions modified within a time period
    pub fn get_recent_changes(&self, since: DateTime<Utc>) -> Vec<&TransactionHistoryEntry> {
        let mut results = Vec::new();

        for (recorded_at, transaction_id) in &self.date_index {
            if *recorded_at >= since {
                if let Some(entries) = self.entries.get(transaction_id) {
                    if let Some(entry) = entries.iter().find(|e| e.recorded_at == *recorded_at) {
                        results.push(entry);
                    }
                }
            }
        }

        results
    }

    /// Gets history statistics
    pub fn get_statistics(&mut self) -> &HistoryStatistics {
        // Check if statistics need to be recalculated
        let needs_update = self.statistics.is_none() ||
            self.statistics_updated.map_or(true, |updated| {
                Utc::now() - updated > Duration::hours(1)
            });

        if needs_update {
            self.calculate_statistics();
        }

        self.statistics.as_ref().unwrap()
    }

    /// Calculates fresh statistics
    fn calculate_statistics(&mut self) {
        let mut operations_count = HashMap::new();
        let mut user_activity = HashMap::new();
        let mut transaction_update_count = HashMap::new();
        let mut earliest_date: Option<DateTime<Utc>> = None;
        let mut latest_date: Option<DateTime<Utc>> = None;

        let total_entries = self.entries.values().map(|v| v.len()).sum();
        let unique_transactions = self.entries.len();

        for entries_list in self.entries.values() {
            let transaction_id = entries_list.first().map(|e| e.transaction.id).unwrap_or_default();
            transaction_update_count.insert(transaction_id, entries_list.len());

            for entry in entries_list {
                // Count operations
                *operations_count.entry(entry.operation.clone()).or_insert(0) += 1;

                // Count user activity
                if let Some(ref user) = entry.performed_by {
                    *user_activity.entry(user.clone()).or_insert(0) += 1;
                }

                // Track date range
                if earliest_date.is_none() || Some(entry.recorded_at) < earliest_date {
                    earliest_date = Some(entry.recorded_at);
                }
                if latest_date.is_none() || Some(entry.recorded_at) > latest_date {
                    latest_date = Some(entry.recorded_at);
                }
            }
        }

        // Top users by activity
        let mut top_users: Vec<_> = user_activity.into_iter().collect();
        top_users.sort_by(|a, b| b.1.cmp(&a.1));
        top_users.truncate(10);

        // Most updated transactions
        let mut most_updated: Vec<_> = transaction_update_count.into_iter().collect();
        most_updated.sort_by(|a, b| b.1.cmp(&a.1));
        most_updated.truncate(10);

        let date_range = if let (Some(start), Some(end)) = (earliest_date, latest_date) {
            Some(DateRange { start, end })
        } else {
            None
        };

        self.statistics = Some(HistoryStatistics {
            total_entries,
            unique_transactions,
            operations_count,
            date_range,
            top_users,
            most_updated_transactions: most_updated,
        });

        self.statistics_updated = Some(Utc::now());
    }

    /// Checks if an entry matches the filter criteria
    fn matches_filter(&self, entry: &TransactionHistoryEntry, filter: &HistoryFilter) -> bool {
        // Filter by transaction IDs
        if let Some(ref ids) = filter.transaction_ids {
            if !ids.contains(&entry.transaction.id) {
                return false;
            }
        }

        // Filter by account IDs
        if let Some(ref account_ids) = filter.account_ids {
            let matches_from = entry.transaction.from_account_id
                .map_or(false, |id| account_ids.contains(&id));
            let matches_to = entry.transaction.to_account_id
                .map_or(false, |id| account_ids.contains(&id));

            if !matches_from && !matches_to {
                return false;
            }
        }

        // Filter by transaction types
        if let Some(ref types) = filter.transaction_types {
            if !types.contains(&entry.transaction.transaction_type) {
                return false;
            }
        }

        // Filter by statuses
        if let Some(ref statuses) = filter.statuses {
            if !statuses.contains(&entry.transaction.status) {
                return false;
            }
        }

        // Filter by currencies
        if let Some(ref currencies) = filter.currencies {
            if !currencies.contains(&entry.transaction.currency) {
                return false;
            }
        }

        // Filter by date range
        if let Some(ref date_range) = filter.date_range {
            let effective_date = entry.transaction.effective_date();
            if effective_date < date_range.start || effective_date > date_range.end {
                return false;
            }
        }

        // Filter by operations
        if let Some(ref operations) = filter.operations {
            if !operations.contains(&entry.operation) {
                return false;
            }
        }

        // Filter by amount range
        if let Some(ref amount_range) = filter.amount_range {
            if entry.transaction.currency == amount_range.currency {
                if entry.transaction.amount < amount_range.min || entry.transaction.amount > amount_range.max {
                    return false;
                }
            }
        }

        // Filter by categories
        if let Some(ref categories) = filter.categories {
            let tx_category = entry.transaction.category.as_deref().unwrap_or("Uncategorized");
            if !categories.contains(&tx_category.to_string()) {
                return false;
            }
        }

        // Filter by tags
        if let Some(ref tags) = filter.tags {
            if !tags.iter().any(|tag| entry.transaction.tags.contains(tag)) {
                return false;
            }
        }

        // Filter by performed by
        if let Some(ref users) = filter.performed_by {
            if let Some(ref user) = entry.performed_by {
                if !users.contains(user) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Text search
        if let Some(ref query) = filter.text_search {
            let query_lower = query.to_lowercase();
            let matches = entry.transaction.description
                .as_ref()
                .map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
                || entry.notes
                    .as_ref()
                    .map_or(false, |notes| notes.to_lowercase().contains(&query_lower));

            if !matches {
                return false;
            }
        }

        true
    }

    /// Sorts entries based on the specified order
    fn sort_entries(&self, entries: &mut Vec<&TransactionHistoryEntry>, sort: SortOrder) {
        match sort {
            SortOrder::CreatedDateDesc => {
                entries.sort_by(|a, b| b.transaction.created_at.cmp(&a.transaction.created_at));
            }
            SortOrder::CreatedDateAsc => {
                entries.sort_by(|a, b| a.transaction.created_at.cmp(&b.transaction.created_at));
            }
            SortOrder::EffectiveDateDesc => {
                entries.sort_by(|a, b| b.transaction.effective_date().cmp(&a.transaction.effective_date()));
            }
            SortOrder::EffectiveDateAsc => {
                entries.sort_by(|a, b| a.transaction.effective_date().cmp(&b.transaction.effective_date()));
            }
            SortOrder::AmountDesc => {
                entries.sort_by(|a, b| b.transaction.amount.cmp(&a.transaction.amount));
            }
            SortOrder::AmountAsc => {
                entries.sort_by(|a, b| a.transaction.amount.cmp(&b.transaction.amount));
            }
            SortOrder::RecordedDateDesc => {
                entries.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));
            }
            SortOrder::RecordedDateAsc => {
                entries.sort_by(|a, b| a.recorded_at.cmp(&b.recorded_at));
            }
        }
    }
}

impl Default for TransactionHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for HistoryOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            HistoryOperation::Created => write!(f, "Created"),
            HistoryOperation::Updated => write!(f, "Updated"),
            HistoryOperation::Cancelled => write!(f, "Cancelled"),
            HistoryOperation::StatusChanged => write!(f, "Status Changed"),
            HistoryOperation::Deleted => write!(f, "Deleted"),
            HistoryOperation::Restored => write!(f, "Restored"),
            HistoryOperation::SystemOperation => write!(f, "System Operation"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{Transaction, TransactionType};

    fn create_test_transaction() -> Transaction {
        Transaction::new(
            TransactionType::Deposit,
            Decimal::new(100, 0),
            Currency::USD,
        ).unwrap()
    }

    #[test]
    fn test_history_creation() {
        let history = TransactionHistory::new();
        assert!(history.entries.is_empty());
        assert!(history.date_index.is_empty());
        assert!(history.operation_index.is_empty());
    }

    #[test]
    fn test_record_creation() {
        let mut history = TransactionHistory::new();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;

        history.record_creation(transaction, Some("test_user".to_string()), None);

        assert!(history.entries.contains_key(&transaction_id));
        let entries = history.get_transaction_history(&transaction_id).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, HistoryOperation::Created);
        assert_eq!(entries[0].performed_by, Some("test_user".to_string()));
    }

    #[test]
    fn test_record_update() {
        let mut history = TransactionHistory::new();
        let mut transaction = create_test_transaction();
        let transaction_id = transaction.id;

        // Record creation
        history.record_creation(transaction.clone(), Some("user1".to_string()), None);

        // Update transaction
        let previous = transaction.clone();
        transaction.description = Some("Updated description".to_string());

        // Record update
        history.record_update(
            previous,
            transaction,
            Some("user2".to_string()),
            Some("Description updated".to_string()),
            None,
        );

        let entries = history.get_transaction_history(&transaction_id).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].operation, HistoryOperation::Updated);
        assert_eq!(entries[1].performed_by, Some("user2".to_string()));
        assert!(entries[1].previous_version.is_some());
    }

    #[test]
    fn test_filter_history() {
        let mut history = TransactionHistory::new();
        let transaction1 = create_test_transaction();
        let mut transaction2 = create_test_transaction();
        transaction2.transaction_type = TransactionType::Withdrawal;

        history.record_creation(transaction1.clone(), Some("user1".to_string()), None);
        history.record_creation(transaction2.clone(), Some("user2".to_string()), None);

        // Filter by transaction type
        let filter = HistoryFilter {
            transaction_types: Some(vec![TransactionType::Deposit]),
            ..Default::default()
        };

        let results = history.filter_history(&filter, SortOrder::CreatedDateDesc);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].transaction.transaction_type, TransactionType::Deposit);
    }

    #[test]
    fn test_search() {
        let mut history = TransactionHistory::new();
        let mut transaction = create_test_transaction();
        transaction.description = Some("Coffee purchase".to_string());

        history.record_creation(transaction, Some("user1".to_string()), Some("Morning coffee".to_string()));

        // Search in description
        let results = history.search("coffee", None).unwrap();
        assert_eq!(results.len(), 1);

        // Search in notes
        let results = history.search("morning", None).unwrap();
        assert_eq!(results.len(), 1);

        // Search with no results
        let results = history.search("xyz", None).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_latest_transaction() {
        let mut history = TransactionHistory::new();
        let mut transaction = create_test_transaction();
        let transaction_id = transaction.id;

        // Record creation
        history.record_creation(transaction.clone(), Some("user1".to_string()), None);

        // Update transaction
        let previous = transaction.clone();
        transaction.description = Some("Updated".to_string());
        history.record_update(previous, transaction.clone(), Some("user2".to_string()), None, None);

        // Get latest version
        let latest = history.get_latest_transaction(&transaction_id).unwrap();
        assert_eq!(latest.description, Some("Updated".to_string()));
    }
}

impl Default for HistoryFilter {
    fn default() -> Self {
        HistoryFilter {
            transaction_ids: None,
            account_ids: None,
            transaction_types: None,
            statuses: None,
            currencies: None,
            date_range: None,
            operations: None,
            amount_range: None,
            categories: None,
            tags: None,
            performed_by: None,
            text_search: None,
            limit: None,
            offset: None,
        }
    }
}