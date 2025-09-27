use iced::{Color, Element, Length};
use iced::widget::{column, pick_list, text, text_input, Space};
use crate::ui::widgets::common::{form_section, action_button};
use crate::ui::{WARNING_COLOR, ERROR_COLOR, SPACING_SMALL, LIGHT_TEXT_COLOR};
use crate::{Message, Transaction, TransactionForm, TransactionType, Currency, TransactionStatus};

pub struct TransactionWidget;

impl TransactionWidget {
    pub fn form_view<'a>(transaction_form: &TransactionForm) -> Element<'a, Message> {
        let mut form_content = column![];

        // Transaction type selection
        form_content = form_content.push(
            column![
                text("Transaction Type").size(14),
                pick_list(
                    vec![TransactionType::Income, TransactionType::Expense,
                         TransactionType::Transfer, TransactionType::Investment,
                         TransactionType::Refund],
                    transaction_form.transaction_type,
                    Message::TransactionTypeSelected,
                ).padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Amount input
        form_content = form_content.push(
            column![
                text("Amount").size(14),
                text_input("0.00", &transaction_form.amount)
                    .on_input(Message::TransactionAmountChanged)
                    .padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Currency selection
        form_content = form_content.push(
            column![
                text("Currency").size(14),
                pick_list(
                    vec![Currency::USD, Currency::EUR, Currency::GBP, Currency::JPY,
                         Currency::CAD, Currency::AUD, Currency::CHF, Currency::CNY,
                         Currency::INR, Currency::BRL],
                    transaction_form.currency,
                    Message::TransactionCurrencySelected,
                ).padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Description input
        form_content = form_content.push(
            column![
                text("Description").size(14),
                text_input("Enter description", &transaction_form.description)
                    .on_input(Message::TransactionDescriptionChanged)
                    .padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Account ID input (simplified for now)
        form_content = form_content.push(
            column![
                text("Account ID").size(14),
                text_input("Account ID (optional)", &transaction_form.account_id)
                    .on_input(Message::TransactionAccountSelected)
                    .padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Error messages
        if !transaction_form.errors.is_empty() {
            for error in &transaction_form.errors {
                form_content = form_content.push(
                    text(error).size(12).style(ERROR_COLOR)
                );
            }
            form_content = form_content.push(Space::with_height(Length::Fixed(10.0)));
        }

        // Submit button
        form_content = form_content.push(
            action_button("Create Transaction", Message::CreateTransaction, WARNING_COLOR)
        );

        form_section("Create New Transaction", form_content.spacing(SPACING_SMALL).into())
    }

    pub fn transaction_list<'a>(transactions: &[Transaction]) -> Element<'a, Message> {
        if transactions.is_empty() {
            return column![
                text("No transactions recorded yet").size(16),
                text("Use the form above to create your first transaction").size(14)
                    .style(LIGHT_TEXT_COLOR),
            ].spacing(10).into();
        }

        let mut list = column![
            text("Recent Transactions").size(20),
            Space::with_height(Length::Fixed(10.0)),
        ];

        // Show most recent transactions first
        let recent_transactions = transactions.iter().rev().take(10);

        for transaction in recent_transactions {
            let transaction_item = Self::transaction_item(transaction);
            list = list.push(transaction_item);
            list = list.push(Space::with_height(Length::Fixed(8.0)));
        }

        if transactions.len() > 10 {
            list = list.push(
                text(format!("And {} more transactions...", transactions.len() - 10))
                    .size(12)
                    .style(LIGHT_TEXT_COLOR)
            );
        }

        list.spacing(5).into()
    }

    fn transaction_item<'a>(transaction: &Transaction) -> Element<'a, Message> {
        let type_color = match transaction.transaction_type {
            TransactionType::Income => Color::from_rgb(0.2, 0.6, 0.2),
            TransactionType::Expense => Color::from_rgb(0.8, 0.2, 0.2),
            TransactionType::Transfer => Color::from_rgb(0.2, 0.4, 0.8),
            TransactionType::Investment => Color::from_rgb(0.6, 0.2, 0.8),
            TransactionType::Refund => Color::from_rgb(0.8, 0.6, 0.2),
        };

        let status_text = match transaction.status {
            TransactionStatus::Pending => "Pending",
            TransactionStatus::Completed => "Completed",
            TransactionStatus::Failed => "Failed",
            TransactionStatus::Cancelled => "Cancelled",
        };

        let transaction_info = column![
            text(&transaction.description).size(14),
            text(format!("{:?} • {}", transaction.transaction_type, status_text))
                .size(12)
                .style(LIGHT_TEXT_COLOR),
            text(transaction.created_at.format("%Y-%m-%d %H:%M").to_string())
                .size(10)
                .style(LIGHT_TEXT_COLOR),
        ].spacing(2);

        let amount_info = column![
            text(format!("{:.2}", transaction.amount))
                .size(16)
                .style(type_color),
            text(transaction.currency.to_string())
                .size(12)
                .style(LIGHT_TEXT_COLOR),
        ].spacing(2);

        iced::widget::container(
            iced::widget::row![
                transaction_info,
                Space::with_width(Length::Fill),
                amount_info,
            ]
            .padding(15)
            .align_items(iced::Alignment::Center)
        )
        .padding(5)
        .style(|_theme| iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
            border: iced::Border::with_radius(5),
            ..iced::widget::container::Appearance::default()
        })
        .into()
    }

    pub fn transaction_summary<'a>(transactions: &[Transaction]) -> Element<'a, Message> {
        let total_transactions = transactions.len();
        let completed_transactions = transactions.iter()
            .filter(|t| t.status == TransactionStatus::Completed)
            .count();
        let pending_transactions = transactions.iter()
            .filter(|t| t.status == TransactionStatus::Pending)
            .count();

        // Calculate totals by type (for USD only for simplicity)
        let income_total = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Income && t.currency == Currency::USD)
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>();

        let expense_total = transactions.iter()
            .filter(|t| t.transaction_type == TransactionType::Expense && t.currency == Currency::USD)
            .map(|t| t.amount)
            .sum::<rust_decimal::Decimal>();

        column![
            text("Transaction Summary").size(18),
            Space::with_height(Length::Fixed(10.0)),
            text(format!("Total Transactions: {}", total_transactions)).size(14),
            text(format!("Completed: {}", completed_transactions)).size(14),
            text(format!("Pending: {}", pending_transactions)).size(14),
            Space::with_height(Length::Fixed(5.0)),
            text(format!("Income (USD): {:.2}", income_total)).size(14).style(Color::from_rgb(0.2, 0.6, 0.2)),
            text(format!("Expenses (USD): {:.2}", expense_total)).size(14).style(Color::from_rgb(0.8, 0.2, 0.2)),
            text(format!("Net (USD): {:.2}", income_total - expense_total)).size(14),
        ]
        .spacing(5)
        .into()
    }
}