//! Loan module for FerrumFinance
//!
//! This module provides comprehensive loan management including various loan types,
//! interest calculations, payment schedules, and loan lifecycle management.

use crate::models::currency::Currency;
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use thiserror::Error;
use uuid::Uuid;

/// Types of loans supported by the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanType {
    /// Personal/consumer loan
    Personal,
    /// Home mortgage loan
    Mortgage,
    /// Business loan
    Business,
    /// Auto/vehicle loan
    Auto,
    /// Student loan
    Student,
    /// Credit line/revolving credit
    CreditLine,
    /// Equipment financing
    Equipment,
    /// Construction loan
    Construction,
    /// Bridge loan
    Bridge,
    /// Custom loan type
    Custom(String),
}

/// Interest rate structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterestRateType {
    /// Fixed interest rate throughout the loan term
    Fixed,
    /// Variable interest rate that can change
    Variable,
    /// Adjustable rate mortgage (ARM) with specific adjustment periods
    Adjustable {
        initial_period_months: u32,
        adjustment_period_months: u32,
        rate_cap_per_adjustment: Option<Decimal>,
        lifetime_rate_cap: Option<Decimal>,
    },
}

/// Payment frequency options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentFrequency {
    /// Monthly payments (12 per year)
    Monthly,
    /// Bi-weekly payments (26 per year)
    BiWeekly,
    /// Weekly payments (52 per year)
    Weekly,
    /// Quarterly payments (4 per year)
    Quarterly,
    /// Semi-annual payments (2 per year)
    SemiAnnual,
    /// Annual payments (1 per year)
    Annual,
    /// Custom payment schedule
    Custom { payments_per_year: u32 },
}

/// Current status of a loan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanStatus {
    /// Loan application is pending approval
    Pending,
    /// Loan has been approved but not yet disbursed
    Approved,
    /// Loan is active and payments are being made
    Active,
    /// Loan payments are past due
    Delinquent,
    /// Loan is in default
    Default,
    /// Loan has been paid off completely
    PaidOff,
    /// Loan has been charged off
    ChargedOff,
    /// Loan is in forbearance
    Forbearance,
    /// Loan has been cancelled
    Cancelled,
}

/// Payment calculation method
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentCalculationMethod {
    /// Equal monthly payments (most common)
    EqualPayments,
    /// Interest-only payments for a period
    InterestOnly { interest_only_months: u32 },
    /// Balloon payment at the end
    Balloon { balloon_amount: Decimal },
    /// Custom payment schedule
    Custom,
}

impl Display for LoanType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            LoanType::Personal => write!(f, "Personal"),
            LoanType::Mortgage => write!(f, "Mortgage"),
            LoanType::Business => write!(f, "Business"),
            LoanType::Auto => write!(f, "Auto"),
            LoanType::Student => write!(f, "Student"),
            LoanType::CreditLine => write!(f, "Line of Credit"),
            LoanType::Equipment => write!(f, "Equipment"),
            LoanType::Construction => write!(f, "Construction"),
            LoanType::Bridge => write!(f, "Bridge"),
            LoanType::Custom(ref name) => write!(f, "{}", name),
        }
    }
}

impl Display for LoanStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            LoanStatus::Pending => write!(f, "Pending"),
            LoanStatus::Approved => write!(f, "Approved"),
            LoanStatus::Active => write!(f, "Active"),
            LoanStatus::Delinquent => write!(f, "Delinquent"),
            LoanStatus::Default => write!(f, "Defaulted"),
            LoanStatus::PaidOff => write!(f, "Paid Off"),
            LoanStatus::ChargedOff => write!(f, "Charged Off"),
            LoanStatus::Forbearance => write!(f, "Forbearance"),
            LoanStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl Display for InterestRateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            InterestRateType::Fixed => write!(f, "Fixed"),
            InterestRateType::Variable => write!(f, "Variable"),
            InterestRateType::Adjustable { .. } => write!(f, "Adjustable"),
        }
    }
}

impl Display for PaymentFrequency {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PaymentFrequency::Monthly => write!(f, "Monthly"),
            PaymentFrequency::BiWeekly => write!(f, "BiWeekly"),
            PaymentFrequency::Weekly => write!(f, "Weekly"),
            PaymentFrequency::Quarterly => write!(f, "Quarterly"),
            PaymentFrequency::SemiAnnual => write!(f, "SemiAnnually"),
            PaymentFrequency::Annual => write!(f, "Annually"),
            PaymentFrequency::Custom { payments_per_year } => write!(f, "Custom({})", payments_per_year),
        }
    }
}

impl Display for PaymentCalculationMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PaymentCalculationMethod::EqualPayments => write!(f, "EqualPayments"),
            PaymentCalculationMethod::InterestOnly { .. } => write!(f, "InterestOnly"),
            PaymentCalculationMethod::Balloon { .. } => write!(f, "BalloonPayment"),
            PaymentCalculationMethod::Custom => write!(f, "Custom"),
        }
    }
}

/// Error types for loan operations
#[derive(Error, Debug)]
pub enum LoanError {
    #[error("Invalid loan amount: {amount}")]
    InvalidAmount { amount: Decimal },
    #[error("Invalid interest rate: {rate}")]
    InvalidInterestRate { rate: Decimal },
    #[error("Invalid loan term: {months} months")]
    InvalidTerm { months: u32 },
    #[error("Loan calculation error: {message}")]
    CalculationError { message: String },
    #[error("Payment amount {payment} is less than minimum required {minimum}")]
    InsufficientPayment { payment: Decimal, minimum: Decimal },
    #[error("Loan not in active status: {status:?}")]
    InvalidStatus { status: LoanStatus },
}

/// Represents a single loan payment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanPayment {
    /// Payment number in the sequence
    pub payment_number: u32,
    /// Due date for this payment
    pub due_date: DateTime<Utc>,
    /// Total payment amount
    pub payment_amount: Decimal,
    /// Principal portion of the payment
    pub principal_amount: Decimal,
    /// Interest portion of the payment
    pub interest_amount: Decimal,
    /// Remaining principal balance after this payment
    pub remaining_balance: Decimal,
    /// Whether this payment has been made
    pub is_paid: bool,
    /// Date when payment was made
    pub paid_date: Option<DateTime<Utc>>,
    /// Actual amount paid (may differ from scheduled)
    pub amount_paid: Option<Decimal>,
    /// Late fees applied to this payment
    pub late_fees: Option<Decimal>,
}

/// Comprehensive loan structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    /// Unique identifier for the loan
    pub id: Uuid,
    /// Type of loan
    pub loan_type: LoanType,
    /// Current status of the loan
    pub status: LoanStatus,
    /// Original loan amount (principal)
    pub principal_amount: Decimal,
    /// Current outstanding balance
    pub outstanding_balance: Decimal,
    /// Annual interest rate as a decimal (e.g., 0.05 for 5%)
    pub interest_rate: Decimal,
    /// Interest rate type (fixed, variable, etc.)
    pub interest_rate_type: InterestRateType,
    /// Loan term in months
    pub term_months: u32,
    /// Payment frequency
    pub payment_frequency: PaymentFrequency,
    /// Payment calculation method
    pub payment_calculation_method: PaymentCalculationMethod,
    /// Currency of the loan
    pub currency: Currency,
    /// Calculated monthly payment amount
    pub payment_amount: Decimal,
    /// Loan origination date
    pub origination_date: DateTime<Utc>,
    /// First payment due date
    pub first_payment_date: DateTime<Utc>,
    /// Maturity date (when loan should be fully paid)
    pub maturity_date: DateTime<Utc>,
    /// Last payment date
    pub last_payment_date: Option<DateTime<Utc>>,
    /// Total amount paid to date
    pub total_paid: Decimal,
    /// Total interest paid to date
    pub total_interest_paid: Decimal,
    /// Number of payments made
    pub payments_made: u32,
    /// Number of missed/late payments
    pub missed_payments: u32,
    /// Late fees accumulated
    pub late_fees: Decimal,
    /// Loan description or purpose
    pub description: Option<String>,
    /// External loan reference number
    pub reference_number: Option<String>,
    /// Lender information
    pub lender: Option<String>,
    /// Borrower information
    pub borrower: Option<String>,
    /// Collateral description (if applicable)
    pub collateral: Option<String>,
    /// Additional terms and conditions
    pub terms: HashMap<String, String>,
    /// Associated account ID for payments
    pub payment_account_id: Option<Uuid>,
    /// Associated account ID for disbursement
    pub disbursement_account_id: Option<Uuid>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Payment schedule (cached for performance)
    pub payment_schedule: Option<Vec<LoanPayment>>,
}

impl Loan {
    /// Creates a new loan with basic parameters
    pub fn new(
        loan_type: LoanType,
        principal_amount: Decimal,
        interest_rate: Decimal,
        term_months: u32,
        currency: Currency,
    ) -> Result<Self, LoanError> {
        if principal_amount <= Decimal::ZERO {
            return Err(LoanError::InvalidAmount { amount: principal_amount });
        }

        if interest_rate < Decimal::ZERO || interest_rate > Decimal::ONE {
            return Err(LoanError::InvalidInterestRate { rate: interest_rate });
        }

        if term_months == 0 {
            return Err(LoanError::InvalidTerm { months: term_months });
        }

        let now = Utc::now();
        let first_payment_date = now + Duration::days(30); // Default to 30 days from now
        let maturity_date = now + Duration::days((term_months as i64) * 30);

        let payment_amount = Self::calculate_payment_amount(
            principal_amount,
            interest_rate,
            term_months,
            &PaymentFrequency::Monthly,
            &PaymentCalculationMethod::EqualPayments,
        )?;

        Ok(Loan {
            id: Uuid::new_v4(),
            loan_type,
            status: LoanStatus::Pending,
            principal_amount,
            outstanding_balance: principal_amount,
            interest_rate,
            interest_rate_type: InterestRateType::Fixed,
            term_months,
            payment_frequency: PaymentFrequency::Monthly,
            payment_calculation_method: PaymentCalculationMethod::EqualPayments,
            currency,
            payment_amount,
            origination_date: now,
            first_payment_date,
            maturity_date,
            last_payment_date: None,
            total_paid: Decimal::ZERO,
            total_interest_paid: Decimal::ZERO,
            payments_made: 0,
            missed_payments: 0,
            late_fees: Decimal::ZERO,
            description: None,
            reference_number: None,
            lender: None,
            borrower: None,
            collateral: None,
            terms: HashMap::new(),
            payment_account_id: None,
            disbursement_account_id: None,
            created_at: now,
            updated_at: now,
            payment_schedule: None,
        })
    }

    /// Calculates the payment amount based on loan parameters
    pub fn calculate_payment_amount(
        principal: Decimal,
        annual_rate: Decimal,
        term_months: u32,
        frequency: &PaymentFrequency,
        calculation_method: &PaymentCalculationMethod,
    ) -> Result<Decimal, LoanError> {
        match calculation_method {
            PaymentCalculationMethod::EqualPayments => {
                let payments_per_year = frequency.payments_per_year();
                let periodic_rate = annual_rate / Decimal::from(payments_per_year);
                let total_payments = Decimal::from(term_months) * Decimal::from(payments_per_year) / Decimal::from(12);

                if periodic_rate == Decimal::ZERO {
                    // No interest loan
                    Ok(principal / total_payments)
                } else {
                    // Standard amortization formula: P * [r(1+r)^n] / [(1+r)^n - 1]
                    let one_plus_r = Decimal::ONE + periodic_rate;
                    let periods = total_payments.to_u32().unwrap_or(0);

                    // Calculate (1+r)^n using repeated multiplication
                    let mut power_term = Decimal::ONE;
                    for _ in 0..periods {
                        power_term *= one_plus_r;
                    }

                    let numerator = periodic_rate * power_term;
                    let denominator = power_term - Decimal::ONE;

                    if denominator == Decimal::ZERO {
                        return Err(LoanError::CalculationError {
                            message: "Invalid calculation parameters".to_string(),
                        });
                    }

                    Ok(principal * numerator / denominator)
                }
            }
            PaymentCalculationMethod::InterestOnly { .. } => {
                let payments_per_year = frequency.payments_per_year();
                let periodic_rate = annual_rate / Decimal::from(payments_per_year);
                Ok(principal * periodic_rate)
            }
            PaymentCalculationMethod::Balloon { .. } => {
                // For balloon loans, calculate based on amortization schedule
                // but with a balloon payment at the end
                Self::calculate_payment_amount(
                    principal,
                    annual_rate,
                    term_months,
                    frequency,
                    &PaymentCalculationMethod::EqualPayments,
                )
            }
            PaymentCalculationMethod::Custom => {
                // Custom calculation would be implemented based on specific requirements
                Ok(Decimal::ZERO)
            }
        }
    }

    /// Generates the complete payment schedule for the loan
    pub fn generate_payment_schedule(&mut self) -> Result<Vec<LoanPayment>, LoanError> {
        let payments_per_year = self.payment_frequency.payments_per_year();
        let total_payments = (self.term_months as f64 * payments_per_year as f64 / 12.0) as u32;
        let periodic_rate = self.interest_rate / Decimal::from(payments_per_year);

        let mut schedule = Vec::new();
        let mut remaining_balance = self.principal_amount;
        let mut current_date = self.first_payment_date;

        let days_between_payments = match self.payment_frequency {
            PaymentFrequency::Monthly => 30,
            PaymentFrequency::BiWeekly => 14,
            PaymentFrequency::Weekly => 7,
            PaymentFrequency::Quarterly => 90,
            PaymentFrequency::SemiAnnual => 182,
            PaymentFrequency::Annual => 365,
            PaymentFrequency::Custom { payments_per_year } => 365 / payments_per_year as i64,
        };

        for payment_num in 1..=total_payments {
            let interest_amount = remaining_balance * periodic_rate;
            let mut principal_amount = self.payment_amount - interest_amount;

            // Ensure the last payment covers any remaining balance
            if payment_num == total_payments || principal_amount >= remaining_balance {
                principal_amount = remaining_balance;
            }

            remaining_balance -= principal_amount;

            schedule.push(LoanPayment {
                payment_number: payment_num,
                due_date: current_date,
                payment_amount: self.payment_amount,
                principal_amount,
                interest_amount,
                remaining_balance,
                is_paid: false,
                paid_date: None,
                amount_paid: None,
                late_fees: None,
            });

            current_date = current_date + Duration::days(days_between_payments);

            if remaining_balance <= Decimal::ZERO {
                break;
            }
        }

        self.payment_schedule = Some(schedule.clone());
        self.updated_at = Utc::now();

        Ok(schedule)
    }

    /// Records a payment against the loan
    pub fn make_payment(&mut self, amount: Decimal, payment_date: DateTime<Utc>) -> Result<(), LoanError> {
        if self.status != LoanStatus::Active {
            return Err(LoanError::InvalidStatus { status: self.status.clone() });
        }

        if amount <= Decimal::ZERO {
            return Err(LoanError::InvalidAmount { amount });
        }

        // Apply payment to outstanding balance
        self.outstanding_balance -= amount;
        self.total_paid += amount;
        self.payments_made += 1;
        self.last_payment_date = Some(payment_date);

        // Update payment schedule if it exists
        if let Some(ref mut schedule) = self.payment_schedule {
            // Find the next unpaid payment and mark it as paid
            if let Some(payment) = schedule.iter_mut().find(|p| !p.is_paid) {
                payment.is_paid = true;
                payment.paid_date = Some(payment_date);
                payment.amount_paid = Some(amount);
                self.total_interest_paid += payment.interest_amount;
            }
        }

        // Check if loan is paid off
        if self.outstanding_balance <= Decimal::ZERO {
            self.status = LoanStatus::PaidOff;
            self.outstanding_balance = Decimal::ZERO;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Gets the next payment due
    pub fn get_next_payment(&self) -> Option<&LoanPayment> {
        self.payment_schedule.as_ref()?
            .iter()
            .find(|payment| !payment.is_paid)
    }

    /// Calculates total interest that will be paid over the life of the loan
    pub fn total_interest(&self) -> Decimal {
        if let Some(ref schedule) = self.payment_schedule {
            schedule.iter().map(|p| p.interest_amount).sum()
        } else {
            (self.payment_amount * Decimal::from(self.term_months)) - self.principal_amount
        }
    }

    /// Gets the percentage of the loan that has been paid
    pub fn payment_progress(&self) -> Decimal {
        if self.principal_amount == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let paid_principal = self.principal_amount - self.outstanding_balance;
        (paid_principal / self.principal_amount) * Decimal::from(100)
    }

    /// Sets the loan status
    pub fn set_status(&mut self, status: LoanStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Adds a term or condition to the loan
    pub fn add_term<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.terms.insert(key.into(), value.into());
        self.updated_at = Utc::now();
    }
}

impl PaymentFrequency {
    /// Returns the number of payments per year for this frequency
    pub fn payments_per_year(&self) -> u32 {
        match self {
            PaymentFrequency::Monthly => 12,
            PaymentFrequency::BiWeekly => 26,
            PaymentFrequency::Weekly => 52,
            PaymentFrequency::Quarterly => 4,
            PaymentFrequency::SemiAnnual => 2,
            PaymentFrequency::Annual => 1,
            PaymentFrequency::Custom { payments_per_year } => *payments_per_year,
        }
    }
}

impl Display for LoanType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            LoanType::Personal => write!(f, "Personal Loan"),
            LoanType::Mortgage => write!(f, "Mortgage"),
            LoanType::Business => write!(f, "Business Loan"),
            LoanType::Auto => write!(f, "Auto Loan"),
            LoanType::Student => write!(f, "Student Loan"),
            LoanType::CreditLine => write!(f, "Credit Line"),
            LoanType::Equipment => write!(f, "Equipment Loan"),
            LoanType::Construction => write!(f, "Construction Loan"),
            LoanType::Bridge => write!(f, "Bridge Loan"),
            LoanType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl Display for LoanStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            LoanStatus::Pending => write!(f, "Pending"),
            LoanStatus::Approved => write!(f, "Approved"),
            LoanStatus::Active => write!(f, "Active"),
            LoanStatus::Delinquent => write!(f, "Delinquent"),
            LoanStatus::Default => write!(f, "Default"),
            LoanStatus::PaidOff => write!(f, "Paid Off"),
            LoanStatus::ChargedOff => write!(f, "Charged Off"),
            LoanStatus::Forbearance => write!(f, "Forbearance"),
            LoanStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl Display for Loan {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} - {} {} @ {:.2}% [{} payments] [{}]",
            self.loan_type,
            self.currency.symbol(),
            self.principal_amount,
            self.interest_rate * Decimal::from(100),
            self.term_months,
            self.status
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loan_creation() {
        let loan = Loan::new(
            LoanType::Personal,
            Decimal::new(10000, 0),
            Decimal::new(5, 2), // 5%
            60, // 5 years
            Currency::USD,
        );

        assert!(loan.is_ok());
        let loan = loan.unwrap();
        assert_eq!(loan.loan_type, LoanType::Personal);
        assert_eq!(loan.principal_amount, Decimal::new(10000, 0));
        assert_eq!(loan.outstanding_balance, Decimal::new(10000, 0));
        assert_eq!(loan.status, LoanStatus::Pending);
    }

    #[test]
    fn test_payment_calculation() {
        let payment = Loan::calculate_payment_amount(
            Decimal::new(10000, 0), // $10,000
            Decimal::new(5, 2),     // 5%
            60,                     // 60 months
            &PaymentFrequency::Monthly,
            &PaymentCalculationMethod::EqualPayments,
        );

        assert!(payment.is_ok());
        let payment = payment.unwrap();
        // Should be approximately $188.71 for a $10k loan at 5% for 5 years
        assert!(payment > Decimal::new(180, 0));
        assert!(payment < Decimal::new(200, 0));
    }

    #[test]
    fn test_payment_schedule_generation() {
        let mut loan = Loan::new(
            LoanType::Personal,
            Decimal::new(1000, 0), // $1,000 for simpler testing
            Decimal::new(12, 2),   // 12%
            12,                    // 12 months
            Currency::USD,
        ).unwrap();

        let schedule = loan.generate_payment_schedule();
        assert!(schedule.is_ok());

        let schedule = schedule.unwrap();
        assert_eq!(schedule.len(), 12);

        // First payment should have some interest component
        let first_payment = &schedule[0];
        assert!(first_payment.interest_amount > Decimal::ZERO);

        // Last payment should be mostly principal
        let last_payment = &schedule[11];
        assert!(last_payment.principal_amount > last_payment.interest_amount);

        // Final balance should be zero (or very close)
        assert!(last_payment.remaining_balance < Decimal::new(1, 0));
    }

    #[test]
    fn test_loan_payment() {
        let mut loan = Loan::new(
            LoanType::Personal,
            Decimal::new(1000, 0),
            Decimal::new(12, 2),
            12,
            Currency::USD,
        ).unwrap();

        loan.set_status(LoanStatus::Active);
        let payment_amount = Decimal::new(100, 0);
        let payment_date = Utc::now();

        let result = loan.make_payment(payment_amount, payment_date);
        assert!(result.is_ok());

        assert_eq!(loan.outstanding_balance, Decimal::new(900, 0));
        assert_eq!(loan.total_paid, payment_amount);
        assert_eq!(loan.payments_made, 1);
        assert!(loan.last_payment_date.is_some());
    }

    #[test]
    fn test_payment_frequency() {
        assert_eq!(PaymentFrequency::Monthly.payments_per_year(), 12);
        assert_eq!(PaymentFrequency::BiWeekly.payments_per_year(), 26);
        assert_eq!(PaymentFrequency::Weekly.payments_per_year(), 52);
    }

    #[test]
    fn test_loan_progress() {
        let mut loan = Loan::new(
            LoanType::Personal,
            Decimal::new(1000, 0),
            Decimal::new(12, 2),
            12,
            Currency::USD,
        ).unwrap();

        loan.outstanding_balance = Decimal::new(600, 0); // 40% paid off
        let progress = loan.payment_progress();
        assert_eq!(progress, Decimal::new(40, 0)); // 40%
    }
}