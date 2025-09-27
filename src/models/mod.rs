//! Models module for FerrumFinance
//!
//! This module contains all the core data models and business logic for the financial
//! management system, including currencies, accounts, transactions, loans, and analytics.

// Re-export commonly used external crates for convenience
pub use chrono::{DateTime, Utc};
pub use rust_decimal::Decimal;
pub use rust_decimal::prelude::ToPrimitive;
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;

// Core financial models
pub mod currency;
pub mod account;
pub mod transaction;
pub mod loan;
pub mod exchange_rate;

// Analytics and history modules
pub mod transaction_aggregation;
pub mod transaction_history;

// Re-export all major types for easy access
pub use currency::{Currency, CurrencyError};
pub use account::{Account, AccountType, AccountStatus, AccountError};
pub use transaction::{
    Transaction, TransactionType, TransactionStatus, TransactionPriority,
    TransactionError, RecurringPattern, RecurringFrequency,
};
pub use loan::{
    Loan, LoanType, LoanStatus, InterestRateType, PaymentFrequency,
    PaymentCalculationMethod, LoanPayment, LoanError,
};
pub use exchange_rate::{
    ExchangeRate, ExchangeRateSource, ExchangeRateError,
    CurrencyConverter, ConversionResult,
};
pub use transaction_aggregation::{
    TransactionSummary, TransactionAggregator, TimePeriod, AggregationType,
    TransactionTrend, TrendDirection, AggregationError,
};
pub use transaction_history::{
    TransactionHistory, TransactionHistoryEntry, HistoryOperation,
    HistoryFilter, SortOrder, HistoryError,
};

/// Common result type for model operations
pub type ModelResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Validates that a monetary amount is valid (non-negative for most operations)
pub fn validate_monetary_amount(amount: Decimal, allow_negative: bool) -> Result<(), String> {
    if !allow_negative && amount < Decimal::ZERO {
        return Err(format!("Amount cannot be negative: {}", amount));
    }

    // Check for reasonable bounds (avoid astronomical amounts that might indicate errors)
    let max_amount = Decimal::new(999_999_999_999i64, 2); // 9.99 trillion
    if amount > max_amount {
        return Err(format!("Amount exceeds maximum allowed: {}", amount));
    }

    Ok(())
}

/// Validates that a percentage is within valid bounds (0-100%)
pub fn validate_percentage(percentage: Decimal) -> Result<(), String> {
    if percentage < Decimal::ZERO {
        return Err(format!("Percentage cannot be negative: {}", percentage));
    }

    if percentage > Decimal::from(100) {
        return Err(format!("Percentage cannot exceed 100%: {}", percentage));
    }

    Ok(())
}

/// Validates that a date range is valid (start before end)
pub fn validate_date_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<(), String> {
    if start >= end {
        return Err(format!(
            "Start date ({}) must be before end date ({})",
            start.format("%Y-%m-%d %H:%M:%S UTC"),
            end.format("%Y-%m-%d %H:%M:%S UTC")
        ));
    }

    Ok(())
}

/// Rounds a decimal to the specified number of decimal places for currency display
pub fn round_to_currency_precision(amount: Decimal, currency: &Currency) -> Decimal {
    let decimal_places = currency.decimal_places();
    let scale = 10_i32.pow(decimal_places);
    (amount * Decimal::from(scale)).round() / Decimal::from(scale)
}

/// Formats a monetary amount for display with currency symbol
pub fn format_money(amount: Decimal, currency: &Currency) -> String {
    let rounded = round_to_currency_precision(amount, currency);
    let symbol = currency.symbol();

    if currency.decimal_places() == 0 {
        format!("{}{}", symbol, rounded.to_string().split('.').next().unwrap_or("0"))
    } else {
        format!("{}{:.2}", symbol, rounded)
    }
}

/// Calculates compound interest
///
/// # Arguments
/// * `principal` - Initial amount
/// * `rate` - Annual interest rate (as decimal, e.g., 0.05 for 5%)
/// * `compounds_per_year` - Number of compounding periods per year
/// * `years` - Number of years
pub fn calculate_compound_interest(
    principal: Decimal,
    rate: Decimal,
    compounds_per_year: u32,
    years: Decimal,
) -> Decimal {
    if rate == Decimal::ZERO {
        return principal;
    }

    let compounds = Decimal::from(compounds_per_year);
    let rate_per_period = rate / compounds;
    let total_periods = compounds * years;

    // A = P(1 + r/n)^(nt)
    let base = Decimal::ONE + rate_per_period;

    // Simple approximation for decimal exponentiation
    // For more precise calculations, a proper decimal math library would be needed
    let mut result = principal;
    let periods_int = total_periods.to_u32().unwrap_or(0);

    for _ in 0..periods_int {
        result *= base;
    }

    result
}

/// Calculates simple interest
///
/// # Arguments
/// * `principal` - Initial amount
/// * `rate` - Annual interest rate (as decimal)
/// * `time_years` - Time in years
pub fn calculate_simple_interest(
    principal: Decimal,
    rate: Decimal,
    time_years: Decimal,
) -> Decimal {
    principal * rate * time_years
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_monetary_amount() {
        // Valid positive amount
        assert!(validate_monetary_amount(Decimal::new(100, 2), false).is_ok());

        // Valid zero amount
        assert!(validate_monetary_amount(Decimal::ZERO, false).is_ok());

        // Invalid negative amount (when not allowed)
        assert!(validate_monetary_amount(Decimal::new(-100, 2), false).is_err());

        // Valid negative amount (when allowed)
        assert!(validate_monetary_amount(Decimal::new(-100, 2), true).is_ok());

        // Invalid astronomical amount
        let huge_amount = Decimal::new(999_999_999_999_999i64, 2);
        assert!(validate_monetary_amount(huge_amount, false).is_err());
    }

    #[test]
    fn test_validate_percentage() {
        // Valid percentages
        assert!(validate_percentage(Decimal::new(50, 0)).is_ok());
        assert!(validate_percentage(Decimal::ZERO).is_ok());
        assert!(validate_percentage(Decimal::new(100, 0)).is_ok());

        // Invalid percentages
        assert!(validate_percentage(Decimal::new(-10, 0)).is_err());
        assert!(validate_percentage(Decimal::new(150, 0)).is_err());
    }

    #[test]
    fn test_validate_date_range() {
        let start = Utc::now();
        let end = start + chrono::Duration::days(1);

        // Valid date range
        assert!(validate_date_range(start, end).is_ok());

        // Invalid date range (start after end)
        assert!(validate_date_range(end, start).is_err());

        // Invalid date range (start equals end)
        assert!(validate_date_range(start, start).is_err());
    }

    #[test]
    fn test_round_to_currency_precision() {
        let amount = Decimal::new(12345, 3); // 12.345

        // USD (2 decimal places)
        let rounded_usd = round_to_currency_precision(amount, &Currency::USD);
        assert_eq!(rounded_usd, Decimal::new(1234, 2)); // 12.34 (banker's rounding)

        // JPY (0 decimal places)
        let rounded_jpy = round_to_currency_precision(amount, &Currency::JPY);
        assert_eq!(rounded_jpy, Decimal::new(12, 0)); // 12
    }

    #[test]
    fn test_format_money() {
        // USD formatting
        let usd_amount = Decimal::new(12345, 2); // 123.45
        assert_eq!(format_money(usd_amount, &Currency::USD), "$123.45");

        // JPY formatting (no decimals)
        let jpy_amount = Decimal::new(12345, 0); // 12345
        assert_eq!(format_money(jpy_amount, &Currency::JPY), "¥12345");
    }

    #[test]
    fn test_simple_interest() {
        let principal = Decimal::new(1000, 0); // $1000
        let rate = Decimal::new(5, 2); // 5% = 0.05
        let time = Decimal::new(2, 0); // 2 years

        let interest = calculate_simple_interest(principal, rate, time);
        assert_eq!(interest, Decimal::new(100, 0)); // $100
    }

    #[test]
    fn test_compound_interest() {
        let principal = Decimal::new(1000, 0); // $1000
        let rate = Decimal::new(5, 2); // 5% = 0.05
        let compounds_per_year = 12; // Monthly compounding
        let years = Decimal::new(1, 0); // 1 year

        let final_amount = calculate_compound_interest(principal, rate, compounds_per_year, years);

        // Should be slightly more than simple interest due to compounding
        let simple_interest_final = principal + calculate_simple_interest(principal, rate, years);
        assert!(final_amount > simple_interest_final);
    }
}