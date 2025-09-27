use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// Currency enum and implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD, EUR, GBP, JPY, CAD, AUD, CHF, CNY, INR, BRL,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::GBP => write!(f, "GBP"),
            Currency::JPY => write!(f, "JPY"),
            Currency::CAD => write!(f, "CAD"),
            Currency::AUD => write!(f, "AUD"),
            Currency::CHF => write!(f, "CHF"),
            Currency::CNY => write!(f, "CNY"),
            Currency::INR => write!(f, "INR"),
            Currency::BRL => write!(f, "BRL"),
        }
    }
}

// Account types and status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Asset, Liability, Equity, Revenue, Expense,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Asset => write!(f, "Asset"),
            AccountType::Liability => write!(f, "Liability"),
            AccountType::Equity => write!(f, "Equity"),
            AccountType::Revenue => write!(f, "Revenue"),
            AccountType::Expense => write!(f, "Expense"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active, Suspended, Closed, Pending,
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::Active => write!(f, "Active"),
            AccountStatus::Suspended => write!(f, "Suspended"),
            AccountStatus::Closed => write!(f, "Closed"),
            AccountStatus::Pending => write!(f, "Pending"),
        }
    }
}

// Transaction types and status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Income, Expense, Transfer, Investment, Refund,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Income => write!(f, "Income"),
            TransactionType::Expense => write!(f, "Expense"),
            TransactionType::Transfer => write!(f, "Transfer"),
            TransactionType::Investment => write!(f, "Investment"),
            TransactionType::Refund => write!(f, "Refund"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending, Completed, Failed, Cancelled,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Completed => write!(f, "Completed"),
            TransactionStatus::Failed => write!(f, "Failed"),
            TransactionStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

// Loan types and status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanType {
    Personal, Mortgage, Auto, Business, Student,
}

impl std::fmt::Display for LoanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoanType::Personal => write!(f, "Personal"),
            LoanType::Mortgage => write!(f, "Mortgage"),
            LoanType::Auto => write!(f, "Auto"),
            LoanType::Business => write!(f, "Business"),
            LoanType::Student => write!(f, "Student"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanStatus {
    Active, Paid, Defaulted, Suspended,
}

impl std::fmt::Display for LoanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoanStatus::Active => write!(f, "Active"),
            LoanStatus::Paid => write!(f, "Paid"),
            LoanStatus::Defaulted => write!(f, "Defaulted"),
            LoanStatus::Suspended => write!(f, "Suspended"),
        }
    }
}

// Model structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
    pub balance: Decimal,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(name: String, account_type: AccountType, currency: Currency, balance: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            account_type,
            currency,
            balance,
            status: AccountStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: Decimal,
    pub currency: Currency,
    pub description: String,
    pub account_id: Uuid,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        amount: Decimal,
        currency: Currency,
        description: String,
        account_id: Uuid,
    ) -> Result<Self, String> {
        if amount <= Decimal::ZERO {
            return Err("Amount must be positive".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            transaction_type,
            amount,
            currency,
            description,
            account_id,
            status: TransactionStatus::Pending,
            created_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub id: Uuid,
    pub loan_type: LoanType,
    pub principal_amount: Decimal,
    pub currency: Currency,
    pub interest_rate: Decimal,
    pub term_months: u32,
    pub status: LoanStatus,
    pub created_at: DateTime<Utc>,
}

impl Loan {
    pub fn new(
        loan_type: LoanType,
        principal_amount: Decimal,
        currency: Currency,
        interest_rate: Decimal,
        term_months: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            loan_type,
            principal_amount,
            currency,
            interest_rate,
            term_months,
            status: LoanStatus::Active,
            created_at: Utc::now(),
        }
    }
}

// Form structs
#[derive(Debug, Clone)]
pub struct AccountForm {
    pub name: String,
    pub account_type: Option<AccountType>,
    pub currency: Option<Currency>,
    pub initial_balance: String,
    pub errors: Vec<String>,
}

impl Default for AccountForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            account_type: None,
            currency: None,
            initial_balance: String::new(),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionForm {
    pub transaction_type: Option<TransactionType>,
    pub amount: String,
    pub currency: Option<Currency>,
    pub description: String,
    pub account_id: String,
    pub errors: Vec<String>,
}

impl Default for TransactionForm {
    fn default() -> Self {
        Self {
            transaction_type: None,
            amount: String::new(),
            currency: None,
            description: String::new(),
            account_id: String::new(),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoanForm {
    pub loan_type: Option<LoanType>,
    pub principal: String,
    pub interest_rate: String,
    pub term_months: String,
    pub errors: Vec<String>,
}

impl Default for LoanForm {
    fn default() -> Self {
        Self {
            loan_type: None,
            principal: String::new(),
            interest_rate: String::new(),
            term_months: String::new(),
            errors: Vec::new(),
        }
    }
}

// Screen enum
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Splash,
    Dashboard,
    Accounts,
    Transactions,
    Loans,
    Reports,
}

// Utility functions
pub fn format_money(amount: Decimal, _currency: Currency) -> String {
    format!("{:.2}", amount)
}