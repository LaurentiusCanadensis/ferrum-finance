use iced::{Color, Element, Length};
use iced::widget::{button, column, row, text, Space};
use crate::ui::widgets::{TransactionWidget, SummaryCard};
use crate::ui::widgets::common::action_button;
use crate::ui::{PRIMARY_COLOR, WARNING_COLOR, SUCCESS_COLOR, ERROR_COLOR, SPACING_LARGE, SPACING_MEDIUM};
use crate::{Message, Transaction, TransactionForm, TransactionType, TransactionStatus};

pub struct TransactionsScreen;

impl TransactionsScreen {
    pub fn view<'a>(
        transactions: &[Transaction],
        transaction_form: &TransactionForm,
        show_form: bool,
    ) -> Element<'a, Message> {
        column![
            Self::header(),
            Space::with_height(Length::Fixed(SPACING_MEDIUM as f32)),
            Self::dashboard_summary(transactions),
            Space::with_height(Length::Fixed(SPACING_LARGE as f32)),
            if show_form {
                TransactionWidget::form_view(transaction_form)
            } else {
                Self::transactions_list_with_actions(transactions)
            }
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn header<'a>() -> Element<'a, Message> {
        row![
            column![
                text("Transaction Management")
                    .size(32)
                    .style(PRIMARY_COLOR),
                text("Track your income, expenses, and financial transactions")
                    .size(16)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
            ],
            Space::with_width(Length::Fill),
            action_button(
                "Add New Transaction",
                Message::ShowTransactionForm,
                WARNING_COLOR
            )
        ]
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn dashboard_summary<'a>(transactions: &[Transaction]) -> Element<'a, Message> {
        let total_transactions = transactions.len();
        let completed_count = transactions.iter()
            .filter(|t| t.status == TransactionStatus::Completed)
            .count();
        let pending_count = transactions.iter()
            .filter(|t| t.status == TransactionStatus::Pending)
            .count();

        // Calculate totals by type (USD only for simplicity)
        let income_total = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Income &&
                       t.currency == crate::Currency::USD &&
                       t.status == TransactionStatus::Completed)
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>();

        let expense_total = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Expense &&
                       t.currency == crate::Currency::USD &&
                       t.status == TransactionStatus::Completed)
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>();

        let net_amount = income_total - expense_total;
        let net_color = if net_amount >= rust_decimal::Decimal::ZERO {
            SUCCESS_COLOR
        } else {
            ERROR_COLOR
        };

        column![
            text("Transaction Overview")
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),
            row![
                SummaryCard::new(
                    "Total Transactions",
                    &total_transactions.to_string(),
                    PRIMARY_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Completed",
                    &completed_count.to_string(),
                    SUCCESS_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Pending",
                    &pending_count.to_string(),
                    WARNING_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Net (USD)",
                    &format!("{:.2}", net_amount),
                    net_color
                ),
            ],
            Space::with_height(Length::Fixed(10.0)),
            row![
                SummaryCard::new(
                    "Income (USD)",
                    &format!("{:.2}", income_total),
                    SUCCESS_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Expenses (USD)",
                    &format!("{:.2}", expense_total),
                    ERROR_COLOR
                ),
                Space::with_width(Length::Fill),
                Space::with_width(Length::Fill),
            ]
        ]
        .spacing(10)
        .into()
    }

    fn transactions_list_with_actions<'a>(transactions: &[Transaction]) -> Element<'a, Message> {
        if transactions.is_empty() {
            return Self::empty_state();
        }

        column![
            row![
                text("Recent Transactions")
                    .size(20)
                    .style(PRIMARY_COLOR),
                Space::with_width(Length::Fill),
                row![
                    button(text("Filter"))
                        .on_press(Message::ShowTransactionFilter)
                        .padding([8, 16])
                        .style(|_theme, _status| iced::widget::button::Appearance {
                            background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                            text_color: Color::from_rgb(0.3, 0.3, 0.3),
                            border: iced::Border::with_radius(5),
                            ..iced::widget::button::Appearance::default()
                        }),
                    Space::with_width(Length::Fixed(10.0)),
                    button(text("Export"))
                        .on_press(Message::ExportTransactions)
                        .padding([8, 16])
                        .style(|_theme, _status| iced::widget::button::Appearance {
                            background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                            text_color: Color::from_rgb(0.3, 0.3, 0.3),
                            border: iced::Border::with_radius(5),
                            ..iced::widget::button::Appearance::default()
                        })
                ]
                .spacing(10)
            ]
            .align_items(iced::Alignment::Center),
            Space::with_height(Length::Fixed(15.0)),
            TransactionWidget::transaction_list(transactions),
        ]
        .spacing(10)
        .into()
    }

    fn empty_state<'a>() -> Element<'a, Message> {
        column![
            Space::with_height(Length::Fixed(40.0)),
            text("No Transactions Yet")
                .size(24)
                .style(Color::from_rgb(0.6, 0.6, 0.6)),
            Space::with_height(Length::Fixed(15.0)),
            text("Record your first transaction to start tracking your finances")
                .size(16)
                .style(Color::from_rgb(0.5, 0.5, 0.5)),
            Space::with_height(Length::Fixed(20.0)),
            action_button(
                "Record Your First Transaction",
                Message::ShowTransactionForm,
                WARNING_COLOR
            ),
            Space::with_height(Length::Fixed(40.0)),
        ]
        .align_items(iced::Alignment::Center)
        .spacing(10)
        .into()
    }

    pub fn transaction_by_type_view<'a>(
        transactions: &[Transaction],
        filter_type: Option<TransactionType>
    ) -> Element<'a, Message> {
        let filtered_transactions: Vec<&Transaction> = if let Some(t_type) = filter_type {
            transactions.iter()
                .filter(|t| t.transaction_type == t_type)
                .collect()
        } else {
            transactions.iter().collect()
        };

        column![
            row![
                button(text("← Back to All Transactions"))
                    .on_press(Message::NavigateTo(crate::Screen::Transactions))
                    .style(|_theme, _status| iced::widget::button::Appearance {
                        background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                        text_color: PRIMARY_COLOR,
                        border: iced::Border::with_radius(5),
                        ..iced::widget::button::Appearance::default()
                    }),
                Space::with_width(Length::Fill),
            ],
            Space::with_height(Length::Fixed(20.0)),
            text(if let Some(t_type) = filter_type {
                format!("{:?} Transactions", t_type)
            } else {
                "All Transactions".to_string()
            })
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),
            Self::transaction_type_summary(&filtered_transactions, filter_type),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn transaction_type_summary<'a>(
        transactions: &[&Transaction],
        transaction_type: Option<TransactionType>
    ) -> Element<'a, Message> {
        let total_amount = transactions.iter()
            .filter(|t| t.currency == crate::Currency::USD) // USD only for simplicity
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>();

        let avg_amount = if !transactions.is_empty() {
            total_amount / rust_decimal::Decimal::from(transactions.len())
        } else {
            rust_decimal::Decimal::ZERO
        };

        let type_color = match transaction_type {
            Some(TransactionType::Income) => SUCCESS_COLOR,
            Some(TransactionType::Expense) => ERROR_COLOR,
            Some(TransactionType::Transfer) => PRIMARY_COLOR,
            Some(TransactionType::Investment) => Color::from_rgb(0.6, 0.2, 0.8),
            Some(TransactionType::Refund) => WARNING_COLOR,
            None => PRIMARY_COLOR,
        };

        column![
            row![
                SummaryCard::new(
                    "Total Count",
                    &transactions.len().to_string(),
                    type_color
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Total Amount (USD)",
                    &format!("{:.2}", total_amount),
                    type_color
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Average (USD)",
                    &format!("{:.2}", avg_amount),
                    type_color
                ),
            ],
            Space::with_height(Length::Fixed(20.0)),
            // Could add a detailed list or chart here
            text("Transaction details would be displayed here")
                .size(14)
                .style(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(10)
        .into()
    }
}