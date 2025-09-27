//! Exchange rate module for FerrumFinance
//!
//! This module provides currency conversion functionality including exchange rate management,
//! historical rates, and currency conversion operations with proper error handling.

use crate::models::currency::Currency;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use thiserror::Error;
use uuid::Uuid;

/// Represents an exchange rate between two currencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    /// Unique identifier for this exchange rate entry
    pub id: Uuid,
    /// Base currency (what 1 unit represents)
    pub base_currency: Currency,
    /// Target currency (what the base currency converts to)
    pub target_currency: Currency,
    /// Exchange rate (how many target currency units for 1 base currency unit)
    pub rate: Decimal,
    /// When this rate is effective from
    pub effective_date: DateTime<Utc>,
    /// When this rate expires (if applicable)
    pub expiry_date: Option<DateTime<Utc>>,
    /// Source of the exchange rate data
    pub source: ExchangeRateSource,
    /// When this rate was last updated
    pub updated_at: DateTime<Utc>,
    /// Additional metadata about this rate
    pub metadata: HashMap<String, String>,
}

/// Source of exchange rate data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExchangeRateSource {
    /// Manually entered rate
    Manual,
    /// Rate from a financial API (e.g., fixer.io, exchangerate-api.com)
    Api { provider: String },
    /// Rate from a bank
    Bank { institution: String },
    /// Rate from a central bank
    CentralBank { country: String },
    /// Rate from a trading platform
    Trading { platform: String },
    /// Calculated/derived rate
    Calculated,
}

/// Error types for exchange rate operations
#[derive(Error, Debug)]
pub enum ExchangeRateError {
    #[error("Exchange rate not found for {from} to {to}")]
    RateNotFound { from: Currency, to: Currency },
    #[error("Exchange rate expired: {rate_date} (current: {current_date})")]
    RateExpired { rate_date: DateTime<Utc>, current_date: DateTime<Utc> },
    #[error("Invalid exchange rate: {rate}")]
    InvalidRate { rate: Decimal },
    #[error("Same currency conversion: {currency}")]
    SameCurrency { currency: Currency },
    #[error("Currency conversion error: {message}")]
    ConversionError { message: String },
    #[error("No path found for conversion from {from} to {to}")]
    NoConversionPath { from: Currency, to: Currency },
    #[error("Exchange rate source error: {message}")]
    SourceError { message: String },
}

/// Currency converter that manages exchange rates and performs conversions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyConverter {
    /// Map of exchange rates keyed by currency pair
    rates: HashMap<(Currency, Currency), ExchangeRate>,
    /// Base currency for the converter (rates stored relative to this)
    pub base_currency: Currency,
    /// When the converter was last updated
    pub last_updated: DateTime<Utc>,
    /// Default rate validity period (in hours)
    pub default_validity_hours: i64,
}

/// Result of a currency conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    /// Original amount being converted
    pub from_amount: Decimal,
    /// Currency being converted from
    pub from_currency: Currency,
    /// Converted amount
    pub to_amount: Decimal,
    /// Currency being converted to
    pub to_currency: Currency,
    /// Exchange rate used for conversion
    pub rate: Decimal,
    /// When the conversion was performed
    pub conversion_date: DateTime<Utc>,
    /// Exchange rate source information
    pub rate_source: ExchangeRateSource,
    /// Rate effective date
    pub rate_date: DateTime<Utc>,
}

impl ExchangeRate {
    /// Creates a new exchange rate
    pub fn new(
        base_currency: Currency,
        target_currency: Currency,
        rate: Decimal,
        source: ExchangeRateSource,
    ) -> Result<Self, ExchangeRateError> {
        if base_currency == target_currency {
            return Err(ExchangeRateError::SameCurrency { currency: base_currency });
        }

        if rate <= Decimal::ZERO {
            return Err(ExchangeRateError::InvalidRate { rate });
        }

        let now = Utc::now();

        Ok(ExchangeRate {
            id: Uuid::new_v4(),
            base_currency,
            target_currency,
            rate,
            effective_date: now,
            expiry_date: None,
            source,
            updated_at: now,
            metadata: HashMap::new(),
        })
    }

    /// Creates a new exchange rate with expiry
    pub fn new_with_expiry(
        base_currency: Currency,
        target_currency: Currency,
        rate: Decimal,
        source: ExchangeRateSource,
        expiry_date: DateTime<Utc>,
    ) -> Result<Self, ExchangeRateError> {
        let mut exchange_rate = Self::new(base_currency, target_currency, rate, source)?;
        exchange_rate.expiry_date = Some(expiry_date);
        Ok(exchange_rate)
    }

    /// Checks if this exchange rate is currently valid
    pub fn is_valid(&self, at_time: Option<DateTime<Utc>>) -> bool {
        let check_time = at_time.unwrap_or_else(Utc::now);

        // Check if rate is effective
        if check_time < self.effective_date {
            return false;
        }

        // Check if rate has expired
        if let Some(expiry) = self.expiry_date {
            if check_time > expiry {
                return false;
            }
        }

        true
    }

    /// Gets the inverse rate (swaps base and target currencies)
    pub fn inverse(&self) -> Result<ExchangeRate, ExchangeRateError> {
        if self.rate == Decimal::ZERO {
            return Err(ExchangeRateError::InvalidRate { rate: self.rate });
        }

        let inverse_rate = Decimal::ONE / self.rate;

        Ok(ExchangeRate {
            id: Uuid::new_v4(),
            base_currency: self.target_currency.clone(),
            target_currency: self.base_currency.clone(),
            rate: inverse_rate,
            effective_date: self.effective_date,
            expiry_date: self.expiry_date,
            source: self.source.clone(),
            updated_at: self.updated_at,
            metadata: self.metadata.clone(),
        })
    }

    /// Adds metadata to the exchange rate
    pub fn add_metadata<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key.into(), value.into());
        self.updated_at = Utc::now();
    }
}

impl CurrencyConverter {
    /// Creates a new currency converter with a base currency
    pub fn new(base_currency: Currency) -> Self {
        CurrencyConverter {
            rates: HashMap::new(),
            base_currency,
            last_updated: Utc::now(),
            default_validity_hours: 24, // Rates valid for 24 hours by default
        }
    }

    /// Adds an exchange rate to the converter
    pub fn add_rate(&mut self, rate: ExchangeRate) -> Result<(), ExchangeRateError> {
        let key = (rate.base_currency.clone(), rate.target_currency.clone());
        let inverse_key = (rate.target_currency.clone(), rate.base_currency.clone());

        // Store the main rate
        self.rates.insert(key.clone(), rate);

        // Also add the inverse rate for convenience
        if !self.rates.contains_key(&inverse_key) {
            if let Ok(inverse_rate) = self.rates[&key].inverse() {
                self.rates.insert(inverse_key, inverse_rate);
            }
        }

        self.last_updated = Utc::now();
        Ok(())
    }

    /// Sets an exchange rate directly
    pub fn set_rate(
        &mut self,
        from: Currency,
        to: Currency,
        rate: Decimal,
        source: ExchangeRateSource,
    ) -> Result<(), ExchangeRateError> {
        let exchange_rate = ExchangeRate::new(from, to, rate, source)?;
        self.add_rate(exchange_rate)?;
        Ok(())
    }

    /// Gets the current exchange rate between two currencies
    pub fn get_rate(&self, from: &Currency, to: &Currency) -> Result<&ExchangeRate, ExchangeRateError> {
        // Same currency
        if from == to {
            return Err(ExchangeRateError::SameCurrency { currency: from.clone() });
        }

        let key = (from.clone(), to.clone());

        if let Some(rate) = self.rates.get(&key) {
            if rate.is_valid(None) {
                return Ok(rate);
            } else {
                return Err(ExchangeRateError::RateExpired {
                    rate_date: rate.effective_date,
                    current_date: Utc::now(),
                });
            }
        }

        // Try to find indirect conversion through base currency
        if from != &self.base_currency && to != &self.base_currency {
            // Convert from -> base -> to
            let from_to_base_key = (from.clone(), self.base_currency.clone());
            let base_to_to_key = (self.base_currency.clone(), to.clone());

            if self.rates.contains_key(&from_to_base_key) && self.rates.contains_key(&base_to_to_key) {
                // We could calculate the cross rate here, but for now just indicate path exists
                return Err(ExchangeRateError::NoConversionPath {
                    from: from.clone(),
                    to: to.clone(),
                });
            }
        }

        Err(ExchangeRateError::RateNotFound {
            from: from.clone(),
            to: to.clone(),
        })
    }

    /// Converts an amount from one currency to another
    pub fn convert(
        &self,
        amount: Decimal,
        from: &Currency,
        to: &Currency,
    ) -> Result<ConversionResult, ExchangeRateError> {
        // Same currency, no conversion needed
        if from == to {
            return Ok(ConversionResult {
                from_amount: amount,
                from_currency: from.clone(),
                to_amount: amount,
                to_currency: to.clone(),
                rate: Decimal::ONE,
                conversion_date: Utc::now(),
                rate_source: ExchangeRateSource::Manual,
                rate_date: Utc::now(),
            });
        }

        let exchange_rate = self.get_rate(from, to)?;
        let converted_amount = amount * exchange_rate.rate;

        Ok(ConversionResult {
            from_amount: amount,
            from_currency: from.clone(),
            to_amount: converted_amount,
            to_currency: to.clone(),
            rate: exchange_rate.rate,
            conversion_date: Utc::now(),
            rate_source: exchange_rate.source.clone(),
            rate_date: exchange_rate.effective_date,
        })
    }

    /// Converts with cross-currency calculation through base currency
    pub fn convert_with_cross_rate(
        &self,
        amount: Decimal,
        from: &Currency,
        to: &Currency,
    ) -> Result<ConversionResult, ExchangeRateError> {
        // Same currency
        if from == to {
            return self.convert(amount, from, to);
        }

        // Direct conversion exists
        if self.rates.contains_key(&(from.clone(), to.clone())) {
            return self.convert(amount, from, to);
        }

        // Try cross conversion through base currency
        if from != &self.base_currency && to != &self.base_currency {
            // Convert from -> base
            let intermediate = self.convert(amount, from, &self.base_currency)?;
            // Convert base -> to
            let final_result = self.convert(intermediate.to_amount, &self.base_currency, to)?;

            // Calculate the effective cross rate
            let cross_rate = final_result.to_amount / amount;

            return Ok(ConversionResult {
                from_amount: amount,
                from_currency: from.clone(),
                to_amount: final_result.to_amount,
                to_currency: to.clone(),
                rate: cross_rate,
                conversion_date: Utc::now(),
                rate_source: ExchangeRateSource::Calculated,
                rate_date: Utc::now(),
            });
        }

        Err(ExchangeRateError::NoConversionPath {
            from: from.clone(),
            to: to.clone(),
        })
    }

    /// Gets all available currency pairs
    pub fn get_available_pairs(&self) -> Vec<(Currency, Currency)> {
        self.rates.keys().cloned().collect()
    }

    /// Removes expired exchange rates
    pub fn cleanup_expired_rates(&mut self) -> usize {
        let now = Utc::now();
        let initial_count = self.rates.len();

        self.rates.retain(|_, rate| rate.is_valid(Some(now)));

        self.last_updated = now;
        initial_count - self.rates.len()
    }

    /// Gets exchange rates for a specific currency
    pub fn get_rates_for_currency(&self, currency: &Currency) -> Vec<&ExchangeRate> {
        self.rates
            .values()
            .filter(|rate| &rate.base_currency == currency || &rate.target_currency == currency)
            .collect()
    }

    /// Updates the default validity period for new rates
    pub fn set_default_validity_hours(&mut self, hours: i64) {
        self.default_validity_hours = hours;
    }
}

impl Display for ExchangeRateSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ExchangeRateSource::Manual => write!(f, "Manual"),
            ExchangeRateSource::Api { provider } => write!(f, "API ({})", provider),
            ExchangeRateSource::Bank { institution } => write!(f, "Bank ({})", institution),
            ExchangeRateSource::CentralBank { country } => write!(f, "Central Bank ({})", country),
            ExchangeRateSource::Trading { platform } => write!(f, "Trading ({})", platform),
            ExchangeRateSource::Calculated => write!(f, "Calculated"),
        }
    }
}

impl Display for ExchangeRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "1 {} = {} {} ({})",
            self.base_currency.code(),
            self.rate,
            self.target_currency.code(),
            self.source
        )
    }
}

impl Display for ConversionResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {} = {} {} (rate: {})",
            self.from_amount,
            self.from_currency.code(),
            self.to_amount,
            self.to_currency.code(),
            self.rate
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_exchange_rate_creation() {
        let rate = ExchangeRate::new(
            Currency::USD,
            Currency::EUR,
            Decimal::new(85, 2), // 0.85
            ExchangeRateSource::Manual,
        );

        assert!(rate.is_ok());
        let rate = rate.unwrap();
        assert_eq!(rate.base_currency, Currency::USD);
        assert_eq!(rate.target_currency, Currency::EUR);
        assert_eq!(rate.rate, Decimal::new(85, 2));
    }

    #[test]
    fn test_same_currency_error() {
        let rate = ExchangeRate::new(
            Currency::USD,
            Currency::USD,
            Decimal::new(1, 0),
            ExchangeRateSource::Manual,
        );

        assert!(rate.is_err());
    }

    #[test]
    fn test_invalid_rate() {
        let rate = ExchangeRate::new(
            Currency::USD,
            Currency::EUR,
            Decimal::ZERO,
            ExchangeRateSource::Manual,
        );

        assert!(rate.is_err());
    }

    #[test]
    fn test_inverse_rate() {
        let rate = ExchangeRate::new(
            Currency::USD,
            Currency::EUR,
            Decimal::new(85, 2), // 0.85
            ExchangeRateSource::Manual,
        ).unwrap();

        let inverse = rate.inverse();
        assert!(inverse.is_ok());

        let inverse = inverse.unwrap();
        assert_eq!(inverse.base_currency, Currency::EUR);
        assert_eq!(inverse.target_currency, Currency::USD);
        // 1/0.85 ≈ 1.176
        assert!(inverse.rate > Decimal::new(117, 2));
        assert!(inverse.rate < Decimal::new(118, 2));
    }

    #[test]
    fn test_currency_converter() {
        let mut converter = CurrencyConverter::new(Currency::USD);

        // Add USD to EUR rate
        let result = converter.set_rate(
            Currency::USD,
            Currency::EUR,
            Decimal::new(85, 2),
            ExchangeRateSource::Manual,
        );
        assert!(result.is_ok());

        // Test conversion
        let conversion = converter.convert(
            Decimal::new(100, 0),
            &Currency::USD,
            &Currency::EUR,
        );

        assert!(conversion.is_ok());
        let result = conversion.unwrap();
        assert_eq!(result.from_amount, Decimal::new(100, 0));
        assert_eq!(result.to_amount, Decimal::new(85, 0)); // 100 * 0.85
    }

    #[test]
    fn test_same_currency_conversion() {
        let converter = CurrencyConverter::new(Currency::USD);

        let conversion = converter.convert(
            Decimal::new(100, 0),
            &Currency::USD,
            &Currency::USD,
        );

        assert!(conversion.is_ok());
        let result = conversion.unwrap();
        assert_eq!(result.from_amount, result.to_amount);
        assert_eq!(result.rate, Decimal::ONE);
    }

    #[test]
    fn test_rate_not_found() {
        let converter = CurrencyConverter::new(Currency::USD);

        let conversion = converter.convert(
            Decimal::new(100, 0),
            &Currency::USD,
            &Currency::EUR,
        );

        assert!(conversion.is_err());
    }

    #[test]
    fn test_available_pairs() {
        let mut converter = CurrencyConverter::new(Currency::USD);

        converter.set_rate(
            Currency::USD,
            Currency::EUR,
            Decimal::new(85, 2),
            ExchangeRateSource::Manual,
        ).unwrap();

        let pairs = converter.get_available_pairs();
        assert!(pairs.len() >= 2); // Should have both directions
        assert!(pairs.contains(&(Currency::USD, Currency::EUR)));
        assert!(pairs.contains(&(Currency::EUR, Currency::USD)));
    }

    #[test]
    fn test_rate_validity() {
        let base_time = Utc::now() - Duration::hours(1); // Use past time as base
        let future = base_time + Duration::hours(2);
        let past = base_time - Duration::hours(1);

        let mut rate = ExchangeRate::new(
            Currency::USD,
            Currency::EUR,
            Decimal::new(85, 2),
            ExchangeRateSource::Manual,
        ).unwrap();

        // Manually set effective date to base time
        rate.effective_date = base_time;

        // Rate is valid at the current time (after effective date)
        assert!(rate.is_valid(None)); // Use current time

        // Set expiry in the past
        rate.expiry_date = Some(past);
        assert!(!rate.is_valid(None));

        // Set expiry in the future
        rate.expiry_date = Some(future);
        assert!(rate.is_valid(None));
    }
}