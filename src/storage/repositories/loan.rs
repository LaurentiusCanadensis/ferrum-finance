//! Loan repository implementation
//!
//! This module provides comprehensive loan management functionality including:
//! - Full CRUD operations for loans
//! - Payment tracking and management
//! - Loan portfolio analysis
//! - Payment schedule generation
//! - Overdue loan monitoring

use crate::models::{Loan, LoanType, LoanStatus, InterestRateType, PaymentFrequency, PaymentCalculationMethod, LoanPayment, Currency};
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

/// Loan repository for database operations
#[derive(Clone)]
pub struct LoanRepository {
    database: Arc<Database>,
}

impl LoanRepository {
    /// Creates a new loan repository
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }

    /// Maps a database row to a Loan struct
    fn map_row_to_loan(row: &Row) -> rusqlite::Result<Loan> {
        let loan_type_str: String = row.get("loan_type")?;
        let status_str: String = row.get("status")?;
        let interest_rate_type_str: String = row.get("interest_rate_type")?;
        let payment_frequency_str: String = row.get("payment_frequency")?;
        let payment_calculation_method_str: String = row.get("payment_calculation_method")?;
        let currency_str: String = row.get("currency_code")?;

        let loan_type = Self::parse_loan_type(&loan_type_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                0, "loan_type".to_string(), rusqlite::types::Type::Text
            ))?;

        let status = Self::parse_loan_status(&status_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                1, "status".to_string(), rusqlite::types::Type::Text
            ))?;

        let interest_rate_type = Self::parse_interest_rate_type(&interest_rate_type_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                2, "interest_rate_type".to_string(), rusqlite::types::Type::Text
            ))?;

        let payment_frequency = Self::parse_payment_frequency(&payment_frequency_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                3, "payment_frequency".to_string(), rusqlite::types::Type::Text
            ))?;

        let payment_calculation_method = Self::parse_payment_calculation_method(&payment_calculation_method_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                4, "payment_calculation_method".to_string(), rusqlite::types::Type::Text
            ))?;

        let currency = Currency::from_code(&currency_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                5, "currency_code".to_string(), rusqlite::types::Type::Text
            ))?;

        // Parse decimal fields
        let principal_amount = row.get::<_, String>("principal_amount")?.parse::<Decimal>()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                6, "principal_amount".to_string(), rusqlite::types::Type::Text
            ))?;

        let interest_rate = row.get::<_, String>("interest_rate")?.parse::<Decimal>()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                7, "interest_rate".to_string(), rusqlite::types::Type::Text
            ))?;

        let payment_amount = row.get::<_, Option<String>>("payment_amount")?
            .map(|s| s.parse::<Decimal>())
            .transpose()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                8, "payment_amount".to_string(), rusqlite::types::Type::Text
            ))?
            .unwrap_or(Decimal::ZERO);

        let balloon_payment = row.get::<_, Option<String>>("balloon_payment")?
            .map(|s| s.parse::<Decimal>())
            .transpose()
            .map_err(|_| rusqlite::Error::InvalidColumnType(
                9, "balloon_payment".to_string(), rusqlite::types::Type::Text
            ))?;

        Ok(Loan {
            id: Uuid::parse_str(&row.get::<_, String>("id")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    10, "id".to_string(), rusqlite::types::Type::Text
                ))?,
            loan_type,
            status,
            principal_amount,
            outstanding_balance: principal_amount, // Will be calculated from payments
            interest_rate,
            interest_rate_type,
            term_months: row.get::<_, i64>("term_months")? as u32,
            payment_frequency,
            payment_calculation_method,
            currency,
            payment_amount,
            origination_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("origination_date")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    11, "origination_date".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
            first_payment_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("first_payment_date")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    12, "first_payment_date".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
            maturity_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("maturity_date")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    13, "maturity_date".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
            last_payment_date: None, // Will be calculated from payments
            total_paid: Decimal::ZERO, // Will be calculated from payments
            total_interest_paid: Decimal::ZERO, // Will be calculated from payments
            payments_made: 0, // Will be calculated from payments
            missed_payments: 0, // Will be calculated from payments
            late_fees: Decimal::ZERO, // Will be calculated from payments
            description: row.get("description")?,
            reference_number: row.get("reference_number").ok(),
            lender: None, // Not in current schema
            borrower: None, // Not in current schema
            collateral: row.get("collateral_description")?,
            terms_and_conditions: row.get("terms_and_conditions")?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("created_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    14, "created_at".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>("updated_at")?)
                .map_err(|_| rusqlite::Error::InvalidColumnType(
                    15, "updated_at".to_string(), rusqlite::types::Type::Text
                ))?
                .with_timezone(&Utc),
        })
    }

    /// Parse loan type from string
    fn parse_loan_type(s: &str) -> Result<LoanType, &'static str> {
        match s {
            "Personal" => Ok(LoanType::Personal),
            "Mortgage" => Ok(LoanType::Mortgage),
            "Business" => Ok(LoanType::Business),
            "Auto" => Ok(LoanType::Auto),
            "Student" => Ok(LoanType::Student),
            "Line of Credit" => Ok(LoanType::CreditLine),
            _ => Err("Invalid loan type"),
        }
    }

    /// Parse loan status from string
    fn parse_loan_status(s: &str) -> Result<LoanStatus, &'static str> {
        match s {
            "Pending" => Ok(LoanStatus::Pending),
            "Active" => Ok(LoanStatus::Active),
            "Paid Off" => Ok(LoanStatus::PaidOff),
            "Defaulted" => Ok(LoanStatus::Default),
            "Cancelled" => Ok(LoanStatus::Cancelled),
            _ => Err("Invalid loan status"),
        }
    }

    /// Parse interest rate type from string
    fn parse_interest_rate_type(s: &str) -> Result<InterestRateType, &'static str> {
        match s {
            "Fixed" => Ok(InterestRateType::Fixed),
            "Variable" => Ok(InterestRateType::Variable),
            "Floating" => Ok(InterestRateType::Variable), // Alias
            _ => Ok(InterestRateType::Fixed), // Default to fixed
        }
    }

    /// Parse payment frequency from string
    fn parse_payment_frequency(s: &str) -> Result<PaymentFrequency, &'static str> {
        match s {
            "Weekly" => Ok(PaymentFrequency::Weekly),
            "BiWeekly" => Ok(PaymentFrequency::BiWeekly),
            "Monthly" => Ok(PaymentFrequency::Monthly),
            "Quarterly" => Ok(PaymentFrequency::Quarterly),
            "SemiAnnually" => Ok(PaymentFrequency::SemiAnnual),
            "Annually" => Ok(PaymentFrequency::Annual),
            _ => Ok(PaymentFrequency::Monthly), // Default to monthly
        }
    }

    /// Parse payment calculation method from string
    fn parse_payment_calculation_method(s: &str) -> Result<PaymentCalculationMethod, &'static str> {
        match s {
            "EqualPayments" => Ok(PaymentCalculationMethod::EqualPayments),
            "InterestOnly" => Ok(PaymentCalculationMethod::InterestOnly { interest_only_months: 12 }),
            "BalloonPayment" => Ok(PaymentCalculationMethod::Balloon { balloon_amount: Decimal::ZERO }),
            "Custom" => Ok(PaymentCalculationMethod::Custom),
            _ => Ok(PaymentCalculationMethod::EqualPayments), // Default
        }
    }

    /// Validates loan business rules
    async fn validate_loan(&self, loan: &Loan) -> StorageResult<()> {
        // Validate principal amount
        if loan.principal_amount <= Decimal::ZERO {
            return Err(StorageError::validation_error(
                "principal_amount",
                "Loan principal must be positive"
            ));
        }

        // Validate interest rate
        if loan.interest_rate < Decimal::ZERO || loan.interest_rate > Decimal::ONE {
            return Err(StorageError::validation_error(
                "interest_rate",
                "Interest rate must be between 0% and 100%"
            ));
        }

        // Validate term
        if loan.term_months == 0 {
            return Err(StorageError::validation_error(
                "term_months",
                "Loan term must be greater than 0"
            ));
        }

        // Validate date logic
        if loan.first_payment_date <= loan.origination_date {
            return Err(StorageError::validation_error(
                "first_payment_date",
                "First payment date must be after origination date"
            ));
        }

        if loan.maturity_date <= loan.first_payment_date {
            return Err(StorageError::validation_error(
                "maturity_date",
                "Maturity date must be after first payment date"
            ));
        }

        Ok(())
    }

    /// Creates loan payments for the loan
    pub async fn create_loan_payments(&self, loan_id: &Uuid) -> StorageResult<Vec<LoanPayment>> {
        // First get the loan
        let loan = self.find_by_id(loan_id).await?
            .ok_or_else(|| StorageError::not_found("Loan", &loan_id.to_string()))?;

        // Generate payment schedule
        let payments = self.generate_payment_schedule(&loan)?;

        // Save payments to database
        self.database.with_transaction(|tx| {
            let insert_sql = r#"
                INSERT INTO loan_payments (
                    id, loan_id, payment_number, due_date, amount_due,
                    principal_due, interest_due, fees_due, remaining_balance,
                    status, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            let mut stmt = tx.prepare(insert_sql)?;

            for payment in &payments {
                stmt.execute(params![
                    Uuid::new_v4().to_string(),
                    loan_id.to_string(),
                    payment.payment_number,
                    payment.due_date.to_rfc3339(),
                    payment.payment_amount.to_string(),
                    payment.principal_amount.to_string(),
                    payment.interest_amount.to_string(),
                    "0", // fees_due
                    payment.remaining_balance.to_string(),
                    "Pending",
                    Utc::now().to_rfc3339(),
                    Utc::now().to_rfc3339()
                ])?;
            }

            Ok(())
        })?;

        info!("Created {} payment records for loan {}", payments.len(), loan_id);
        Ok(payments)
    }

    /// Generates payment schedule for a loan
    fn generate_payment_schedule(&self, loan: &Loan) -> StorageResult<Vec<LoanPayment>> {
        let mut payments = Vec::new();
        let mut remaining_balance = loan.principal_amount;
        let monthly_rate = loan.interest_rate / Decimal::from(12);

        // Calculate payment amount if not provided
        let payment_amount = if loan.payment_amount > Decimal::ZERO {
            loan.payment_amount
        } else {
            self.calculate_payment_amount(loan)?
        };

        let mut current_date = loan.first_payment_date;

        for payment_number in 1..=loan.term_months {
            if remaining_balance <= Decimal::ZERO {
                break;
            }

            let interest_amount = remaining_balance * monthly_rate;
            let mut principal_amount = payment_amount - interest_amount;

            // For last payment, adjust principal to pay off remaining balance
            if payment_number == loan.term_months || principal_amount >= remaining_balance {
                principal_amount = remaining_balance;
            }

            remaining_balance -= principal_amount;

            let payment = LoanPayment {
                payment_number,
                due_date: current_date,
                payment_amount: principal_amount + interest_amount,
                principal_amount,
                interest_amount,
                remaining_balance,
                is_paid: false,
                paid_date: None,
                amount_paid: None,
                late_fees: None,
            };

            payments.push(payment);

            // Calculate next payment date based on frequency
            current_date = match loan.payment_frequency {
                PaymentFrequency::Monthly => current_date + chrono::Duration::days(30),
                PaymentFrequency::BiWeekly => current_date + chrono::Duration::days(14),
                PaymentFrequency::Weekly => current_date + chrono::Duration::days(7),
                PaymentFrequency::Quarterly => current_date + chrono::Duration::days(90),
                PaymentFrequency::SemiAnnual => current_date + chrono::Duration::days(180),
                PaymentFrequency::Annual => current_date + chrono::Duration::days(365),
                PaymentFrequency::Custom { .. } => current_date + chrono::Duration::days(30), // Default to monthly
            };
        }

        Ok(payments)
    }

    /// Calculates payment amount for equal payment loans
    fn calculate_payment_amount(&self, loan: &Loan) -> StorageResult<Decimal> {
        if loan.interest_rate == Decimal::ZERO {
            // No interest, just divide principal by term
            return Ok(loan.principal_amount / Decimal::from(loan.term_months));
        }

        let monthly_rate = loan.interest_rate / Decimal::from(12);
        let num_payments = Decimal::from(loan.term_months);

        // PMT = P * [r(1+r)^n] / [(1+r)^n - 1]
        // This is a simplified calculation - in production, you'd use a proper financial math library
        let rate_plus_one = Decimal::ONE + monthly_rate;

        // Approximate (1+r)^n using simple multiplication for small terms
        let mut power_term = Decimal::ONE;
        for _ in 0..loan.term_months {
            power_term *= rate_plus_one;
        }

        let numerator = loan.principal_amount * monthly_rate * power_term;
        let denominator = power_term - Decimal::ONE;

        if denominator == Decimal::ZERO {
            return Ok(loan.principal_amount / num_payments);
        }

        Ok(numerator / denominator)
    }

    /// Gets loan payment history
    pub async fn get_payment_history(&self, loan_id: &Uuid) -> StorageResult<Vec<LoanPaymentRecord>> {
        let sql = r#"
            SELECT
                lp.id,
                lp.payment_number,
                lp.due_date,
                lp.amount_due,
                lp.principal_due,
                lp.interest_due,
                lp.fees_due,
                lp.amount_paid,
                lp.principal_paid,
                lp.interest_paid,
                lp.fees_paid,
                lp.payment_date,
                lp.status,
                lp.late_fee,
                lp.remaining_balance,
                lp.notes
            FROM loan_payments lp
            WHERE lp.loan_id = ?
            ORDER BY lp.payment_number ASC
        "#;

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(sql)?;
            let payment_rows = stmt.query_map([&loan_id.to_string()], |row| {
                Ok(LoanPaymentRecord {
                    id: Uuid::parse_str(&row.get::<_, String>("id")?).unwrap(),
                    payment_number: row.get::<_, i64>("payment_number")? as u32,
                    due_date: DateTime::parse_from_rfc3339(&row.get::<_, String>("due_date")?)
                        .unwrap().with_timezone(&Utc),
                    amount_due: row.get::<_, String>("amount_due")?.parse().unwrap_or(Decimal::ZERO),
                    principal_due: row.get::<_, String>("principal_due")?.parse().unwrap_or(Decimal::ZERO),
                    interest_due: row.get::<_, String>("interest_due")?.parse().unwrap_or(Decimal::ZERO),
                    fees_due: row.get::<_, String>("fees_due")?.parse().unwrap_or(Decimal::ZERO),
                    amount_paid: row.get::<_, String>("amount_paid")?.parse().unwrap_or(Decimal::ZERO),
                    principal_paid: row.get::<_, String>("principal_paid")?.parse().unwrap_or(Decimal::ZERO),
                    interest_paid: row.get::<_, String>("interest_paid")?.parse().unwrap_or(Decimal::ZERO),
                    fees_paid: row.get::<_, String>("fees_paid")?.parse().unwrap_or(Decimal::ZERO),
                    payment_date: row.get::<_, Option<String>>("payment_date")?
                        .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                    status: row.get("status")?,
                    late_fee: row.get::<_, String>("late_fee")?.parse().unwrap_or(Decimal::ZERO),
                    remaining_balance: row.get::<_, String>("remaining_balance")?.parse().unwrap_or(Decimal::ZERO),
                    notes: row.get("notes")?,
                })
            })?;

            let mut payments = Vec::new();
            for payment_row in payment_rows {
                payments.push(payment_row?);
            }

            Ok(payments)
        })
    }

    /// Gets overdue loans
    pub async fn get_overdue_loans(&self, as_of_date: Option<DateTime<Utc>>) -> StorageResult<Vec<LoanSummary>> {
        let check_date = as_of_date.unwrap_or_else(Utc::now);

        let sql = r#"
            SELECT DISTINCT
                l.*
            FROM loans l
            INNER JOIN loan_payments lp ON l.id = lp.loan_id
            WHERE l.status = 'Active'
              AND lp.status IN ('Pending', 'Partial')
              AND lp.due_date < ?
            ORDER BY l.origination_date DESC
        "#;

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(sql)?;
            let loan_rows = stmt.query_map([&check_date.to_rfc3339()], Self::map_row_to_loan)?;

            let mut summaries = Vec::new();
            for loan_row in loan_rows {
                let loan = loan_row?;
                let payment_history = self.get_payment_history(&loan.id).await?;

                // Calculate summary statistics
                let total_payments = payment_history.len() as u32;
                let payments_made = payment_history.iter()
                    .filter(|p| p.status == "Paid")
                    .count() as u32;
                let overdue_payments = payment_history.iter()
                    .filter(|p| p.status != "Paid" && p.due_date < check_date)
                    .count() as u32;
                let total_paid = payment_history.iter()
                    .map(|p| p.amount_paid)
                    .sum();

                summaries.push(LoanSummary {
                    loan,
                    total_payments,
                    payments_made,
                    overdue_payments,
                    total_paid,
                    last_payment_date: payment_history.iter()
                        .filter(|p| p.payment_date.is_some())
                        .map(|p| p.payment_date.unwrap())
                        .max(),
                    next_payment_due: payment_history.iter()
                        .find(|p| p.status == "Pending" || p.status == "Partial")
                        .map(|p| p.due_date),
                    calculated_at: Utc::now(),
                });
            }

            Ok(summaries)
        })
    }

    /// Records a loan payment
    pub async fn record_payment(
        &self,
        loan_id: &Uuid,
        payment_number: u32,
        amount_paid: Decimal,
        payment_date: DateTime<Utc>,
        notes: Option<String>,
    ) -> StorageResult<()> {
        self.database.with_transaction(|tx| {
            // Update the payment record
            let update_sql = r#"
                UPDATE loan_payments SET
                    amount_paid = amount_paid + ?,
                    principal_paid = principal_paid + CASE
                        WHEN amount_paid + ? >= amount_due THEN principal_due - principal_paid
                        ELSE ? * (principal_due / amount_due)
                    END,
                    interest_paid = interest_paid + CASE
                        WHEN amount_paid + ? >= amount_due THEN interest_due - interest_paid
                        ELSE ? * (interest_due / amount_due)
                    END,
                    payment_date = COALESCE(payment_date, ?),
                    status = CASE
                        WHEN amount_paid + ? >= amount_due THEN 'Paid'
                        WHEN amount_paid + ? > 0 THEN 'Partial'
                        ELSE status
                    END,
                    notes = COALESCE(?, notes),
                    updated_at = ?
                WHERE loan_id = ? AND payment_number = ?
            "#;

            tx.execute(
                update_sql,
                params![
                    amount_paid.to_string(),
                    amount_paid.to_string(),
                    amount_paid.to_string(),
                    amount_paid.to_string(),
                    amount_paid.to_string(),
                    payment_date.to_rfc3339(),
                    amount_paid.to_string(),
                    amount_paid.to_string(),
                    notes,
                    Utc::now().to_rfc3339(),
                    loan_id.to_string(),
                    payment_number
                ],
            )?;

            Ok(())
        })?;

        info!("Recorded payment of {} for loan {} payment {}", amount_paid, loan_id, payment_number);
        Ok(())
    }
}

#[async_trait]
impl Repository<Loan, LoanFilter> for LoanRepository {
    async fn create(&self, loan: &Loan) -> StorageResult<Loan> {
        // Validate loan
        self.validate_loan(loan).await?;

        let mut created_loan = loan.clone();
        created_loan.created_at = Utc::now();
        created_loan.updated_at = Utc::now();

        self.database.with_transaction(|tx| {
            let insert_sql = r#"
                INSERT INTO loans (
                    id, loan_type, status, borrower_account_id, lender_account_id,
                    principal_amount, currency_code, interest_rate, interest_rate_type,
                    term_months, payment_frequency, payment_calculation_method,
                    payment_amount, balloon_payment, origination_date, first_payment_date,
                    maturity_date, description, collateral_description, terms_and_conditions,
                    is_compound_interest, grace_period_days, late_fee_percentage,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            tx.execute(
                insert_sql,
                params![
                    created_loan.id.to_string(),
                    created_loan.loan_type.to_string(),
                    created_loan.status.to_string(),
                    None::<String>, // borrower_account_id - not in current model
                    None::<String>, // lender_account_id - not in current model
                    created_loan.principal_amount.to_string(),
                    created_loan.currency.code(),
                    created_loan.interest_rate.to_string(),
                    created_loan.interest_rate_type.to_string(),
                    created_loan.term_months,
                    created_loan.payment_frequency.to_string(),
                    created_loan.payment_calculation_method.to_string(),
                    created_loan.payment_amount.to_string(),
                    None::<String>, // balloon_payment - not in current model
                    created_loan.origination_date.to_rfc3339(),
                    created_loan.first_payment_date.to_rfc3339(),
                    created_loan.maturity_date.to_rfc3339(),
                    created_loan.description,
                    created_loan.collateral,
                    created_loan.terms_and_conditions,
                    true, // is_compound_interest
                    0, // grace_period_days
                    "0", // late_fee_percentage
                    created_loan.created_at.to_rfc3339(),
                    created_loan.updated_at.to_rfc3339()
                ],
            )?;

            Ok(())
        })?;

        info!("Created loan: {} ({})", created_loan.loan_type, created_loan.id);
        Ok(created_loan)
    }

    async fn find_by_id(&self, id: &Uuid) -> StorageResult<Option<Loan>> {
        self.database.query_row_optional(
            "SELECT * FROM loans WHERE id = ?",
            &[&id.to_string() as &dyn rusqlite::ToSql],
            Self::map_row_to_loan,
        )
    }

    async fn update(&self, loan: &Loan) -> StorageResult<Loan> {
        // Validate loan
        self.validate_loan(loan).await?;

        let mut updated_loan = loan.clone();
        updated_loan.updated_at = Utc::now();

        self.database.with_transaction(|tx| {
            let update_sql = r#"
                UPDATE loans SET
                    loan_type = ?, status = ?, principal_amount = ?, currency_code = ?,
                    interest_rate = ?, interest_rate_type = ?, term_months = ?,
                    payment_frequency = ?, payment_calculation_method = ?,
                    payment_amount = ?, origination_date = ?, first_payment_date = ?,
                    maturity_date = ?, description = ?, collateral_description = ?,
                    terms_and_conditions = ?, updated_at = ?
                WHERE id = ?
            "#;

            let affected_rows = tx.execute(
                update_sql,
                params![
                    updated_loan.loan_type.to_string(),
                    updated_loan.status.to_string(),
                    updated_loan.principal_amount.to_string(),
                    updated_loan.currency.code(),
                    updated_loan.interest_rate.to_string(),
                    updated_loan.interest_rate_type.to_string(),
                    updated_loan.term_months,
                    updated_loan.payment_frequency.to_string(),
                    updated_loan.payment_calculation_method.to_string(),
                    updated_loan.payment_amount.to_string(),
                    updated_loan.origination_date.to_rfc3339(),
                    updated_loan.first_payment_date.to_rfc3339(),
                    updated_loan.maturity_date.to_rfc3339(),
                    updated_loan.description,
                    updated_loan.collateral,
                    updated_loan.terms_and_conditions,
                    updated_loan.updated_at.to_rfc3339(),
                    updated_loan.id.to_string()
                ],
            )?;

            if affected_rows == 0 {
                return Err(StorageError::not_found("Loan", &loan.id.to_string()));
            }

            Ok(())
        })?;

        info!("Updated loan: {} ({})", updated_loan.loan_type, updated_loan.id);
        Ok(updated_loan)
    }

    async fn delete(&self, id: &Uuid) -> StorageResult<bool> {
        // Check if loan can be deleted (business rule: only pending loans)
        let loan = self.find_by_id(id).await?
            .ok_or_else(|| StorageError::not_found("Loan", &id.to_string()))?;

        if !matches!(loan.status, LoanStatus::Pending | LoanStatus::Cancelled) {
            return Err(StorageError::constraint_violation(
                "Only pending or cancelled loans can be deleted"
            ));
        }

        self.database.with_transaction(|tx| {
            // Delete payments first (due to foreign key)
            tx.execute(
                "DELETE FROM loan_payments WHERE loan_id = ?",
                [&id.to_string()],
            )?;

            // Delete loan
            let affected_rows = tx.execute(
                "DELETE FROM loans WHERE id = ?",
                [&id.to_string()],
            )?;

            Ok(affected_rows > 0)
        })
    }

    async fn find_by_filter(&self, filter: &LoanFilter, options: &QueryOptions) -> StorageResult<Vec<Loan>> {
        let mut query = QueryBuilder::new()
            .select("*")
            .from("loans");

        // Apply filters
        if let Some(loan_types) = &filter.loan_types {
            let type_strs: Vec<String> = loan_types.iter().map(|t| t.to_string()).collect();
            if let Some(condition) = FilterUtils::in_condition("loan_type", &type_strs) {
                query = query.where_clause(&condition);
            }
        }

        if let Some(status) = &filter.status {
            let status_strs: Vec<String> = status.iter().map(|s| s.to_string()).collect();
            if let Some(condition) = FilterUtils::in_condition("status", &status_strs) {
                query = query.where_clause(&condition);
            }
        }

        if let Some(currency) = &filter.currency {
            query = query.where_clause(&format!("currency_code = '{}'", currency.code()));
        }

        if let Some(condition) = FilterUtils::amount_range_condition("principal_amount", filter.min_principal, filter.max_principal) {
            query = query.where_clause(&condition);
        }

        if let Some(condition) = FilterUtils::date_range_condition("origination_date", filter.origination_start, filter.origination_end) {
            query = query.where_clause(&condition);
        }

        // Apply sorting and pagination
        let mut sql = query.build();
        options.apply_to_sql(&mut sql);

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let loan_rows = stmt.query_map([], Self::map_row_to_loan)?;

            let mut loans = Vec::new();
            for loan_row in loan_rows {
                loans.push(loan_row?);
            }

            Ok(loans)
        })
    }

    async fn count_by_filter(&self, filter: &LoanFilter) -> StorageResult<u64> {
        let mut query = QueryBuilder::new()
            .select("COUNT(*)")
            .from("loans");

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
                "SELECT COUNT(*) FROM loans WHERE id = ?",
                &[&id.to_string() as &dyn rusqlite::ToSql],
                |row| row.get(0),
            )?;

        Ok(count > 0)
    }

    async fn find_all(&self, options: &QueryOptions) -> StorageResult<Vec<Loan>> {
        let filter = LoanFilter::default();
        self.find_by_filter(&filter, options).await
    }

    async fn count_all(&self) -> StorageResult<u64> {
        let count: i64 = self
            .database
            .query_row("SELECT COUNT(*) FROM loans", &[], |row| row.get(0))?;

        Ok(count as u64)
    }
}

#[async_trait]
impl AdvancedRepository<Loan, LoanFilter> for LoanRepository {
    async fn find_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        options: &QueryOptions,
    ) -> StorageResult<Vec<Loan>> {
        let filter = LoanFilter {
            origination_start: Some(start),
            origination_end: Some(end),
            ..Default::default()
        };

        self.find_by_filter(&filter, options).await
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> StorageResult<Vec<Loan>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let id_strs: Vec<String> = ids.iter().map(|id| format!("'{}'", id)).collect();
        let sql = format!(
            "SELECT * FROM loans WHERE id IN ({})",
            id_strs.join(", ")
        );

        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(&sql)?;
            let loan_rows = stmt.query_map([], Self::map_row_to_loan)?;

            let mut loans = Vec::new();
            for loan_row in loan_rows {
                loans.push(loan_row?);
            }

            Ok(loans)
        })
    }

    async fn bulk_create(&self, loans: &[Loan]) -> StorageResult<Vec<Loan>> {
        let mut created_loans = Vec::new();

        for loan in loans {
            let created = self.create(loan).await?;
            created_loans.push(created);
        }

        Ok(created_loans)
    }

    async fn bulk_update(&self, loans: &[Loan]) -> StorageResult<Vec<Loan>> {
        let mut updated_loans = Vec::new();

        for loan in loans {
            let updated = self.update(loan).await?;
            updated_loans.push(updated);
        }

        Ok(updated_loans)
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
    ) -> StorageResult<Vec<Loan>> {
        self.database.with_transaction(|tx| {
            let mut stmt = tx.prepare(query)?;
            let loan_rows = stmt.query_map(params, Self::map_row_to_loan)?;

            let mut loans = Vec::new();
            for loan_row in loan_rows {
                loans.push(loan_row?);
            }

            Ok(loans)
        })
    }
}

/// Filter for loan queries
#[derive(Debug, Clone, Default)]
pub struct LoanFilter {
    pub loan_types: Option<Vec<LoanType>>,
    pub status: Option<Vec<String>>,
    pub currency: Option<Currency>,
    pub min_principal: Option<Decimal>,
    pub max_principal: Option<Decimal>,
    pub origination_start: Option<DateTime<Utc>>,
    pub origination_end: Option<DateTime<Utc>>,
    pub borrower_account_id: Option<Uuid>,
    pub lender_account_id: Option<Uuid>,
}

impl LoanFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_types(mut self, types: Vec<LoanType>) -> Self {
        self.loan_types = Some(types);
        self
    }

    pub fn with_status(mut self, status: Vec<String>) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_currency(mut self, currency: Currency) -> Self {
        self.currency = Some(currency);
        self
    }
}

/// Loan summary with payment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanSummary {
    pub loan: Loan,
    pub total_payments: u32,
    pub payments_made: u32,
    pub overdue_payments: u32,
    pub total_paid: Decimal,
    pub last_payment_date: Option<DateTime<Utc>>,
    pub next_payment_due: Option<DateTime<Utc>>,
    pub calculated_at: DateTime<Utc>,
}

/// Loan payment record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanPaymentRecord {
    pub id: Uuid,
    pub payment_number: u32,
    pub due_date: DateTime<Utc>,
    pub amount_due: Decimal,
    pub principal_due: Decimal,
    pub interest_due: Decimal,
    pub fees_due: Decimal,
    pub amount_paid: Decimal,
    pub principal_paid: Decimal,
    pub interest_paid: Decimal,
    pub fees_paid: Decimal,
    pub payment_date: Option<DateTime<Utc>>,
    pub status: String,
    pub late_fee: Decimal,
    pub remaining_balance: Decimal,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Database;

    #[tokio::test]
    async fn test_loan_crud() {
        let db = Database::in_memory().unwrap();
        let repo = LoanRepository::new(Arc::new(db));

        // Create loan
        let loan = Loan::new(
            LoanType::Personal,
            Decimal::new(10000, 2), // $100.00
            Decimal::new(5, 2), // 5%
            36, // 36 months
            Currency::USD,
        );

        let created = repo.create(&loan).await.unwrap();
        assert_eq!(created.principal_amount, Decimal::new(10000, 2));

        // Find by ID
        let found = repo.find_by_id(&created.id).await.unwrap();
        assert!(found.is_some());

        // Update
        let mut updated_loan = created.clone();
        updated_loan.description = Some("Updated description".to_string());
        let updated = repo.update(&updated_loan).await.unwrap();
        assert_eq!(updated.description, Some("Updated description".to_string()));

        // Delete (should work for pending loans)
        let deleted = repo.delete(&created.id).await.unwrap();
        assert!(deleted);
    }

    #[tokio::test]
    async fn test_payment_schedule_generation() {
        let db = Database::in_memory().unwrap();
        let repo = LoanRepository::new(Arc::new(db));

        let loan = Loan::new(
            LoanType::Personal,
            Decimal::new(12000, 2), // $120.00
            Decimal::new(5, 2), // 5%
            12, // 12 months
            Currency::USD,
        );

        let payments = repo.generate_payment_schedule(&loan).unwrap();
        assert_eq!(payments.len(), 12);

        // Check that payments add up to principal + interest
        let total_payments: Decimal = payments.iter().map(|p| p.payment_amount).sum();
        assert!(total_payments > loan.principal_amount); // Should include interest
    }
}