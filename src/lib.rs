//! # FerrumFinance - Comprehensive Financial Management System
//!
//! FerrumFinance is a Rust-based application for managing loans, cash flow, and financial transactions.
//! Built with the Iced GUI framework, it provides a modern interface for multi-currency financial management.
//!
//! ## Features
//!
//! - **Multi-currency support** with real-time exchange rates
//! - **Comprehensive account management** following standard accounting principles
//! - **Advanced transaction processing** with categorization and analytics
//! - **Loan management** with payment schedules and interest calculations
//! - **Historical data tracking** with audit trails
//! - **Financial analytics** with trend analysis and reporting
//!
//! ## Core Modules
//!
//! - [`models`] - Core data models for financial entities
//! - [`storage`] - SQLite database integration with repository pattern
//! - [`assets`] - Static assets and resources
//!
//! ## Storage Layer Features
//!
//! - **Repository Pattern** - Clean separation between business logic and data access
//! - **Comprehensive CRUD Operations** - Full create, read, update, delete functionality
//! - **Advanced Querying** - Complex filters, sorting, and pagination
//! - **Transaction Safety** - ACID compliance with automatic rollback
//! - **Multi-currency Support** - Native handling of different currencies
//! - **Audit Trail** - Complete change tracking for compliance
//! - **Performance Optimization** - Proper indexing and query optimization

pub use rust_decimal::Decimal;

// Comprehensive financial models with full implementation
pub mod models;

// Asset management for logos and images
pub mod assets;

// Database and storage layer
pub mod storage;

// Re-export commonly used types from models
pub use models::{
    // Core types
    Currency, CurrencyError,
    Account, AccountType, AccountStatus, AccountError,
    Transaction, TransactionType, TransactionStatus, TransactionError,
    Loan, LoanType, LoanStatus, LoanError,
    ExchangeRate, CurrencyConverter, ExchangeRateError,

    // Analytics types
    TransactionAggregator, TransactionSummary, TimePeriod,
    TransactionHistory, HistoryOperation,

    // Common utilities
    ModelResult, validate_monetary_amount, format_money,

    // Re-exported external types
    DateTime, Utc, Uuid,
};

// Re-export storage types for easy access
pub use storage::{
    // Core storage types
    Database, DatabaseManager, DatabaseConfig,
    StorageError, StorageResult,

    // Repository types
    Repository, RepositoryManager,
    AccountRepository, TransactionRepository, LoanRepository, ExchangeRateRepository,
};

// Re-export asset management functions
pub use assets::{
    get_logo_handle, get_splash_logo, get_app_logo,
    LOGO_1, LOGO_2, LOGO_3, LOGO_4,
};