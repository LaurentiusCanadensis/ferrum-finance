//! Currency module for FerrumFinance
//!
//! This module provides currency representation and utility functions for multi-currency
//! financial operations with proper serialization and display formatting.

use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use thiserror::Error;

/// Represents various currencies supported by the financial system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    /// United States Dollar
    USD,
    /// Euro
    EUR,
    /// British Pound Sterling
    GBP,
    /// Japanese Yen
    JPY,
    /// Canadian Dollar
    CAD,
    /// Australian Dollar
    AUD,
    /// Swiss Franc
    CHF,
    /// Chinese Yuan
    CNY,
    /// Custom currency with ISO 4217 code
    Custom(String),
}

/// Error types for currency operations
#[derive(Error, Debug)]
pub enum CurrencyError {
    #[error("Invalid currency code: {0}")]
    InvalidCode(String),
    #[error("Currency conversion not available for {from} to {to}")]
    ConversionNotAvailable { from: String, to: String },
}

impl Currency {
    /// Returns the ISO 4217 currency code
    pub fn code(&self) -> &str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
            Currency::CHF => "CHF",
            Currency::CNY => "CNY",
            Currency::Custom(code) => code,
        }
    }

    /// Returns the currency symbol
    pub fn symbol(&self) -> &str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::GBP => "£",
            Currency::JPY => "¥",
            Currency::CAD => "C$",
            Currency::AUD => "A$",
            Currency::CHF => "CHF",
            Currency::CNY => "¥",
            Currency::Custom(_) => "",
        }
    }

    /// Returns the currency name
    pub fn name(&self) -> &str {
        match self {
            Currency::USD => "US Dollar",
            Currency::EUR => "Euro",
            Currency::GBP => "British Pound Sterling",
            Currency::JPY => "Japanese Yen",
            Currency::CAD => "Canadian Dollar",
            Currency::AUD => "Australian Dollar",
            Currency::CHF => "Swiss Franc",
            Currency::CNY => "Chinese Yuan",
            Currency::Custom(code) => code,
        }
    }

    /// Returns the number of decimal places typically used for this currency
    pub fn decimal_places(&self) -> u32 {
        match self {
            Currency::JPY => 0, // Japanese Yen doesn't use decimal places
            _ => 2, // Most currencies use 2 decimal places
        }
    }

    /// Returns true if this is a major trading currency
    pub fn is_major(&self) -> bool {
        matches!(
            self,
            Currency::USD | Currency::EUR | Currency::GBP | Currency::JPY | Currency::CHF
        )
    }

    /// Returns all supported major currencies
    pub fn major_currencies() -> Vec<Currency> {
        vec![
            Currency::USD,
            Currency::EUR,
            Currency::GBP,
            Currency::JPY,
            Currency::CAD,
            Currency::AUD,
            Currency::CHF,
            Currency::CNY,
        ]
    }

    /// Creates a custom currency with validation
    pub fn custom<S: Into<String>>(code: S) -> Result<Currency, CurrencyError> {
        let code = code.into();
        if code.len() != 3 {
            return Err(CurrencyError::InvalidCode(code));
        }
        if !code.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(CurrencyError::InvalidCode(code));
        }
        Ok(Currency::Custom(code))
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.code())
    }
}

impl FromStr for Currency {
    type Err = CurrencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Currency::USD),
            "EUR" => Ok(Currency::EUR),
            "GBP" => Ok(Currency::GBP),
            "JPY" => Ok(Currency::JPY),
            "CAD" => Ok(Currency::CAD),
            "AUD" => Ok(Currency::AUD),
            "CHF" => Ok(Currency::CHF),
            "CNY" => Ok(Currency::CNY),
            code if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) => {
                Ok(Currency::Custom(code.to_string()))
            }
            _ => Err(CurrencyError::InvalidCode(s.to_string())),
        }
    }
}

impl Default for Currency {
    fn default() -> Self {
        Currency::USD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_code() {
        assert_eq!(Currency::USD.code(), "USD");
        assert_eq!(Currency::EUR.code(), "EUR");
        assert_eq!(Currency::Custom("XYZ".to_string()).code(), "XYZ");
    }

    #[test]
    fn test_currency_symbol() {
        assert_eq!(Currency::USD.symbol(), "$");
        assert_eq!(Currency::EUR.symbol(), "€");
        assert_eq!(Currency::GBP.symbol(), "£");
    }

    #[test]
    fn test_currency_decimal_places() {
        assert_eq!(Currency::USD.decimal_places(), 2);
        assert_eq!(Currency::JPY.decimal_places(), 0);
    }

    #[test]
    fn test_currency_from_str() {
        assert_eq!(Currency::from_str("USD").unwrap(), Currency::USD);
        assert_eq!(Currency::from_str("usd").unwrap(), Currency::USD);
        assert!(Currency::from_str("INVALID").is_err());
    }

    #[test]
    fn test_custom_currency() {
        let custom = Currency::custom("XYZ").unwrap();
        assert_eq!(custom.code(), "XYZ");

        assert!(Currency::custom("xy").is_err()); // Too short
        assert!(Currency::custom("xyz").is_err()); // Not uppercase
    }

    #[test]
    fn test_currency_display() {
        assert_eq!(format!("{}", Currency::USD), "USD");
        assert_eq!(format!("{}", Currency::EUR), "EUR");
    }

    #[test]
    fn test_major_currencies() {
        let majors = Currency::major_currencies();
        assert!(majors.contains(&Currency::USD));
        assert!(majors.contains(&Currency::EUR));
        assert_eq!(majors.len(), 8);
    }

    #[test]
    fn test_is_major() {
        assert!(Currency::USD.is_major());
        assert!(Currency::EUR.is_major());
        assert!(!Currency::Custom("XYZ".to_string()).is_major());
    }
}