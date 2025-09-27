//! Transaction aggregation module for FerrumFinance
//!
//! This module provides analytical capabilities for transaction data including
//! summaries, trends, categorization, and financial insights.

use crate::models::{
    currency::Currency,
    transaction::{Transaction, TransactionType, TransactionStatus},
};
use chrono::{DateTime, Utc, Duration, Datelike};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use thiserror::Error;

/// Time period for aggregation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimePeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    Custom { start: DateTime<Utc>, end: DateTime<Utc> },
}

/// Types of transaction aggregations available
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum of amounts
    Sum,
    /// Average of amounts
    Average,
    /// Count of transactions
    Count,
    /// Minimum amount
    Min,
    /// Maximum amount
    Max,
    /// Net flow (income - expenses)
    NetFlow,
}

/// Error types for transaction aggregation operations
#[derive(Error, Debug)]
pub enum AggregationError {
    #[error("No transactions found for the specified criteria")]
    NoTransactions,
    #[error("Invalid time period: {message}")]
    InvalidTimePeriod { message: String },
    #[error("Currency mismatch in aggregation")]
    CurrencyMismatch,
    #[error("Insufficient data for calculation: {message}")]
    InsufficientData { message: String },
}

/// Aggregated transaction summary for a specific period and criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    /// Period this summary covers
    pub period: TimePeriod,
    /// Currency for this summary
    pub currency: Currency,
    /// Start date of the period
    pub period_start: DateTime<Utc>,
    /// End date of the period
    pub period_end: DateTime<Utc>,
    /// Total number of transactions
    pub transaction_count: u32,
    /// Total amount of all transactions
    pub total_amount: Decimal,
    /// Average transaction amount
    pub average_amount: Decimal,
    /// Minimum transaction amount
    pub min_amount: Decimal,
    /// Maximum transaction amount
    pub max_amount: Decimal,
    /// Total income (credits)
    pub total_income: Decimal,
    /// Total expenses (debits)
    pub total_expenses: Decimal,
    /// Net flow (income - expenses)
    pub net_flow: Decimal,
    /// Breakdown by transaction type
    pub by_type: HashMap<TransactionType, TransactionTypeAggregate>,
    /// Breakdown by category
    pub by_category: HashMap<String, CategoryAggregate>,
    /// Transaction count by status
    pub by_status: HashMap<TransactionStatus, u32>,
}

/// Aggregated data for a specific transaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTypeAggregate {
    /// Transaction type
    pub transaction_type: TransactionType,
    /// Count of transactions of this type
    pub count: u32,
    /// Total amount for this transaction type
    pub total_amount: Decimal,
    /// Average amount for this transaction type
    pub average_amount: Decimal,
    /// Percentage of total transaction count
    pub percentage_of_count: Decimal,
    /// Percentage of total amount
    pub percentage_of_amount: Decimal,
}

/// Aggregated data for a specific category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAggregate {
    /// Category name
    pub category: String,
    /// Count of transactions in this category
    pub count: u32,
    /// Total amount for this category
    pub total_amount: Decimal,
    /// Average amount for this category
    pub average_amount: Decimal,
    /// Percentage of total transaction count
    pub percentage_of_count: Decimal,
    /// Percentage of total amount
    pub percentage_of_amount: Decimal,
}

/// Trend analysis for transactions over multiple periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionTrend {
    /// Period type for this trend analysis
    pub period_type: TimePeriod,
    /// Currency for this trend
    pub currency: Currency,
    /// Start date of the trend analysis
    pub analysis_start: DateTime<Utc>,
    /// End date of the trend analysis
    pub analysis_end: DateTime<Utc>,
    /// Data points for each period
    pub data_points: Vec<TrendDataPoint>,
    /// Overall trend direction
    pub trend_direction: TrendDirection,
    /// Growth rate (percentage change from first to last period)
    pub growth_rate: Decimal,
    /// Average amount across all periods
    pub overall_average: Decimal,
    /// Volatility (standard deviation)
    pub volatility: Decimal,
}

/// Single data point in a trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDataPoint {
    /// Period start date
    pub period_start: DateTime<Utc>,
    /// Period end date
    pub period_end: DateTime<Utc>,
    /// Value for this period
    pub value: Decimal,
    /// Transaction count for this period
    pub transaction_count: u32,
    /// Percentage change from previous period
    pub period_change: Option<Decimal>,
}

/// Direction of trend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Budget comparison analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAnalysis {
    /// Period this analysis covers
    pub period: TimePeriod,
    /// Currency for this analysis
    pub currency: Currency,
    /// Budget comparisons by category
    pub category_comparisons: HashMap<String, BudgetCategoryComparison>,
    /// Overall budget vs actual
    pub overall_budget: Decimal,
    /// Overall actual spending
    pub overall_actual: Decimal,
    /// Overall variance (actual - budget)
    pub overall_variance: Decimal,
    /// Overall variance percentage
    pub overall_variance_percentage: Decimal,
}

/// Budget comparison for a specific category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCategoryComparison {
    /// Category name
    pub category: String,
    /// Budgeted amount
    pub budgeted: Decimal,
    /// Actual amount spent
    pub actual: Decimal,
    /// Variance (actual - budget)
    pub variance: Decimal,
    /// Variance as percentage of budget
    pub variance_percentage: Decimal,
    /// Whether over or under budget
    pub status: BudgetStatus,
}

/// Budget status for a category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetStatus {
    UnderBudget,
    OnBudget,
    OverBudget,
}

/// Transaction aggregator that performs various analytical calculations
#[derive(Debug, Clone)]
pub struct TransactionAggregator {
    /// Base currency for conversions
    pub base_currency: Currency,
    /// Tolerance for "on budget" comparison (percentage)
    pub budget_tolerance: Decimal,
}

impl TransactionAggregator {
    /// Creates a new transaction aggregator
    pub fn new(base_currency: Currency) -> Self {
        TransactionAggregator {
            base_currency,
            budget_tolerance: Decimal::new(5, 2), // 5% tolerance
        }
    }

    /// Creates a summary for transactions in the given period
    pub fn create_summary(
        &self,
        transactions: &[Transaction],
        period: TimePeriod,
        currency: Option<Currency>,
    ) -> Result<TransactionSummary, AggregationError> {
        if transactions.is_empty() {
            return Err(AggregationError::NoTransactions);
        }

        let target_currency = currency.clone().unwrap_or_else(|| self.base_currency.clone());
        let (period_start, period_end) = self.get_period_bounds(&period)?;

        // Filter transactions by period and currency
        let filtered_transactions: Vec<&Transaction> = transactions
            .iter()
            .filter(|t| {
                let tx_date = t.effective_date();
                tx_date >= period_start && tx_date <= period_end
            })
            .filter(|t| currency.is_none() || t.currency == target_currency)
            .collect();

        if filtered_transactions.is_empty() {
            return Err(AggregationError::NoTransactions);
        }

        let transaction_count = filtered_transactions.len() as u32;
        let amounts: Vec<Decimal> = filtered_transactions.iter().map(|t| t.amount).collect();

        let total_amount = amounts.iter().sum();
        let average_amount = total_amount / Decimal::from(transaction_count);
        let min_amount = amounts.iter().min().copied().unwrap_or(Decimal::ZERO);
        let max_amount = amounts.iter().max().copied().unwrap_or(Decimal::ZERO);

        // Calculate income and expenses
        let (total_income, total_expenses) = self.calculate_income_expenses(&filtered_transactions);
        let net_flow = total_income - total_expenses;

        // Aggregate by transaction type
        let by_type = self.aggregate_by_type(&filtered_transactions, total_amount, transaction_count);

        // Aggregate by category
        let by_category = self.aggregate_by_category(&filtered_transactions, total_amount, transaction_count);

        // Count by status
        let by_status = self.count_by_status(&filtered_transactions);

        Ok(TransactionSummary {
            period,
            currency: target_currency,
            period_start,
            period_end,
            transaction_count,
            total_amount,
            average_amount,
            min_amount,
            max_amount,
            total_income,
            total_expenses,
            net_flow,
            by_type,
            by_category,
            by_status,
        })
    }

    /// Creates a trend analysis over multiple periods
    pub fn create_trend_analysis(
        &self,
        transactions: &[Transaction],
        period_type: TimePeriod,
        num_periods: u32,
        aggregation_type: AggregationType,
        currency: Option<Currency>,
    ) -> Result<TransactionTrend, AggregationError> {
        let target_currency = currency.clone().unwrap_or_else(|| self.base_currency.clone());
        let mut data_points: Vec<TrendDataPoint> = Vec::new();

        // Generate periods and calculate values
        let end_date = Utc::now();
        for i in 0..num_periods {
            let period_end = self.subtract_period(&end_date, &period_type, i)?;
            let period_start = self.subtract_period(&period_end, &period_type, 1)?;

            let period_transactions: Vec<&Transaction> = transactions
                .iter()
                .filter(|t| {
                    let tx_date = t.effective_date();
                    tx_date >= period_start && tx_date <= period_end
                })
                .filter(|t| currency.is_none() || t.currency == target_currency)
                .collect();

            let value = match aggregation_type {
                AggregationType::Sum => period_transactions.iter().map(|t| t.amount).sum(),
                AggregationType::Average => {
                    if period_transactions.is_empty() {
                        Decimal::ZERO
                    } else {
                        period_transactions.iter().map(|t| t.amount).sum::<Decimal>() / Decimal::from(period_transactions.len())
                    }
                }
                AggregationType::Count => Decimal::from(period_transactions.len()),
                AggregationType::Min => period_transactions.iter().map(|t| t.amount).min().unwrap_or(Decimal::ZERO),
                AggregationType::Max => period_transactions.iter().map(|t| t.amount).max().unwrap_or(Decimal::ZERO),
                AggregationType::NetFlow => {
                    let (income, expenses) = self.calculate_income_expenses(&period_transactions);
                    income - expenses
                }
            };

            let period_change = if i > 0 && !data_points.is_empty() {
                let prev_value = data_points[0].value;
                if prev_value != Decimal::ZERO {
                    Some((value - prev_value) / prev_value * Decimal::from(100))
                } else {
                    None
                }
            } else {
                None
            };

            data_points.insert(0, TrendDataPoint {
                period_start,
                period_end,
                value,
                transaction_count: period_transactions.len() as u32,
                period_change,
            });
        }

        // Analyze trend direction
        let trend_direction = self.analyze_trend_direction(&data_points);

        // Calculate growth rate
        let growth_rate = if data_points.len() >= 2 {
            let first_value = data_points[0].value;
            let last_value = data_points[data_points.len() - 1].value;
            if first_value != Decimal::ZERO {
                (last_value - first_value) / first_value * Decimal::from(100)
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        // Calculate overall average and volatility
        let values: Vec<Decimal> = data_points.iter().map(|dp| dp.value).collect();
        let overall_average = if !values.is_empty() {
            values.iter().sum::<Decimal>() / Decimal::from(values.len())
        } else {
            Decimal::ZERO
        };

        let volatility = self.calculate_volatility(&values, overall_average);

        let analysis_start = data_points.first().map(|dp| dp.period_start).unwrap_or(end_date);
        let analysis_end = data_points.last().map(|dp| dp.period_end).unwrap_or(end_date);

        Ok(TransactionTrend {
            period_type,
            currency: target_currency,
            analysis_start,
            analysis_end,
            data_points,
            trend_direction,
            growth_rate,
            overall_average,
            volatility,
        })
    }

    /// Helper method to get period bounds
    fn get_period_bounds(&self, period: &TimePeriod) -> Result<(DateTime<Utc>, DateTime<Utc>), AggregationError> {
        let now = Utc::now();
        match period {
            TimePeriod::Daily => {
                let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start + Duration::days(1) - Duration::seconds(1);
                Ok((start, end))
            }
            TimePeriod::Weekly => {
                let days_since_monday = now.weekday().num_days_from_monday();
                let start = (now - Duration::days(days_since_monday as i64))
                    .date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = start + Duration::days(7) - Duration::seconds(1);
                Ok((start, end))
            }
            TimePeriod::Monthly => {
                let start = now.date_naive()
                    .with_day(1).unwrap()
                    .and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end = if now.month() == 12 {
                    start.with_year(start.year() + 1).unwrap().with_month(1).unwrap()
                } else {
                    start.with_month(start.month() + 1).unwrap()
                } - Duration::seconds(1);
                Ok((start, end))
            }
            TimePeriod::Custom { start, end } => {
                if start > end {
                    return Err(AggregationError::InvalidTimePeriod {
                        message: "Start date must be before end date".to_string(),
                    });
                }
                Ok((*start, *end))
            }
            _ => Err(AggregationError::InvalidTimePeriod {
                message: "Period type not yet implemented".to_string(),
            }),
        }
    }

    /// Helper to subtract periods from a date
    fn subtract_period(&self, date: &DateTime<Utc>, period: &TimePeriod, count: u32) -> Result<DateTime<Utc>, AggregationError> {
        match period {
            TimePeriod::Daily => Ok(*date - Duration::days(count as i64)),
            TimePeriod::Weekly => Ok(*date - Duration::weeks(count as i64)),
            TimePeriod::Monthly => {
                let mut result = *date;
                for _ in 0..count {
                    result = if result.month() == 1 {
                        result.with_year(result.year() - 1).unwrap().with_month(12).unwrap()
                    } else {
                        result.with_month(result.month() - 1).unwrap()
                    };
                }
                Ok(result)
            }
            _ => Err(AggregationError::InvalidTimePeriod {
                message: "Period type not supported for subtraction".to_string(),
            }),
        }
    }

    /// Calculates income and expenses from transactions
    fn calculate_income_expenses(&self, transactions: &[&Transaction]) -> (Decimal, Decimal) {
        let mut income = Decimal::ZERO;
        let mut expenses = Decimal::ZERO;

        for transaction in transactions {
            match transaction.transaction_type {
                TransactionType::Income | TransactionType::Deposit | TransactionType::Dividend => {
                    income += transaction.amount;
                }
                TransactionType::Expense | TransactionType::Withdrawal | TransactionType::Payment | TransactionType::Fee => {
                    expenses += transaction.amount;
                }
                _ => {} // Other types don't contribute to income/expense calculation
            }
        }

        (income, expenses)
    }

    /// Aggregates transactions by type
    fn aggregate_by_type(
        &self,
        transactions: &[&Transaction],
        total_amount: Decimal,
        total_count: u32,
    ) -> HashMap<TransactionType, TransactionTypeAggregate> {
        let mut by_type: HashMap<TransactionType, Vec<&Transaction>> = HashMap::new();

        // Group transactions by type
        for transaction in transactions {
            by_type.entry(transaction.transaction_type.clone())
                .or_insert_with(Vec::new)
                .push(transaction);
        }

        // Create aggregates
        by_type.into_iter().map(|(tx_type, tx_list)| {
            let count = tx_list.len() as u32;
            let type_total: Decimal = tx_list.iter().map(|t| t.amount).sum();
            let average_amount = if count > 0 {
                type_total / Decimal::from(count)
            } else {
                Decimal::ZERO
            };

            let percentage_of_count = if total_count > 0 {
                Decimal::from(count) / Decimal::from(total_count) * Decimal::from(100)
            } else {
                Decimal::ZERO
            };

            let percentage_of_amount = if total_amount > Decimal::ZERO {
                type_total / total_amount * Decimal::from(100)
            } else {
                Decimal::ZERO
            };

            (tx_type.clone(), TransactionTypeAggregate {
                transaction_type: tx_type,
                count,
                total_amount: type_total,
                average_amount,
                percentage_of_count,
                percentage_of_amount,
            })
        }).collect()
    }

    /// Aggregates transactions by category
    fn aggregate_by_category(
        &self,
        transactions: &[&Transaction],
        total_amount: Decimal,
        total_count: u32,
    ) -> HashMap<String, CategoryAggregate> {
        let mut by_category: HashMap<String, Vec<&Transaction>> = HashMap::new();

        // Group transactions by category
        for transaction in transactions {
            let category = transaction.category.clone().unwrap_or_else(|| "Uncategorized".to_string());
            by_category.entry(category)
                .or_insert_with(Vec::new)
                .push(transaction);
        }

        // Create aggregates
        by_category.into_iter().map(|(category, tx_list)| {
            let count = tx_list.len() as u32;
            let category_total: Decimal = tx_list.iter().map(|t| t.amount).sum();
            let average_amount = if count > 0 {
                category_total / Decimal::from(count)
            } else {
                Decimal::ZERO
            };

            let percentage_of_count = if total_count > 0 {
                Decimal::from(count) / Decimal::from(total_count) * Decimal::from(100)
            } else {
                Decimal::ZERO
            };

            let percentage_of_amount = if total_amount > Decimal::ZERO {
                category_total / total_amount * Decimal::from(100)
            } else {
                Decimal::ZERO
            };

            (category.clone(), CategoryAggregate {
                category: category.clone(),
                count,
                total_amount: category_total,
                average_amount,
                percentage_of_count,
                percentage_of_amount,
            })
        }).collect()
    }

    /// Counts transactions by status
    fn count_by_status(&self, transactions: &[&Transaction]) -> HashMap<TransactionStatus, u32> {
        let mut by_status = HashMap::new();

        for transaction in transactions {
            *by_status.entry(transaction.status.clone()).or_insert(0) += 1;
        }

        by_status
    }

    /// Analyzes trend direction from data points
    fn analyze_trend_direction(&self, data_points: &[TrendDataPoint]) -> TrendDirection {
        if data_points.len() < 2 {
            return TrendDirection::Stable;
        }

        let mut increasing_count = 0;
        let mut decreasing_count = 0;

        for i in 1..data_points.len() {
            let current = data_points[i].value;
            let previous = data_points[i - 1].value;

            if current > previous {
                increasing_count += 1;
            } else if current < previous {
                decreasing_count += 1;
            }
        }

        let total_changes = increasing_count + decreasing_count;
        if total_changes == 0 {
            return TrendDirection::Stable;
        }

        let increasing_ratio = increasing_count as f64 / total_changes as f64;

        match increasing_ratio {
            r if r >= 0.7 => TrendDirection::Increasing,
            r if r <= 0.3 => TrendDirection::Decreasing,
            r if r >= 0.4 && r <= 0.6 => TrendDirection::Volatile,
            _ => TrendDirection::Stable,
        }
    }

    /// Calculates volatility (standard deviation) of values
    fn calculate_volatility(&self, values: &[Decimal], average: Decimal) -> Decimal {
        if values.len() < 2 {
            return Decimal::ZERO;
        }

        let variance: Decimal = values.iter()
            .map(|v| {
                let diff = *v - average;
                diff * diff // Use multiplication instead of powi
            })
            .sum::<Decimal>() / Decimal::from(values.len() - 1);

        // Approximate square root using Newton's method
        if variance <= Decimal::ZERO {
            Decimal::ZERO
        } else {
            let mut result = variance / Decimal::from(2);
            for _ in 0..10 {
                result = (result + variance / result) / Decimal::from(2);
            }
            result
        }
    }
}

impl Display for TimePeriod {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TimePeriod::Daily => write!(f, "Daily"),
            TimePeriod::Weekly => write!(f, "Weekly"),
            TimePeriod::Monthly => write!(f, "Monthly"),
            TimePeriod::Quarterly => write!(f, "Quarterly"),
            TimePeriod::Yearly => write!(f, "Yearly"),
            TimePeriod::Custom { start, end } => write!(f, "Custom ({} to {})", start.format("%Y-%m-%d"), end.format("%Y-%m-%d")),
        }
    }
}

impl Display for TrendDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TrendDirection::Increasing => write!(f, "Increasing"),
            TrendDirection::Decreasing => write!(f, "Decreasing"),
            TrendDirection::Stable => write!(f, "Stable"),
            TrendDirection::Volatile => write!(f, "Volatile"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::Transaction;

    fn create_test_transaction(amount: Decimal, tx_type: TransactionType, currency: Currency) -> Transaction {
        let mut tx = Transaction::new(tx_type, amount, currency).unwrap();
        tx.status = TransactionStatus::Completed;
        tx
    }

    #[test]
    fn test_aggregator_creation() {
        let aggregator = TransactionAggregator::new(Currency::USD);
        assert_eq!(aggregator.base_currency, Currency::USD);
        assert_eq!(aggregator.budget_tolerance, Decimal::new(5, 2));
    }

    #[test]
    fn test_summary_creation() {
        let aggregator = TransactionAggregator::new(Currency::USD);

        let transactions = vec![
            create_test_transaction(Decimal::new(100, 0), TransactionType::Income, Currency::USD),
            create_test_transaction(Decimal::new(50, 0), TransactionType::Expense, Currency::USD),
            create_test_transaction(Decimal::new(25, 0), TransactionType::Expense, Currency::USD),
        ];

        let period = TimePeriod::Custom {
            start: Utc::now() - Duration::days(1),
            end: Utc::now() + Duration::days(1),
        };

        let summary = aggregator.create_summary(&transactions, period, Some(Currency::USD));
        assert!(summary.is_ok());

        let summary = summary.unwrap();
        assert_eq!(summary.transaction_count, 3);
        assert_eq!(summary.total_amount, Decimal::new(175, 0));
        assert_eq!(summary.total_income, Decimal::new(100, 0));
        assert_eq!(summary.total_expenses, Decimal::new(75, 0));
        assert_eq!(summary.net_flow, Decimal::new(25, 0));
    }

    #[test]
    fn test_empty_transactions() {
        let aggregator = TransactionAggregator::new(Currency::USD);
        let transactions = vec![];

        let period = TimePeriod::Daily;
        let result = aggregator.create_summary(&transactions, period, Some(Currency::USD));

        assert!(result.is_err());
        if let Err(AggregationError::NoTransactions) = result {
            // Expected error
        } else {
            panic!("Expected NoTransactions error");
        }
    }

    #[test]
    fn test_income_expenses_calculation() {
        let aggregator = TransactionAggregator::new(Currency::USD);

        let transactions = vec![
            create_test_transaction(Decimal::new(100, 0), TransactionType::Income, Currency::USD),
            create_test_transaction(Decimal::new(50, 0), TransactionType::Expense, Currency::USD),
        ];

        let transaction_refs: Vec<&Transaction> = transactions.iter().collect();
        let (income, expenses) = aggregator.calculate_income_expenses(&transaction_refs);

        assert_eq!(income, Decimal::new(100, 0));
        assert_eq!(expenses, Decimal::new(50, 0));
    }
}