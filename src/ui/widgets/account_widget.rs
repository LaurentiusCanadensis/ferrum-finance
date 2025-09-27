use iced::{Color, Element, Length};
use iced::widget::{button, column, pick_list, text, text_input, Space};
use crate::ui::widgets::common::{form_section, action_button};
use crate::ui::{SUCCESS_COLOR, ERROR_COLOR, SPACING_SMALL};
use crate::messages::Message;
use crate::types::{Account, AccountForm, AccountType, Currency, AccountStatus};

pub struct AccountWidget;

impl AccountWidget {
    pub fn form_view<'a>(account_form: &AccountForm) -> Element<'a, Message> {
        let mut form_content = column![];

        // Account name input
        form_content = form_content.push(
            column![
                text("Account Name").size(14),
                text_input("Enter account name", &account_form.name)
                    .on_input(Message::AccountNameChanged)
                    .padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Account type selection
        form_content = form_content.push(
            column![
                text("Account Type").size(14),
                pick_list(
                    vec![AccountType::Asset, AccountType::Liability, AccountType::Equity,
                         AccountType::Revenue, AccountType::Expense],
                    account_form.account_type,
                    Message::AccountTypeSelected,
                ).padding(10),
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
                    account_form.currency,
                    Message::AccountCurrencySelected,
                ).padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Initial balance input
        form_content = form_content.push(
            column![
                text("Initial Balance").size(14),
                text_input("0.00", &account_form.initial_balance)
                    .on_input(Message::AccountInitialBalanceChanged)
                    .padding(10),
            ].spacing(SPACING_SMALL as u16 / 2)
        );

        // Error messages
        if !account_form.errors.is_empty() {
            for error in &account_form.errors {
                form_content = form_content.push(
                    text(error).size(12).style(ERROR_COLOR)
                );
            }
            form_content = form_content.push(Space::with_height(Length::Fixed(10.0)));
        }

        // Submit button
        form_content = form_content.push(
            action_button("Create Account", Message::CreateAccount, SUCCESS_COLOR)
        );

        form_section("Create New Account", form_content.spacing(SPACING_SMALL).into())
    }

    pub fn account_list<'a>(accounts: &[Account]) -> Element<'a, Message> {
        if accounts.is_empty() {
            return column![
                text("No accounts created yet").size(16),
                text("Use the form above to create your first account").size(14)
                    .style(Color::from_rgb(0.6, 0.6, 0.6)),
            ].spacing(10).into();
        }

        let mut list = column![
            text("Your Accounts").size(20),
            Space::with_height(Length::Fixed(10.0)),
        ];

        for account in accounts {
            let account_item = Self::account_item(account);
            list = list.push(account_item);
            list = list.push(Space::with_height(Length::Fixed(10.0)));
        }

        list.spacing(5).into()
    }

    fn account_item<'a>(account: &Account) -> Element<'a, Message> {
        let status_color = match account.status {
            AccountStatus::Active => SUCCESS_COLOR,
            AccountStatus::Suspended => Color::from_rgb(0.8, 0.6, 0.2),
            AccountStatus::Closed => Color::from_rgb(0.6, 0.6, 0.6),
            AccountStatus::Pending => Color::from_rgb(0.2, 0.6, 0.8),
        };

        let account_info = column![
            text(&account.name).size(16),
            text(format!("{:?} • {:?}", account.account_type, account.status))
                .size(12)
                .style(Color::from_rgb(0.5, 0.5, 0.5)),
        ].spacing(2);

        let balance_info = column![
            text(format!("{:.2}", account.balance))
                .size(16),
            text(account.currency.to_string())
                .size(12)
                .style(status_color),
        ].spacing(2);

        button(
            iced::widget::row![
                account_info,
                Space::with_width(Length::Fill),
                balance_info,
            ]
            .padding(15)
            .align_items(iced::Alignment::Center)
        )
        .style(|_theme, _status| iced::widget::button::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
            text_color: Color::from_rgb(0.3, 0.3, 0.3),
            border: iced::Border::with_radius(5),
            ..iced::widget::button::Appearance::default()
        })
        .into()
    }

    pub fn account_summary<'a>(accounts: &[Account]) -> Element<'a, Message> {
        let total_accounts = accounts.len();
        let active_accounts = accounts.iter()
            .filter(|a| a.status == AccountStatus::Active)
            .count();
        let total_balance = accounts.iter()
            .filter(|a| a.currency == Currency::USD) // For simplicity, show only USD
            .map(|a| a.balance)
            .sum::<rust_decimal::Decimal>();

        column![
            text("Account Summary").size(18),
            Space::with_height(Length::Fixed(10.0)),
            text(format!("Total Accounts: {}", total_accounts)).size(14),
            text(format!("Active Accounts: {}", active_accounts)).size(14),
            text(format!("Total Balance (USD): {:.2}", total_balance)).size(14),
        ]
        .spacing(5)
        .into()
    }
}