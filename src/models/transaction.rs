//! Transaction module for FerrumFinance
//!
//! This module provides comprehensive transaction management including different transaction types,
//! status tracking, and multi-currency support for financial operations.

use crate::models::currency::Currency;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use thiserror::Error;
use uuid::Uuid;

/// Types of financial transactions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    /// Money deposit into an account
    Deposit,
    /// Money withdrawal from an account
    Withdrawal,
    /// Transfer between two accounts
    Transfer,
    /// Payment to external party
    Payment,
    /// Income/revenue transaction
    Income,
    /// Expense transaction
    Expense,
    /// Loan disbursement
    LoanDisbursement,
    /// Loan payment/repayment
    LoanPayment,
    /// Interest accrual
    Interest,
    /// Fee charged
    Fee,
    /// Dividend payment
    Dividend,
    /// Investment purchase
    Investment,
    /// Currency exchange
    CurrencyExchange,
    /// Adjustment/correction
    Adjustment,
    /// Refund transaction
    Refund,
}

/// Current status of a transaction
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction is pending processing
    Pending,
    /// Transaction has been completed successfully
    Completed,
    /// Transaction failed to process
    Failed,
    /// Transaction was cancelled
    Cancelled,
    /// Transaction is being processed
    Processing,
    /// Transaction requires authorization
    Authorized,
    /// Transaction has been settled
    Settled,
    /// Transaction is on hold
    OnHold,
    /// Transaction is scheduled for future processing
    Scheduled,
    /// Transaction was declined/rejected
    Declined,
}

/// Priority level for transaction processing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TransactionType::Deposit => write!(f, "Deposit"),
            TransactionType::Withdrawal => write!(f, "Withdrawal"),
            TransactionType::Transfer => write!(f, "Transfer"),
            TransactionType::Payment => write!(f, "Payment"),
            TransactionType::Income => write!(f, "Income"),
            TransactionType::Expense => write!(f, "Expense"),
            TransactionType::LoanDisbursement => write!(f, "LoanDisbursement"),
            TransactionType::LoanPayment => write!(f, "LoanPayment"),
            TransactionType::Interest => write!(f, "Interest"),
            TransactionType::Fee => write!(f, "Fee"),
            TransactionType::Dividend => write!(f, "Dividend"),
            TransactionType::Investment => write!(f, "Investment"),
            TransactionType::CurrencyExchange => write!(f, "CurrencyExchange"),
            TransactionType::Adjustment => write!(f, "Adjustment"),
            TransactionType::Refund => write!(f, "Refund"),
        }
    }
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Completed => write!(f, "Completed"),
            TransactionStatus::Failed => write!(f, "Failed"),
            TransactionStatus::Cancelled => write!(f, "Cancelled"),
            TransactionStatus::Processing => write!(f, "Processing"),
            TransactionStatus::Authorized => write!(f, "Authorized"),
            TransactionStatus::Settled => write!(f, "Settled"),
            TransactionStatus::OnHold => write!(f, "OnHold"),
            TransactionStatus::Scheduled => write!(f, "Scheduled"),
            TransactionStatus::Declined => write!(f, "Declined"),
        }
    }
}

impl Display for TransactionPriority {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TransactionPriority::Low => write!(f, "Low"),
            TransactionPriority::Normal => write!(f, "Normal"),
            TransactionPriority::High => write!(f, "High"),
            TransactionPriority::Urgent => write!(f, "Urgent"),
        }
    }
}

/// Error types for transaction operations
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Invalid transaction amount: {amount}")]
    InvalidAmount { amount: Decimal },
    #[error("Transaction not found: {id}")]
    NotFound { id: Uuid },
    #[error("Cannot modify transaction in status: {status:?}")]
    InvalidStatus { status: TransactionStatus },
    #[error("Account not found: {account_id}")]
    AccountNotFound { account_id: Uuid },
    #[error("Insufficient balance for transaction")]
    InsufficientBalance,
    #[error("Currency mismatch: expected {expected}, got {actual}")]
    CurrencyMismatch { expected: Currency, actual: Currency },
    #[error("Invalid transaction type for operation: {transaction_type:?}")]
    InvalidTransactionType { transaction_type: TransactionType },
}

/// Represents a financial transaction in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique identifier for the transaction
    pub id: Uuid,
    /// Type of transaction
    pub transaction_type: TransactionType,
    /// Current status of the transaction
    pub status: TransactionStatus,
    /// Transaction amount (always positive, direction determined by type)
    pub amount: Decimal,
    /// Currency of the transaction
    pub currency: Currency,
    /// Source account ID (for transfers, withdrawals, payments)
    pub from_account_id: Option<Uuid>,
    /// Destination account ID (for transfers, deposits, income)
    pub to_account_id: Option<Uuid>,
    /// External reference ID (bank transaction ID, etc.)
    pub external_reference: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>,
    /// Transaction priority
    pub priority: TransactionPriority,
    /// When the transaction was created
    pub created_at: DateTime<Utc>,
    /// When the transaction was last updated
    pub updated_at: DateTime<Utc>,
    /// When the transaction was executed/completed
    pub executed_at: Option<DateTime<Utc>>,
    /// Scheduled execution time (for future transactions)
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Reference to related transaction (for transfers, refunds)
    pub related_transaction_id: Option<Uuid>,
    /// Category for expense/income classification
    pub category: Option<String>,
    /// Tags for organization and filtering
    pub tags: Vec<String>,
    /// Exchange rate used (for currency conversions)
    pub exchange_rate: Option<Decimal>,
    /// Fee charged for this transaction
    pub fee: Option<Decimal>,
    /// Fee currency (may differ from transaction currency)
    pub fee_currency: Option<Currency>,
    /// Notes or comments
    pub notes: Option<String>,
    /// User who created/authorized the transaction
    pub created_by: Option<String>,
    /// Recurring transaction pattern (if applicable)
    pub recurring_pattern: Option<RecurringPattern>,
}

/// Pattern for recurring transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurringPattern {
    /// Frequency of recurrence
    pub frequency: RecurringFrequency,
    /// Interval multiplier (every N periods)
    pub interval: u32,
    /// End date for recurring pattern
    pub end_date: Option<DateTime<Utc>>,
    /// Maximum number of occurrences
    pub max_occurrences: Option<u32>,
    /// Number of occurrences created so far
    pub occurrences_created: u32,
    /// Whether the pattern is currently active
    pub is_active: bool,
}

/// Frequency options for recurring transactions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurringFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl Transaction {
    /// Creates a new transaction
    pub fn new(
        transaction_type: TransactionType,
        amount: Decimal,
        currency: Currency,
    ) -> Result<Self, TransactionError> {
        if amount <= Decimal::ZERO {
            return Err(TransactionError::InvalidAmount { amount });
        }

        let now = Utc::now();

        Ok(Transaction {
            id: Uuid::new_v4(),
            transaction_type,
            status: TransactionStatus::Pending,
            amount,
            currency,
            from_account_id: None,
            to_account_id: None,
            external_reference: None,
            description: None,
            metadata: HashMap::new(),
            priority: TransactionPriority::Normal,
            created_at: now,
            updated_at: now,
            executed_at: None,
            scheduled_at: None,
            related_transaction_id: None,
            category: None,
            tags: Vec::new(),
            exchange_rate: None,
            fee: None,
            fee_currency: None,
            notes: None,
            created_by: None,
            recurring_pattern: None,
        })
    }

    /// Creates a transfer transaction between two accounts
    pub fn create_transfer(
        amount: Decimal,
        currency: Currency,
        from_account_id: Uuid,
        to_account_id: Uuid,
        description: Option<String>,
    ) -> Result<Self, TransactionError> {
        let mut transaction = Transaction::new(TransactionType::Transfer, amount, currency)?;
        transaction.from_account_id = Some(from_account_id);
        transaction.to_account_id = Some(to_account_id);
        transaction.description = description;
        Ok(transaction)
    }

    /// Creates a deposit transaction
    pub fn create_deposit(
        amount: Decimal,
        currency: Currency,
        to_account_id: Uuid,
        description: Option<String>,
    ) -> Result<Self, TransactionError> {
        let mut transaction = Transaction::new(TransactionType::Deposit, amount, currency)?;
        transaction.to_account_id = Some(to_account_id);
        transaction.description = description;
        Ok(transaction)
    }

    /// Creates a withdrawal transaction
    pub fn create_withdrawal(
        amount: Decimal,
        currency: Currency,
        from_account_id: Uuid,
        description: Option<String>,
    ) -> Result<Self, TransactionError> {
        let mut transaction = Transaction::new(TransactionType::Withdrawal, amount, currency)?;
        transaction.from_account_id = Some(from_account_id);
        transaction.description = description;
        Ok(transaction)
    }

    /// Updates the transaction status
    pub fn set_status(&mut self, status: TransactionStatus) {
        let is_completed = status == TransactionStatus::Completed;
        self.status = status;
        self.updated_at = Utc::now();

        if is_completed && self.executed_at.is_none() {
            self.executed_at = Some(Utc::now());
        }
    }

    /// Adds metadata to the transaction
    pub fn add_metadata<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
    }

    /// Adds a tag to the transaction
    pub fn add_tag<S: Into<String>>(&mut self, tag: S) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Removes a tag from the transaction
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Sets the fee for the transaction
    pub fn set_fee(&mut self, fee: Decimal, currency: Currency) {
        self.fee = Some(fee);
        self.fee_currency = Some(currency);
        self.updated_at = Utc::now();
    }

    /// Gets the total cost including fees
    pub fn total_cost(&self) -> Decimal {
        let base_amount = self.amount;
        let fee_amount = self.fee.unwrap_or(Decimal::ZERO);

        // If fee is in the same currency, add directly
        if self.fee_currency.is_none() || self.fee_currency == Some(self.currency.clone()) {
            base_amount + fee_amount
        } else {
            // Fee is in different currency - would need exchange rate conversion
            // For now, just return base amount
            base_amount
        }
    }

    /// Checks if the transaction can be modified
    pub fn can_be_modified(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Pending | TransactionStatus::OnHold
        )
    }

    /// Checks if the transaction is completed
    pub fn is_completed(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Completed | TransactionStatus::Settled
        )
    }

    /// Checks if the transaction involves the specified account
    pub fn involves_account(&self, account_id: &Uuid) -> bool {
        self.from_account_id.as_ref() == Some(account_id) ||
        self.to_account_id.as_ref() == Some(account_id)
    }

    /// Gets the effective date for the transaction (executed_at or created_at)
    pub fn effective_date(&self) -> DateTime<Utc> {
        self.executed_at.unwrap_or(self.created_at)
    }

    /// Validates the transaction
    pub fn validate(&self) -> Result<(), TransactionError> {
        // Validate amount
        if self.amount <= Decimal::ZERO {
            return Err(TransactionError::InvalidAmount { amount: self.amount });
        }

        // Validate account requirements based on transaction type
        match self.transaction_type {
            TransactionType::Transfer => {
                if self.from_account_id.is_none() || self.to_account_id.is_none() {
                    return Err(TransactionError::InvalidTransactionType {
                        transaction_type: self.transaction_type.clone(),
                    });
                }
            }
            TransactionType::Deposit | TransactionType::Income => {
                if self.to_account_id.is_none() {
                    return Err(TransactionError::InvalidTransactionType {
                        transaction_type: self.transaction_type.clone(),
                    });
                }
            }
            TransactionType::Withdrawal | TransactionType::Expense | TransactionType::Payment => {
                if self.from_account_id.is_none() {
                    return Err(TransactionError::InvalidTransactionType {
                        transaction_type: self.transaction_type.clone(),
                    });
                }
            }
            _ => {} // Other types may have flexible account requirements
        }

        Ok(())
    }

    /// Sets a recurring pattern for the transaction
    pub fn set_recurring_pattern(&mut self, pattern: RecurringPattern) {
        self.recurring_pattern = Some(pattern);
        self.updated_at = Utc::now();
    }

    /// Removes the recurring pattern
    pub fn remove_recurring_pattern(&mut self) {
        self.recurring_pattern = None;
        self.updated_at = Utc::now();
    }
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TransactionType::Deposit => write!(f, "Deposit"),
            TransactionType::Withdrawal => write!(f, "Withdrawal"),
            TransactionType::Transfer => write!(f, "Transfer"),
            TransactionType::Payment => write!(f, "Payment"),
            TransactionType::Income => write!(f, "Income"),
            TransactionType::Expense => write!(f, "Expense"),
            TransactionType::LoanDisbursement => write!(f, "Loan Disbursement"),
            TransactionType::LoanPayment => write!(f, "Loan Payment"),
            TransactionType::Interest => write!(f, "Interest"),
            TransactionType::Fee => write!(f, "Fee"),
            TransactionType::Dividend => write!(f, "Dividend"),
            TransactionType::Investment => write!(f, "Investment"),
            TransactionType::CurrencyExchange => write!(f, "Currency Exchange"),
            TransactionType::Adjustment => write!(f, "Adjustment"),
            TransactionType::Refund => write!(f, "Refund"),
        }
    }
}

impl Display for TransactionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Completed => write!(f, "Completed"),
            TransactionStatus::Failed => write!(f, "Failed"),
            TransactionStatus::Cancelled => write!(f, "Cancelled"),
            TransactionStatus::Processing => write!(f, "Processing"),
            TransactionStatus::Authorized => write!(f, "Authorized"),
            TransactionStatus::Settled => write!(f, "Settled"),
            TransactionStatus::OnHold => write!(f, "On Hold"),
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} - {} {} {} [{}]",
            self.transaction_type,
            self.currency.symbol(),
            self.amount,
            self.description.as_deref().unwrap_or(""),
            self.status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let transaction = Transaction::new(
            TransactionType::Deposit,
            Decimal::new(100, 0),
            Currency::USD,
        );

        assert!(transaction.is_ok());
        let tx = transaction.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::Deposit);
        assert_eq!(tx.amount, Decimal::new(100, 0));
        assert_eq!(tx.currency, Currency::USD);
        assert_eq!(tx.status, TransactionStatus::Pending);
    }

    #[test]
    fn test_invalid_amount() {
        let transaction = Transaction::new(
            TransactionType::Deposit,
            Decimal::ZERO,
            Currency::USD,
        );

        assert!(transaction.is_err());
    }

    #[test]
    fn test_transfer_creation() {
        let from_account = Uuid::new_v4();
        let to_account = Uuid::new_v4();

        let transfer = Transaction::create_transfer(
            Decimal::new(100, 0),
            Currency::USD,
            from_account,
            to_account,
            Some("Test transfer".to_string()),
        );

        assert!(transfer.is_ok());
        let tx = transfer.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.from_account_id, Some(from_account));
        assert_eq!(tx.to_account_id, Some(to_account));
    }

    #[test]
    fn test_transaction_status_update() {
        let mut transaction = Transaction::new(
            TransactionType::Deposit,
            Decimal::new(100, 0),
            Currency::USD,
        ).unwrap();

        assert_eq!(transaction.status, TransactionStatus::Pending);
        assert!(transaction.executed_at.is_none());

        transaction.set_status(TransactionStatus::Completed);
        assert_eq!(transaction.status, TransactionStatus::Completed);
        assert!(transaction.executed_at.is_some());
    }

    #[test]
    fn test_transaction_validation() {
        // Valid transfer
        let mut transaction = Transaction::new(
            TransactionType::Transfer,
            Decimal::new(100, 0),
            Currency::USD,
        ).unwrap();

        transaction.from_account_id = Some(Uuid::new_v4());
        transaction.to_account_id = Some(Uuid::new_v4());
        assert!(transaction.validate().is_ok());

        // Invalid transfer (missing accounts)
        transaction.from_account_id = None;
        assert!(transaction.validate().is_err());
    }

    #[test]
    fn test_tag_management() {
        let mut transaction = Transaction::new(
            TransactionType::Expense,
            Decimal::new(50, 0),
            Currency::USD,
        ).unwrap();

        transaction.add_tag("business");
        transaction.add_tag("travel");
        assert_eq!(transaction.tags.len(), 2);

        transaction.remove_tag("business");
        assert_eq!(transaction.tags.len(), 1);
        assert!(!transaction.tags.contains(&"business".to_string()));
    }

    #[test]
    fn test_total_cost_calculation() {
        let mut transaction = Transaction::new(
            TransactionType::Payment,
            Decimal::new(100, 0),
            Currency::USD,
        ).unwrap();

        // Without fee
        assert_eq!(transaction.total_cost(), Decimal::new(100, 0));

        // With fee in same currency
        transaction.set_fee(Decimal::new(5, 0), Currency::USD);
        assert_eq!(transaction.total_cost(), Decimal::new(105, 0));
    }
}