use iced::{Color, Element, Length};
use iced::widget::{column, row, text, Space};
use crate::ui::widgets::common::SummaryCard;
use crate::ui::{PRIMARY_COLOR, SUCCESS_COLOR, WARNING_COLOR, SECONDARY_COLOR, SPACING_MEDIUM};
use crate::messages::Message;
use crate::types::{Account, Transaction, Loan, Screen};

pub struct DashboardWidget;

impl DashboardWidget {
    pub fn new<'a>(
        accounts: &[Account],
        transactions: &[Transaction],
        loans: &[Loan]
    ) -> Element<'a, Message> {
        column![
            text("Financial Overview")
                .size(32)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(SPACING_MEDIUM as f32)),

            // Summary cards row
            row![
                SummaryCard::clickable(
                    "Total Accounts",
                    &accounts.len().to_string(),
                    SUCCESS_COLOR,
                    Message::NavigateTo(Screen::Accounts)
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::clickable(
                    "Recent Transactions",
                    &transactions.len().to_string(),
                    WARNING_COLOR,
                    Message::NavigateTo(Screen::Transactions)
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::clickable(
                    "Active Loans",
                    &loans.len().to_string(),
                    SECONDARY_COLOR,
                    Message::NavigateTo(Screen::Loans)
                ),
            ],

            Space::with_height(Length::Fixed(SPACING_MEDIUM as f32 + 10.0)),

            // Welcome message and features
            Self::welcome_section(),
        ]
        .spacing(15)
        .padding(20)
        .into()
    }

    fn welcome_section<'a>() -> Element<'a, Message> {
        column![
            text("Welcome to FerrumFinance!")
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),
            text("Your comprehensive financial management system provides:")
                .size(16),
            Space::with_height(Length::Fixed(10.0)),
            Self::feature_list(),
        ]
        .spacing(5)
        .into()
    }

    fn feature_list<'a>() -> Element<'a, Message> {
        column![
            text("• Multi-currency account and transaction management").size(14),
            text("• Advanced loan tracking with payment schedules").size(14),
            text("• Real-time exchange rate conversion").size(14),
            text("• Comprehensive financial reporting and analytics").size(14),
            text("• Secure SQLite database storage with audit trails").size(14),
            text("• Professional dashboard interface with quick navigation").size(14),
        ]
        .spacing(8)
        .into()
    }

    // Quick stats section for expanded dashboard
    pub fn quick_stats<'a>(
        accounts: &[Account],
        transactions: &[Transaction],
        loans: &[Loan]
    ) -> Element<'a, Message> {
        let active_accounts = accounts.iter()
            .filter(|a| a.status == crate::types::AccountStatus::Active)
            .count();

        let completed_transactions = transactions.iter()
            .filter(|t| t.status == crate::types::TransactionStatus::Completed)
            .count();

        let active_loans = loans.iter()
            .filter(|l| l.status == crate::types::LoanStatus::Active)
            .count();

        column![
            text("Quick Statistics").size(20).style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(10.0)),
            row![
                SummaryCard::new("Active Accounts", &active_accounts.to_string(), SUCCESS_COLOR),
                Space::with_width(Length::Fixed(15.0)),
                SummaryCard::new("Completed Transactions", &completed_transactions.to_string(), WARNING_COLOR),
                Space::with_width(Length::Fixed(15.0)),
                SummaryCard::new("Active Loans", &active_loans.to_string(), SECONDARY_COLOR),
            ]
        ]
        .spacing(10)
        .into()
    }
}