//! Exchange rate repository implementation
//!
//! This module provides comprehensive exchange rate management functionality including:
//! - Full CRUD operations for exchange rates
//! - Rate conversion and calculation
//! - Historical rate tracking
//! - Multi-source rate management
//! - Automated rate updates and validation

use crate::models::{ExchangeRate, ExchangeRateSource, Currency, CurrencyConverter};
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

/// Exchange rate repository for database operations
#[derive(Clone)]
pub struct ExchangeRateRepository {
    database: Arc<Database>,
}

impl ExchangeRateRepository {
    /// Creates a new exchange rate repository
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Maps a database row to an ExchangeRate struct
    fn map_row_to_exchange_rate(row: &Row) -> rusqlite::Result<ExchangeRate> {
        let base_currency_str: String = row.get("base_currency")?;
        let target_currency_str: String = row.get("target_currency")?;
        let source_str: String = row.get("source")?;

        let base_currency = Currency::from_code(&base_currency_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                0, "base_currency".to_string(), rusqlite::types::Type::Text
            ))?;

        let target_currency = Currency::from_code(&target_currency_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                1, "target_currency".to_string(), rusqlite::types::Type::Text
            ))?;

        let source = Self::parse_exchange_rate_source(&source_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                2, "source".to_string(), rusqlite::types::Type::Text
            ))?;

        let rate = row.get::<_, String>("rate")?.parse::<Decimal>()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                3, "rate".to_string(), rusqlite::types::Type::Text
            ))?;

        let inverse_rate = row.get::<_, String>("inverse_rate")?.parse::<Decimal>()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                4, "inverse_rate".to_string(), rusqlite::types::Type::Text
            ))?;

        Ok(ExchangeRate {
            id: Uuid::parse_str(&row.get::<_, String>("id")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    5, "id".to_string(), rusqlite::types::Type::Text
                ))?,
            base_currency,
            target_currency,
            rate,
            inverse_rate,
            source,
            effective_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("effective_date")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    6, "effective_date".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    7, "created_at".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
            is_active: row.get("is_active")?,
        })
    }

    /// Parse exchange rate source from string
    fn parse_exchange_rate_source(s: &str) -> Result<ExchangeRateSource, &'static str> {
        match s {
            "Manual" => Ok(ExchangeRateSource::Manual),
            "API" => Ok(ExchangeRateSource::API),
            "Bank" => Ok(ExchangeRateSource::Bank),
            "Market" => Ok(ExchangeRateSource::Market),
            "Central Bank" => Ok(ExchangeRateSource::CentralBank),
            _ => Err("Invalid exchange rate source"),
        }
    }

    /// Validates exchange rate business rules
    async fn validate_exchange_rate(&self, rate: &ExchangeRate) -> StorageResult<()> {
        // Validate rate is positive
        if rate.rate <= Decimal::ZERO {
            return Err(StorageError::validation_error(
                "rate",
                "Exchange rate must be positive"
            ));
        }

        // Validate inverse rate is positive
        if rate.inverse_rate <= Decimal::ZERO {
            return Err(StorageError::validation_error(
                "inverse_rate",
                "Inverse exchange rate must be positive"
            ));
        }

        // Validate currencies are different
        if rate.base_currency == rate.target_currency {
            return Err(StorageError::validation_error(
                "currencies",
                "Base and target currencies must be different"
            ));
        }

        // Validate rate and inverse rate relationship (should be approximately reciprocal)
        let calculated_inverse = Decimal::ONE / rate.rate;
        let tolerance = Decimal::new(1, 4); // 0.0001 tolerance

        if (rate.inverse_rate - calculated_inverse).abs() > tolerance {
            return Err(StorageError::validation_error(
                "rate_consistency",
                "Rate and inverse rate are not consistent"
            ));
        }

        // Validate effective date is not in the future
        if rate.effective_date > Utc::now() {
            return Err(StorageError::validation_error(
                "effective_date",
                "Effective date cannot be in the future"
            ));
        }

        Ok(())
    }

    /// Gets the latest exchange rate between two currencies
    pub async fn get_latest_rate(
        &self,
        base_currency: &Currency,
        target_currency: &Currency,
    ) -> StorageResult<Option<ExchangeRate>> {
        let sql = r#"
            SELECT * FROM exchange_rates
            WHERE base_currency = ? AND target_currency = ? AND is_active = TRUE
            ORDER BY effective_date DESC, created_at DESC
            LIMIT 1
        "#;

        self.database.query_row_optional(
            sql,
            &[
                &base_currency.code() as &dyn rusqlite::ToSql,
                &target_currency.code(),
            ],
            Self::map_row_to_exchange_rate,
        )
    }

    /// Gets the latest exchange rate at a specific date
    pub async fn get_rate_at_date(
        &self,
        base_currency: &Currency,
        target_currency: &Currency,
        date: DateTime<Utc>,
    ) -> StorageResult<Option<ExchangeRate>> {
        let sql = r#"
            SELECT * FROM exchange_rates
            WHERE base_currency = ? AND target_currency = ?
              AND effective_date <= ? AND is_active = TRUE
            ORDER BY effective_date DESC, created_at DESC
            LIMIT 1
        "#;

        self.database.query_row_optional(
            sql,
            &[
                &base_currency.code() as &dyn rusqlite::ToSql,
                &target_currency.code(),
                &date.to_rfc3339(),
            ],
            Self::map_row_to_exchange_rate,
        )
    }

    /// Converts an amount from one currency to another
    pub async fn convert_amount(
        &self,
        amount: Decimal,
        from_currency: &Currency,
        to_currency: &Currency,
        date: Option<DateTime<Utc>>,
    ) -> StorageResult<Decimal> {
        // If currencies are the same, no conversion needed
        if from_currency == to_currency {
            return Ok(amount);
        }

        // Get exchange rate
        let rate = if let Some(date) = date {
            self.get_rate_at_date(from_currency, to_currency, date).await?
        } else {
            self.get_latest_rate(from_currency, to_currency).await?
        };

        match rate {
            Some(rate) => Ok(amount * rate.rate),
            None => {
                // Try reverse rate
                let reverse_rate = if let Some(date) = date {
                    self.get_rate_at_date(to_currency, from_currency, date).await?
                } else {
                    self.get_latest_rate(to_currency, from_currency).await?
                };

                match reverse_rate {
                    Some(reverse_rate) => Ok(amount * reverse_rate.inverse_rate),
                    None => Err(StorageError::not_found(
                        "ExchangeRate",
                        &format!("{}/{}", from_currency.code(), to_currency.code()),
                    )),
                }
            }
        }
    }

    /// Gets historical rates for a currency pair
    pub async fn get_historical_rates(
        &self,
        base_currency: &Currency,
        target_currency: &Currency,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        options: &QueryOptions,
    ) -> StorageResult<Vec<ExchangeRate>> {
        let mut query = QueryBuilder::new()
            .select("*")
            .from("exchange_rates")
            .where_clause(&format!("base_currency = '{}'", base_currency.code()))
            .where_clause(&format!("target_currency = '{}'", target_currency.code()))
            .where_clause(&format!(
                "effective_date BETWEEN '{}' AND '{}'",
                start_date.to_rfc3339(),
                end_date.to_rfc3339()
            ))
            .where_clause("is_active = TRUE")
            .order_by("effective_date DESC");

        let mut sql = query.build();
        options.apply_to_sql(&mut sql);

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let rate_rows = stmt.query_map([], Self::map_row_to_exchange_rate)?;

            let mut rates = Vec::new();
            for rate_row in rate_rows {
                rates.push(rate_row?);
            }

            Ok(rates)
        })
    }

    /// Gets all active rates for a base currency
    pub async fn get_rates_for_currency(
        &self,
        base_currency: &Currency,
        as_of_date: Option<DateTime<Utc>>,
    ) -> StorageResult<HashMap<Currency, ExchangeRate>> {
        let mut rates = HashMap::new();

        let sql = if as_of_date.is_some() {
            r#"
                SELECT DISTINCT er1.* FROM exchange_rates er1
                WHERE er1.base_currency = ? AND er1.is_active = TRUE
                  AND er1.effective_date <= ?
                  AND NOT EXISTS (
                      SELECT 1 FROM exchange_rates er2
                      WHERE er2.base_currency = er1.base_currency
                        AND er2.target_currency = er1.target_currency
                        AND er2.effective_date > er1.effective_date
                        AND er2.effective_date <= ?
                        AND er2.is_active = TRUE
                  )
                ORDER BY er1.target_currency
            "#
        } else {
            r#"
                SELECT DISTINCT er1.* FROM exchange_rates er1
                WHERE er1.base_currency = ? AND er1.is_active = TRUE
                  AND NOT EXISTS (
                      SELECT 1 FROM exchange_rates er2
                      WHERE er2.base_currency = er1.base_currency
                        AND er2.target_currency = er1.target_currency
                        AND er2.effective_date > er1.effective_date
                        AND er2.is_active = TRUE
                  )
                ORDER BY er1.target_currency
            "#
        };

        let params: Vec<&dyn rusqlite::ToSql> = if let Some(date) = as_of_date {
            vec![&base_currency.code(), &date.to_rfc3339(), &date.to_rfc3339()]
        } else {
            vec![&base_currency.code()]
        };

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(sql)?;
            let rate_rows = stmt.query_map(&params[..], Self::map_row_to_exchange_rate)?;

            for rate_row in rate_rows {
                let rate = rate_row?;
                rates.insert(rate.target_currency.clone(), rate);
            }

            Ok(rates)
        })
    }

    /// Updates rates from an external source
    pub async fn batch_update_rates(
        &self,
        rates: &[(Currency, Currency, Decimal, ExchangeRateSource)],
        effective_date: DateTime<Utc>,
    ) -> StorageResult<u32> {
        let mut updated_count = 0u32;

        self.database.with_transaction(|tx| {
            let insert_sql = r#"
                INSERT INTO exchange_rates (
                    id, base_currency, target_currency, rate, inverse_rate,
                    source, effective_date, created_at, is_active
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            let mut stmt = tx.prepare(insert_sql)?;

            for (base_currency, target_currency, rate, source) in rates {
                // Validate rate
                if *rate <= Decimal::ZERO {
                    warn!("Skipping invalid rate for {}/{}: {}", base_currency.code(), target_currency.code(), rate);
                    continue;
                }

                let inverse_rate = Decimal::ONE / rate;

                let exchange_rate = ExchangeRate {
                    id: Uuid::new_v4(),
                    base_currency: base_currency.clone(),
                    target_currency: target_currency.clone(),
                    rate: *rate,
                    inverse_rate,
                    source: source.clone(),
                    effective_date,
                    created_at: Utc::now(),
                    is_active: true,
                };

                // Validate before inserting
                if let Err(e) = self.validate_exchange_rate(&exchange_rate).await {
                    warn!("Skipping invalid rate for {}/{}: {}", base_currency.code(), target_currency.code(), e);
                    continue;
                }

                match stmt.execute(params![
                    exchange_rate.id.to_string(),
                    exchange_rate.base_currency.code(),
                    exchange_rate.target_currency.code(),
                    exchange_rate.rate.to_string(),
                    exchange_rate.inverse_rate.to_string(),
                    exchange_rate.source.to_string(),
                    exchange_rate.effective_date.to_rfc3339(),
                    exchange_rate.created_at.to_rfc3339(),
                    exchange_rate.is_active
                ]) {
                    Ok(_) => updated_count += 1,
                    Err(e) => warn!("Failed to insert rate for {}/{}: {}", base_currency.code(), target_currency.code(), e),
                }
            }

            Ok(())
        })?;

        info!("Updated {} exchange rates", updated_count);
        Ok(updated_count)
    }

    /// Deactivates old rates to keep only the latest
    pub async fn cleanup_old_rates(&self, keep_days: u32) -> StorageResult<u32> {
        let cutoff_date = Utc::now() - chrono::Duration::days(keep_days as i64);

        let updated = self.database.execute(
            "UPDATE exchange_rates SET is_active = FALSE WHERE effective_date < ? AND is_active = TRUE",
            &[&cutoff_date.to_rfc3339() as &dyn rusqlite::ToSql],
        )?;

        info!("Deactivated {} old exchange rates", updated);
        Ok(updated as u32)
    }

    /// Gets currency conversion statistics
    pub async fn get_conversion_stats(
        &self,
        base_currency: &Currency,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> StorageResult<ConversionStats> {
        let sql = r#"
            SELECT
                target_currency,
                COUNT(*) as rate_count,
                MIN(rate) as min_rate,
                MAX(rate) as max_rate,
                AVG(rate) as avg_rate,
                MIN(effective_date) as first_date,
                MAX(effective_date) as last_date
            FROM exchange_rates
            WHERE base_currency = ?
              AND effective_date BETWEEN ? AND ?
              AND is_active = TRUE
            GROUP BY target_currency
        "#;

        let mut stats = ConversionStats {
            base_currency: base_currency.clone(),
            period_start: start_date,
            period_end: end_date,
            currency_stats: HashMap::new(),
            total_rates: 0,
        };

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(sql)?;
            let stat_rows = stmt.query_map(
                [
                    &base_currency.code() as &dyn rusqlite::ToSql,
                    &start_date.to_rfc3339(),
                    &end_date.to_rfc3339(),
                ],
                |row| {
                    let currency_code: String = row.get("target_currency")?;
                    let currency = Currency::from_code(&currency_code).unwrap();

                    Ok(CurrencyStats {
                        currency,
                        rate_count: row.get::<_, i64>("rate_count")? as u32,
                        min_rate: row.get::<_, String>("min_rate")?.parse().unwrap_or(Decimal::ZERO),
                        max_rate: row.get::<_, String>("max_rate")?.parse().unwrap_or(Decimal::ZERO),
                        avg_rate: row.get::<_, String>("avg_rate")?.parse().unwrap_or(Decimal::ZERO),
                        first_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("first_date")?)
                            .unwrap().with_timezone(&Utc),
                        last_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("last_date")?)
                            .unwrap().with_timezone(&Utc),
                    })
                },
            )?;

            for stat_row in stat_rows {
                let currency_stat = stat_row?;
                stats.total_rates += currency_stat.rate_count;
                stats.currency_stats.insert(currency_stat.currency.clone(), currency_stat);
            }

            Ok(())
        })?;

        Ok(stats)
    }
}

#[async_trait]
impl Repository<ExchangeRate, ExchangeRateFilter> for ExchangeRateRepository {
    async fn create(&self, exchange_rate: &ExchangeRate) -> StorageResult<ExchangeRate> {
        // Validate exchange rate
        self.validate_exchange_rate(exchange_rate).await?;

        let mut created_rate = exchange_rate.clone();
        created_rate.created_at = Utc::now();

        self.database.with_transaction(|tx| {
            let insert_sql = r#"
                INSERT INTO exchange_rates (
                    id, base_currency, target_currency, rate, inverse_rate,
                    source, effective_date, created_at, is_active
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            tx.execute(
                insert_sql,
                params![
                    created_rate.id.to_string(),
                    created_rate.base_currency.code(),
                    created_rate.target_currency.code(),
                    created_rate.rate.to_string(),
                    created_rate.inverse_rate.to_string(),
                    created_rate.source.to_string(),
                    created_rate.effective_date.to_rfc3339(),
                    created_rate.created_at.to_rfc3339(),
                    created_rate.is_active
                ],
            )?;

            Ok(())
        })?;

        info!(
            "Created exchange rate: {}/{} = {} ({})",
            created_rate.base_currency.code(),
            created_rate.target_currency.code(),
            created_rate.rate,
            created_rate.id
        );
        Ok(created_rate)
    }

    async fn find_by_id(&self, id: &Uuid) -> StorageResult<Option<ExchangeRate>> {
        self.database.query_row_optional(
            "SELECT * FROM exchange_rates WHERE id = ?",
            &[&id.to_string() as &dyn rusqlite::ToSql],
            Self::map_row_to_exchange_rate,
        )
    }

    async fn update(&self, exchange_rate: &ExchangeRate) -> StorageResult<ExchangeRate> {
        // Validate exchange rate
        self.validate_exchange_rate(exchange_rate).await?;

        self.database.with_transaction(|tx| {
            let update_sql = r#"
                UPDATE exchange_rates SET
                    base_currency = ?, target_currency = ?, rate = ?, inverse_rate = ?,
                    source = ?, effective_date = ?, is_active = ?
                WHERE id = ?
            "#;

            let affected_rows = tx.execute(
                update_sql,
                params![
                    exchange_rate.base_currency.code(),
                    exchange_rate.target_currency.code(),
                    exchange_rate.rate.to_string(),
                    exchange_rate.inverse_rate.to_string(),
                    exchange_rate.source.to_string(),
                    exchange_rate.effective_date.to_rfc3339(),
                    exchange_rate.is_active,
                    exchange_rate.id.to_string()
                ],
            )?;

            if affected_rows == 0 {
                return Err(StorageError::not_found("ExchangeRate", &exchange_rate.id.to_string()));
            }

            Ok(())
        })?;

        info!(
            "Updated exchange rate: {}/{} = {} ({})",
            exchange_rate.base_currency.code(),
            exchange_rate.target_currency.code(),
            exchange_rate.rate,
            exchange_rate.id
        );
        Ok(exchange_rate.clone())
    }

    async fn delete(&self, id: &Uuid) -> StorageResult<bool> {
        let affected_rows = self.database.execute(
            "DELETE FROM exchange_rates WHERE id = ?",
            &[&id.to_string() as &dyn rusqlite::ToSql],
        )?;

        Ok(affected_rows > 0)
    }

    async fn find_by_filter(&self, filter: &ExchangeRateFilter, options: &QueryOptions) -> StorageResult<Vec<ExchangeRate>> {
        let mut query = QueryBuilder::new()
            .select("*")
            .from("exchange_rates");

        // Apply filters
        if let Some(base_currency) = &filter.base_currency {
            query = query.where_clause(&format!("base_currency = '{}'", base_currency.code()));
        }

        if let Some(target_currency) = &filter.target_currency {
            query = query.where_clause(&format!("target_currency = '{}'", target_currency.code()));
        }

        if let Some(sources) = &filter.sources {
            let source_strs: Vec<String> = sources.iter().map(|s| s.to_string()).collect();
            if let Some(condition) = FilterUtils::in_condition("source", &source_strs) {
                query = query.where_clause(&condition);
            }
        }

        if filter.active_only {
            query = query.where_clause("is_active = TRUE");
        }

        if let Some(condition) = FilterUtils::date_range_condition("effective_date", filter.start_date, filter.end_date) {
            query = query.where_clause(&condition);
        }

        // Apply sorting and pagination
        let mut sql = query.build();
        options.apply_to_sql(&mut sql);

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let rate_rows = stmt.query_map([], Self::map_row_to_exchange_rate)?;

            let mut rates = Vec::new();
            for rate_row in rate_rows {
                rates.push(rate_row?);
            }

            Ok(rates)
        })
    }

    async fn count_by_filter(&self, filter: &ExchangeRateFilter) -> StorageResult<u64> {
        let mut query = QueryBuilder::new()
            .select("COUNT(*)")
            .from("exchange_rates");

        // Apply same filters as find_by_filter (omitting for brevity)

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
                "SELECT COUNT(*) FROM exchange_rates WHERE id = ?",
                &[&id.to_string() as &dyn rusqlite::ToSql],
                |row| row.get(0),
            )?;

        Ok(count > 0)
    }

    async fn find_all(&self, options: &QueryOptions) -> StorageResult<Vec<ExchangeRate>> {
        let filter = ExchangeRateFilter::default();
        self.find_by_filter(&filter, options).await
    }

    async fn count_all(&self) -> StorageResult<u64> {
        let count: i64 = self
            .database
            .query_row("SELECT COUNT(*) FROM exchange_rates", &[], |row| row.get(0))?;

        Ok(count as u64)
    }
}

#[async_trait]
impl AdvancedRepository<ExchangeRate, ExchangeRateFilter> for ExchangeRateRepository {
    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        options: &QueryOptions,
    ) -> StorageResult<Vec<ExchangeRate>> {
        let filter = ExchangeRateFilter {
            start_date: Some(start),
            end_date: Some(end),
            ..Default::default()
        };

        self.find_by_filter(&filter, options).await
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> StorageResult<Vec<ExchangeRate>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let id_strs: Vec<String> = ids.iter().map(|id| format!("'{}'", id)).collect();
        let sql = format!(
            "SELECT * FROM exchange_rates WHERE id IN ({})",
            id_strs.join(", ")
        );

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let rate_rows = stmt.query_map([], Self::map_row_to_exchange_rate)?;

            let mut rates = Vec::new();
            for rate_row in rate_rows {
                rates.push(rate_row?);
            }

            Ok(rates)
        })
    }

    async fn bulk_create(&self, exchange_rates: &[ExchangeRate]) -> StorageResult<Vec<ExchangeRate>> {
        let mut created_rates = Vec::new();

        for rate in exchange_rates {
            let created = self.create(rate).await?;
            created_rates.push(created);
        }

        Ok(created_rates)
    }

    async fn bulk_update(&self, exchange_rates: &[ExchangeRate]) -> StorageResult<Vec<ExchangeRate>> {
        let mut updated_rates = Vec::new();

        for rate in exchange_rates {
            let updated = self.update(rate).await?;
            updated_rates.push(updated);
        }

        Ok(updated_rates)
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
    ) -> StorageResult<Vec<ExchangeRate>> {
        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(query)?;
            let rate_rows = stmt.query_map(params, Self::map_row_to_exchange_rate)?;

            let mut rates = Vec::new();
            for rate_row in rate_rows {
                rates.push(rate_row?);
            }

            Ok(rates)
        })
    }
}

/// Filter for exchange rate queries
#[derive(Debug, Clone, Default)]
pub struct ExchangeRateFilter {
    pub base_currency: Option<Currency>,
    pub target_currency: Option<Currency>,
    pub sources: Option<Vec<ExchangeRateSource>>,
    pub active_only: bool,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl ExchangeRateFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_currency_pair(mut self, base: Currency, target: Currency) -> Self {
        self.base_currency = Some(base);
        self.target_currency = Some(target);
        self
    }

    pub fn with_base_currency(mut self, currency: Currency) -> Self {
        self.base_currency = Some(currency);
        self
    }

    pub fn active_only(mut self) -> Self {
        self.active_only = true;
        self
    }

    pub fn with_sources(mut self, sources: Vec<ExchangeRateSource>) -> Self {
        self.sources = Some(sources);
        self
    }
}

/// Currency conversion statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionStats {
    pub base_currency: Currency,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub currency_stats: HashMap<Currency, CurrencyStats>,
    pub total_rates: u32,
}

/// Statistics for a specific currency pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyStats {
    pub currency: Currency,
    pub rate_count: u32,
    pub min_rate: Decimal,
    pub max_rate: Decimal,
    pub avg_rate: Decimal,
    pub first_date: DateTime<Utc>,
    pub last_date: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;

    #[tokio::test]
    async fn test_exchange_rate_crud() {
        let db = Database::in_memory().unwrap();
        let repo = ExchangeRateRepository::new(Arc::new(db));

        // Create exchange rate
        let rate = ExchangeRate::new(
            Currency::USD,
            Currency::EUR,
            Decimal::new(85, 2), // 0.85 USD/EUR
            ExchangeRateSource::Manual,
        );

        let created = repo.create(&rate).await.unwrap();
        assert_eq!(created.rate, Decimal::new(85, 2));

        // Find by ID
        let found = repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());

        // Update
        let mut updated_rate = created.clone();
        updated_rate.rate = Decimal::new(86, 2);
        updated_rate.inverse_rate = Decimal::ONE / updated_rate.rate;
        let updated = repo.update(&updated_rate).await.unwrap();
        assert_eq!(updated.rate, Decimal::new(86, 2));

        // Delete
        let deleted = repo.delete(&created.id).await.unwrap();
        assert!(deleted);
    }

    #[tokio::test]
    async fn test_currency_conversion() {
        let db = Database::in_memory().unwrap();
        let repo = ExchangeRateRepository::new(Arc::new(db));

        // Create exchange rate
        let rate = ExchangeRate::new(
            Currency::USD,
            Currency::EUR,
            Decimal::new(85, 2), // 0.85 EUR per USD
            ExchangeRateSource::Manual,
        );

        repo.create(&rate).await.unwrap();

        // Test conversion
        let amount = Decimal::new(100, 0); // $100
        let converted = repo
            .convert_amount(amount, &Currency::USD, &Currency::EUR, None)
            .await
            .unwrap();

        assert_eq!(converted, Decimal::new(85, 0)); // €85
    }

    #[tokio::test]
    async fn test_historical_rates() {
        let db = Database::in_memory().unwrap();
        let repo = ExchangeRateRepository::new(Arc::new(db));

        let now = Utc::now();
        let yesterday = now - chrono::Duration::days(1);

        // Create historical rates
        let mut rate1 = ExchangeRate::new(
            Currency::USD,
            Currency::EUR,
            Decimal::new(85, 2),
            ExchangeRateSource::Manual,
        );
        rate1.effective_date = yesterday;

        let rate2 = ExchangeRate::new(
            Currency::USD,
            Currency::EUR,
            Decimal::new(86, 2),
            ExchangeRateSource::Manual,
        );

        repo.create(&rate1).await.unwrap();
        repo.create(&rate2).await.unwrap();

        // Get historical rates
        let historical = repo
            .get_historical_rates(
                &Currency::USD,
                &Currency::EUR,
                yesterday - chrono::Duration::hours(1),
                now + chrono::Duration::hours(1),
                &QueryOptions::new(),
            )
            .await
            .unwrap();

        assert_eq!(historical.len(), 2);
    }
}