use iced::{Color, Element, Length};
use iced::widget::{button, column, row, text, Space};
use crate::ui::widgets::SummaryCard;
use crate::ui::widgets::common::action_button;
use crate::ui::{PRIMARY_COLOR, SUCCESS_COLOR, WARNING_COLOR, SECONDARY_COLOR, ERROR_COLOR, SPACING_LARGE, SPACING_MEDIUM};
use crate::{Message, Account, Transaction, Loan, AccountStatus, TransactionStatus, TransactionType, LoanStatus};

pub struct ReportsScreen;

impl ReportsScreen {
    pub fn view<'a>(
        accounts: &[Account],
        transactions: &[Transaction],
        loans: &[Loan],
    ) -> Element<'a, Message> {
        column![
            Self::header(),
            Space::with_height(Length::Fixed(SPACING_MEDIUM as f32)),
            Self::executive_summary(accounts, transactions, loans),
            Space::with_height(Length::Fixed(SPACING_LARGE as f32)),
            Self::detailed_reports(accounts, transactions, loans),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn header<'a>() -> Element<'a, Message> {
        row![
            column![
                text("Financial Reports & Analytics")
                    .size(32)
                    .style(PRIMARY_COLOR),
                text("Comprehensive analysis of your financial portfolio and performance")
                    .size(16)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
            ],
            Space::with_width(Length::Fill),
            row![
                action_button(
                    "Export PDF",
                    Message::ExportReportPDF,
                    PRIMARY_COLOR
                ),
                Space::with_width(Length::Fixed(10.0)),
                action_button(
                    "Email Report",
                    Message::EmailReport,
                    SUCCESS_COLOR
                ),
            ]
            .spacing(10)
        ]
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn executive_summary<'a>(
        accounts: &[Account],
        transactions: &[Transaction],
        loans: &[Loan]
    ) -> Element<'a, Message> {
        let total_accounts = accounts.len();
        let active_accounts = accounts.iter().filter(|a| a.status == AccountStatus::Active).count();
        let total_transactions = transactions.len();
        let completed_transactions = transactions.iter()
            .filter(|t| t.status == TransactionStatus::Completed)
            .count();
        let active_loans = loans.iter().filter(|l| l.status == LoanStatus::Active).count();

        // Calculate key financial metrics (USD only for simplicity)
        let total_assets = accounts.iter()
            .filter(|a| a.currency == crate::Currency::USD &&
                       a.account_type == crate::AccountType::Asset &&
                       a.status == AccountStatus::Active)
            .map(|a| a.balance)
            .sum::<rust_decimal::Decimal>();

        let total_income = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Income &&
                       t.currency == crate::Currency::USD &&
                       t.status == TransactionStatus::Completed)
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>();

        let total_expenses = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Expense &&
                       t.currency == crate::Currency::USD &&
                       t.status == TransactionStatus::Completed)
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>();

        let total_debt = loans.iter()
            .filter(|l| l.currency == crate::Currency::USD && l.status == LoanStatus::Active)
            .map(|l| l.principal_amount)
            .sum::<rust_decimal::Decimal>();

        let net_worth = total_assets - total_debt;
        let net_income = total_income - total_expenses;

        column![
            text("Executive Summary")
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),

            // First row - Account and transaction counts
            row![
                SummaryCard::new("Accounts", &total_accounts.to_string(), PRIMARY_COLOR),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new("Active Accounts", &active_accounts.to_string(), SUCCESS_COLOR),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new("Transactions", &total_transactions.to_string(), WARNING_COLOR),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new("Active Loans", &active_loans.to_string(), SECONDARY_COLOR),
            ],

            Space::with_height(Length::Fixed(15.0)),

            // Second row - Financial metrics
            row![
                SummaryCard::new(
                    "Total Assets (USD)",
                    &format!("{:.2}", total_assets),
                    SUCCESS_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Total Debt (USD)",
                    &format!("{:.2}", total_debt),
                    ERROR_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Net Worth (USD)",
                    &format!("{:.2}", net_worth),
                    if net_worth >= rust_decimal::Decimal::ZERO { SUCCESS_COLOR } else { ERROR_COLOR }
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Net Income (USD)",
                    &format!("{:.2}", net_income),
                    if net_income >= rust_decimal::Decimal::ZERO { SUCCESS_COLOR } else { ERROR_COLOR }
                ),
            ],
        ]
        .spacing(10)
        .into()
    }

    fn detailed_reports<'a>(
        accounts: &[Account],
        transactions: &[Transaction],
        loans: &[Loan]
    ) -> Element<'a, Message> {
        column![
            text("Detailed Analysis")
                .size(20)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),

            row![
                Self::account_breakdown(accounts),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                Self::transaction_analysis(transactions),
            ],

            Space::with_height(Length::Fixed(SPACING_MEDIUM as f32)),

            row![
                Self::loan_portfolio_analysis(loans),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                Self::financial_health_metrics(accounts, transactions, loans),
            ]
        ]
        .spacing(10)
        .into()
    }

    fn account_breakdown<'a>(accounts: &[Account]) -> Element<'a, Message> {
        let asset_accounts = accounts.iter().filter(|a| a.account_type == crate::AccountType::Asset).count();
        let liability_accounts = accounts.iter().filter(|a| a.account_type == crate::AccountType::Liability).count();
        let equity_accounts = accounts.iter().filter(|a| a.account_type == crate::AccountType::Equity).count();
        let revenue_accounts = accounts.iter().filter(|a| a.account_type == crate::AccountType::Revenue).count();
        let expense_accounts = accounts.iter().filter(|a| a.account_type == crate::AccountType::Expense).count();

        Self::report_section("Account Breakdown", vec![
            format!("Asset Accounts: {}", asset_accounts),
            format!("Liability Accounts: {}", liability_accounts),
            format!("Equity Accounts: {}", equity_accounts),
            format!("Revenue Accounts: {}", revenue_accounts),
            format!("Expense Accounts: {}", expense_accounts),
        ])
    }

    fn transaction_analysis<'a>(transactions: &[Transaction]) -> Element<'a, Message> {
        let income_count = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Income).count();
        let expense_count = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Expense).count();
        let transfer_count = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Transfer).count();
        let investment_count = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Investment).count();

        let avg_transaction_amount = if !transactions.is_empty() {
            transactions.iter()
                .filter(|t| t.currency == crate::Currency::USD)
                .map(|t| t.amount)
                .sum::<rust_decimal::Decimal>() / rust_decimal::Decimal::from(transactions.len())
        } else {
            rust_decimal::Decimal::ZERO
        };

        Self::report_section("Transaction Analysis", vec![
            format!("Income Transactions: {}", income_count),
            format!("Expense Transactions: {}", expense_count),
            format!("Transfer Transactions: {}", transfer_count),
            format!("Investment Transactions: {}", investment_count),
            format!("Avg Transaction (USD): {:.2}", avg_transaction_amount),
        ])
    }

    fn loan_portfolio_analysis<'a>(loans: &[Loan]) -> Element<'a, Message> {
        let personal_loans = loans.iter().filter(|l| l.loan_type == crate::LoanType::Personal).count();
        let mortgage_loans = loans.iter().filter(|l| l.loan_type == crate::LoanType::Mortgage).count();
        let auto_loans = loans.iter().filter(|l| l.loan_type == crate::LoanType::Auto).count();
        let business_loans = loans.iter().filter(|l| l.loan_type == crate::LoanType::Business).count();

        let avg_interest_rate = if !loans.is_empty() {
            loans.iter()
                .map(|l| l.interest_rate)
                .sum::<rust_decimal::Decimal>() / rust_decimal::Decimal::from(loans.len())
        } else {
            rust_decimal::Decimal::ZERO
        };

        Self::report_section("Loan Portfolio", vec![
            format!("Personal Loans: {}", personal_loans),
            format!("Mortgage Loans: {}", mortgage_loans),
            format!("Auto Loans: {}", auto_loans),
            format!("Business Loans: {}", business_loans),
            format!("Avg Interest Rate: {:.2}%", avg_interest_rate),
        ])
    }

    fn financial_health_metrics<'a>(
        accounts: &[Account],
        transactions: &[Transaction],
        loans: &[Loan]
    ) -> Element<'a, Message> {
        // Calculate some basic financial health metrics
        let total_assets = accounts.iter()
            .filter(|a| a.currency == crate::Currency::USD &&
                       a.account_type == crate::AccountType::Asset &&
                       a.status == AccountStatus::Active)
            .map(|a| a.balance)
            .sum::<rust_decimal::Decimal>();

        let total_debt = loans.iter()
            .filter(|l| l.currency == crate::Currency::USD && l.status == LoanStatus::Active)
            .map(|l| l.principal_amount)
            .sum::<rust_decimal::Decimal>();

        let debt_to_asset_ratio = if total_assets > rust_decimal::Decimal::ZERO {
            (total_debt / total_assets * rust_decimal::Decimal::from(100))
        } else {
            rust_decimal::Decimal::ZERO
        };

        let monthly_income = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Income &&
                       t.currency == crate::Currency::USD &&
                       t.status == TransactionStatus::Completed)
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>() / rust_decimal::Decimal::from(12); // Simplified monthly average

        let monthly_expenses = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Expense &&
                       t.currency == crate::Currency::USD &&
                       t.status == TransactionStatus::Completed)
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>() / rust_decimal::Decimal::from(12);

        let savings_rate = if monthly_income > rust_decimal::Decimal::ZERO {
            ((monthly_income - monthly_expenses) / monthly_income * rust_decimal::Decimal::from(100))
        } else {
            rust_decimal::Decimal::ZERO
        };

        Self::report_section("Financial Health", vec![
            format!("Debt-to-Asset Ratio: {:.1}%", debt_to_asset_ratio),
            format!("Monthly Income (avg): ${:.2}", monthly_income),
            format!("Monthly Expenses (avg): ${:.2}", monthly_expenses),
            format!("Savings Rate: {:.1}%", savings_rate),
            format!("Financial Score: {}", Self::calculate_financial_score(&debt_to_asset_ratio, &savings_rate)),
        ])
    }

    fn report_section<'a>(title: &str, items: Vec<String>) -> Element<'a, Message> {
        let mut content = column![
            text(title).size(18).style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(10.0)),
        ];

        for item in items {
            content = content.push(
                text(format!("• {}", item)).size(14)
            );
        }

        iced::widget::container(content.spacing(8))
            .padding(20)
            .style(|_theme| iced::widget::container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                border: iced::Border::with_radius(8),
                ..iced::widget::container::Appearance::default()
            })
            .into()
    }

    fn calculate_financial_score(debt_ratio: &rust_decimal::Decimal, savings_rate: &rust_decimal::Decimal) -> String {
        let debt_ratio_f64 = debt_ratio.to_string().parse::<f64>().unwrap_or(0.0);
        let savings_rate_f64 = savings_rate.to_string().parse::<f64>().unwrap_or(0.0);

        let mut score = 100.0;

        // Penalize high debt ratio
        if debt_ratio_f64 > 50.0 {
            score -= 30.0;
        } else if debt_ratio_f64 > 30.0 {
            score -= 20.0;
        } else if debt_ratio_f64 > 15.0 {
            score -= 10.0;
        }

        // Reward good savings rate
        if savings_rate_f64 > 20.0 {
            score += 10.0;
        } else if savings_rate_f64 > 10.0 {
            score += 5.0;
        } else if savings_rate_f64 < 0.0 {
            score -= 20.0;
        }

        let final_score = score.max(0.0).min(100.0) as u8;

        match final_score {
            90..=100 => "Excellent (A+)".to_string(),
            80..=89 => "Very Good (A)".to_string(),
            70..=79 => "Good (B)".to_string(),
            60..=69 => "Fair (C)".to_string(),
            50..=59 => "Poor (D)".to_string(),
            _ => "Critical (F)".to_string(),
        }
    }

    pub fn export_summary<'a>(
        accounts: &[Account],
        transactions: &[Transaction],
        loans: &[Loan]
    ) -> Element<'a, Message> {
        column![
            text("Export Options")
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(20.0)),
            row![
                action_button("Export to PDF", Message::ExportReportPDF, PRIMARY_COLOR),
                Space::with_width(Length::Fixed(15.0)),
                action_button("Export to CSV", Message::ExportReportCSV, SUCCESS_COLOR),
                Space::with_width(Length::Fixed(15.0)),
                action_button("Email Report", Message::EmailReport, WARNING_COLOR),
            ],
            Space::with_height(Length::Fixed(20.0)),
            text("Report will include:")
                .size(16),
            text(format!("• {} accounts with detailed balances", accounts.len()))
                .size(14),
            text(format!("• {} transactions with full history", transactions.len()))
                .size(14),
            text(format!("• {} loans with payment schedules", loans.len()))
                .size(14),
            text("• Financial health analysis and recommendations")
                .size(14),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
}