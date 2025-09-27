//! Account module for FerrumFinance
//!
//! This module provides account management functionality including different account types,
//! balance tracking, and comprehensive account operations for financial management.

use crate::models::currency::Currency;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use thiserror::Error;
use uuid::Uuid;

/// Types of accounts following standard accounting principles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Asset accounts (cash, receivables, inventory, etc.)
    Asset,
    /// Liability accounts (payables, loans, credit cards, etc.)
    Liability,
    /// Equity accounts (owner's equity, retained earnings, etc.)
    Equity,
    /// Revenue/Income accounts
    Revenue,
    /// Expense accounts
    Expense,
}

/// Account status indicating the current state of the account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    /// Account is active and can be used for transactions
    Active,
    /// Account is temporarily suspended
    Suspended,
    /// Account is closed and cannot be used
    Closed,
    /// Account is pending activation
    Pending,
}

impl Display for AccountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AccountType::Asset => write!(f, "Asset"),
            AccountType::Liability => write!(f, "Liability"),
            AccountType::Equity => write!(f, "Equity"),
            AccountType::Revenue => write!(f, "Revenue"),
            AccountType::Expense => write!(f, "Expense"),
        }
    }
}

impl Display for AccountStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AccountStatus::Active => write!(f, "Active"),
            AccountStatus::Suspended => write!(f, "Suspended"),
            AccountStatus::Closed => write!(f, "Closed"),
            AccountStatus::Pending => write!(f, "Pending"),
        }
    }
}

/// Error types for account operations
#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: Decimal, available: Decimal },
    #[error("Account is not active: {status:?}")]
    AccountNotActive { status: AccountStatus },
    #[error("Currency not supported: {currency}")]
    UnsupportedCurrency { currency: String },
    #[error("Invalid account operation: {reason}")]
    InvalidOperation { reason: String },
}

/// Represents a financial account in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier for the account
    pub id: Uuid,
    /// Human-readable account name
    pub name: String,
    /// Type of account (Asset, Liability, etc.)
    pub account_type: AccountType,
    /// Primary currency for this account
    pub primary_currency: Currency,
    /// Current balance in the primary currency
    pub balance: Decimal,
    /// Multi-currency balances (including primary currency)
    pub currency_balances: HashMap<Currency, Decimal>,
    /// Account status
    pub status: AccountStatus,
    /// Optional account number for external reference
    pub account_number: Option<String>,
    /// Optional description or notes
    pub description: Option<String>,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Optional parent account ID for hierarchical accounting
    pub parent_account_id: Option<Uuid>,
    /// Tags for categorization and filtering
    pub tags: Vec<String>,
}

impl Account {
    /// Creates a new account with the specified parameters
    pub fn new<S: Into<String>>(
        name: S,
        account_type: AccountType,
        primary_currency: Currency,
    ) -> Self {
        let now = Utc::now();
        let mut currency_balances = HashMap::new();
        currency_balances.insert(primary_currency.clone(), Decimal::ZERO);

        Account {
            id: Uuid::new_v4(),
            name: name.into(),
            account_type,
            primary_currency: primary_currency.clone(),
            balance: Decimal::ZERO,
            currency_balances,
            status: AccountStatus::Active,
            account_number: None,
            description: None,
            created_at: now,
            updated_at: now,
            parent_account_id: None,
            tags: Vec::new(),
        }
    }

    /// Updates the account name and timestamp
    pub fn set_name<S: Into<String>>(&mut self, name: S) {
        self.name = name.into();
        self.updated_at = Utc::now();
    }

    /// Updates the account description
    pub fn set_description<S: Into<String>>(&mut self, description: Option<S>) {
        self.description = description.map(|d| d.into());
        self.updated_at = Utc::now();
    }

    /// Sets the account number
    pub fn set_account_number<S: Into<String>>(&mut self, account_number: Option<S>) {
        self.account_number = account_number.map(|n| n.into());
        self.updated_at = Utc::now();
    }

    /// Sets the account status
    pub fn set_status(&mut self, status: AccountStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Adds a tag to the account
    pub fn add_tag<S: Into<String>>(&mut self, tag: S) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Removes a tag from the account
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Gets the balance for a specific currency
    pub fn get_balance(&self, currency: &Currency) -> Decimal {
        self.currency_balances.get(currency).copied().unwrap_or(Decimal::ZERO)
    }

    /// Gets the balance in the primary currency
    pub fn get_primary_balance(&self) -> Decimal {
        self.balance
    }

    /// Updates the balance for a specific currency
    pub fn set_balance(&mut self, currency: Currency, amount: Decimal) -> Result<(), AccountError> {
        self.currency_balances.insert(currency.clone(), amount);

        // Update primary balance if it's the primary currency
        if currency == self.primary_currency {
            self.balance = amount;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Adds to the balance for a specific currency
    pub fn add_balance(&mut self, currency: Currency, amount: Decimal) -> Result<(), AccountError> {
        if self.status != AccountStatus::Active {
            return Err(AccountError::AccountNotActive { status: self.status.clone() });
        }

        let current_balance = self.get_balance(&currency);
        let new_balance = current_balance + amount;
        self.set_balance(currency, new_balance)?;
        Ok(())
    }

    /// Subtracts from the balance for a specific currency
    pub fn subtract_balance(&mut self, currency: Currency, amount: Decimal) -> Result<(), AccountError> {
        if self.status != AccountStatus::Active {
            return Err(AccountError::AccountNotActive { status: self.status.clone() });
        }

        let current_balance = self.get_balance(&currency);
        let new_balance = current_balance - amount;

        // Check for sufficient funds for asset accounts
        if matches!(self.account_type, AccountType::Asset) && new_balance < Decimal::ZERO {
            return Err(AccountError::InsufficientFunds {
                required: amount,
                available: current_balance,
            });
        }

        self.set_balance(currency, new_balance)?;
        Ok(())
    }

    /// Checks if the account has sufficient balance in a specific currency
    pub fn has_sufficient_balance(&self, currency: &Currency, amount: Decimal) -> bool {
        let current_balance = self.get_balance(currency);

        // For asset accounts, balance must be >= amount
        // For liability/equity accounts, we can have negative balances
        match self.account_type {
            AccountType::Asset => current_balance >= amount,
            _ => true, // Liability and equity accounts can go negative
        }
    }

    /// Gets all currencies with non-zero balances
    pub fn get_active_currencies(&self) -> Vec<Currency> {
        self.currency_balances
            .iter()
            .filter(|(_, balance)| **balance != Decimal::ZERO)
            .map(|(currency, _)| currency.clone())
            .collect()
    }

    /// Gets the total number of transactions (placeholder - would be calculated from transaction history)
    pub fn get_transaction_count(&self) -> usize {
        // This would be implemented when integrating with transaction storage
        0
    }

    /// Checks if the account is a parent account
    pub fn is_parent_account(&self) -> bool {
        // This would be implemented when checking for child accounts
        false
    }

    /// Sets the parent account
    pub fn set_parent_account(&mut self, parent_id: Option<Uuid>) {
        self.parent_account_id = parent_id;
        self.updated_at = Utc::now();
    }

    /// Validates account state consistency
    pub fn validate(&self) -> Result<(), AccountError> {
        // Ensure primary currency balance matches currency_balances
        let primary_balance = self.currency_balances.get(&self.primary_currency)
            .copied()
            .unwrap_or(Decimal::ZERO);

        if self.balance != primary_balance {
            return Err(AccountError::InvalidOperation {
                reason: "Primary balance does not match currency balance".to_string(),
            });
        }

        // Ensure account has primary currency in balances
        if !self.currency_balances.contains_key(&self.primary_currency) {
            return Err(AccountError::InvalidOperation {
                reason: "Primary currency missing from currency balances".to_string(),
            });
        }

        Ok(())
    }
}

impl AccountType {
    /// Returns true if this account type normally has a debit balance
    pub fn has_debit_balance(&self) -> bool {
        matches!(self, AccountType::Asset | AccountType::Expense)
    }

    /// Returns true if this account type normally has a credit balance
    pub fn has_credit_balance(&self) -> bool {
        matches!(self, AccountType::Liability | AccountType::Equity | AccountType::Revenue)
    }

    /// Returns all available account types
    pub fn all() -> Vec<AccountType> {
        vec![
            AccountType::Asset,
            AccountType::Liability,
            AccountType::Equity,
            AccountType::Revenue,
            AccountType::Expense,
        ]
    }
}

impl Display for AccountType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AccountType::Asset => write!(f, "Asset"),
            AccountType::Liability => write!(f, "Liability"),
            AccountType::Equity => write!(f, "Equity"),
            AccountType::Revenue => write!(f, "Revenue"),
            AccountType::Expense => write!(f, "Expense"),
        }
    }
}

impl Display for AccountStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            AccountStatus::Active => write!(f, "Active"),
            AccountStatus::Suspended => write!(f, "Suspended"),
            AccountStatus::Closed => write!(f, "Closed"),
            AccountStatus::Pending => write!(f, "Pending"),
        }
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} ({}) - {} {}",
            self.name,
            self.account_type,
            self.primary_currency.symbol(),
            self.balance
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new("Test Account", AccountType::Asset, Currency::USD);

        assert_eq!(account.name, "Test Account");
        assert_eq!(account.account_type, AccountType::Asset);
        assert_eq!(account.primary_currency, Currency::USD);
        assert_eq!(account.balance, Decimal::ZERO);
        assert_eq!(account.status, AccountStatus::Active);
    }

    #[test]
    fn test_balance_operations() {
        let mut account = Account::new("Test Account", AccountType::Asset, Currency::USD);

        // Add balance
        assert!(account.add_balance(Currency::USD, Decimal::new(100, 0)).is_ok());
        assert_eq!(account.get_balance(&Currency::USD), Decimal::new(100, 0));

        // Subtract balance
        assert!(account.subtract_balance(Currency::USD, Decimal::new(50, 0)).is_ok());
        assert_eq!(account.get_balance(&Currency::USD), Decimal::new(50, 0));
    }

    #[test]
    fn test_insufficient_funds() {
        let mut account = Account::new("Test Account", AccountType::Asset, Currency::USD);
        account.add_balance(Currency::USD, Decimal::new(100, 0)).unwrap();

        let result = account.subtract_balance(Currency::USD, Decimal::new(150, 0));
        assert!(result.is_err());

        if let Err(AccountError::InsufficientFunds { required, available }) = result {
            assert_eq!(required, Decimal::new(150, 0));
            assert_eq!(available, Decimal::new(100, 0));
        }
    }

    #[test]
    fn test_multi_currency_support() {
        let mut account = Account::new("Test Account", AccountType::Asset, Currency::USD);

        // Add EUR balance
        assert!(account.add_balance(Currency::EUR, Decimal::new(200, 0)).is_ok());
        assert_eq!(account.get_balance(&Currency::EUR), Decimal::new(200, 0));
        assert_eq!(account.get_balance(&Currency::USD), Decimal::ZERO);

        let active_currencies = account.get_active_currencies();
        assert_eq!(active_currencies.len(), 1);
        assert!(active_currencies.contains(&Currency::EUR));
    }

    #[test]
    fn test_account_type_balance_properties() {
        assert!(AccountType::Asset.has_debit_balance());
        assert!(!AccountType::Asset.has_credit_balance());

        assert!(AccountType::Liability.has_credit_balance());
        assert!(!AccountType::Liability.has_debit_balance());
    }

    #[test]
    fn test_tag_management() {
        let mut account = Account::new("Test Account", AccountType::Asset, Currency::USD);

        account.add_tag("business");
        account.add_tag("checking");
        assert_eq!(account.tags.len(), 2);
        assert!(account.tags.contains(&"business".to_string()));

        account.remove_tag("business");
        assert_eq!(account.tags.len(), 1);
        assert!(!account.tags.contains(&"business".to_string()));
    }

    #[test]
    fn test_account_validation() {
        let account = Account::new("Test Account", AccountType::Asset, Currency::USD);
        assert!(account.validate().is_ok());
    }
}