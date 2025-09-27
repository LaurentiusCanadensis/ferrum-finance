use iced::{Color, Element, Length};
use iced::widget::{button, column, row, text, Space};
use crate::ui::widgets::{AccountWidget, SummaryCard};
use crate::ui::widgets::common::action_button;
use crate::ui::{PRIMARY_COLOR, SUCCESS_COLOR, SPACING_LARGE, SPACING_MEDIUM};
use crate::{Message, Account, AccountForm, AccountStatus};

pub struct AccountsScreen;

impl AccountsScreen {
    pub fn view<'a>(
        accounts: &[Account],
        account_form: &AccountForm,
        show_form: bool,
    ) -> Element<'a, Message> {
        column![
            Self::header(),
            Space::with_height(Length::Fixed(SPACING_MEDIUM as f32)),
            Self::dashboard_summary(accounts),
            Space::with_height(Length::Fixed(SPACING_LARGE as f32)),
            if show_form {
                AccountWidget::form_view(account_form)
            } else {
                Self::accounts_list_with_actions(accounts)
            }
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn header<'a>() -> Element<'a, Message> {
        row![
            column![
                text("Account Management")
                    .size(32)
                    .style(PRIMARY_COLOR),
                text("Manage your financial accounts and track balances")
                    .size(16)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
            ],
            Space::with_width(Length::Fill),
            action_button(
                "Add New Account",
                Message::ShowAccountForm,
                SUCCESS_COLOR
            )
        ]
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn dashboard_summary<'a>(accounts: &[Account]) -> Element<'a, Message> {
        let total_accounts = accounts.len();
        let active_accounts = accounts.iter()
            .filter(|a| a.status == AccountStatus::Active)
            .count();
        let suspended_accounts = accounts.iter()
            .filter(|a| a.status == AccountStatus::Suspended)
            .count();

        // Calculate total balance in USD (simplified)
        let total_balance_usd = accounts.iter()
            .filter(|a| a.currency == crate::Currency::USD && a.status == AccountStatus::Active)
            .map(|a| a.balance)
            .sum::<rust_decimal::Decimal>();

        column![
            text("Account Overview")
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),
            row![
                SummaryCard::new(
                    "Total Accounts",
                    &total_accounts.to_string(),
                    PRIMARY_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Active Accounts",
                    &active_accounts.to_string(),
                    SUCCESS_COLOR
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Total Balance (USD)",
                    &format!("{:.2}", total_balance_usd),
                    Color::from_rgb(0.2, 0.6, 0.8)
                ),
                Space::with_width(Length::Fixed(SPACING_MEDIUM as f32)),
                SummaryCard::new(
                    "Suspended",
                    &suspended_accounts.to_string(),
                    Color::from_rgb(0.8, 0.6, 0.2)
                ),
            ]
        ]
        .spacing(10)
        .into()
    }

    fn accounts_list_with_actions<'a>(accounts: &[Account]) -> Element<'a, Message> {
        if accounts.is_empty() {
            return Self::empty_state();
        }

        column![
            row![
                text("Your Accounts")
                    .size(20)
                    .style(PRIMARY_COLOR),
                Space::with_width(Length::Fill),
                button(text("Refresh"))
                    .on_press(Message::RefreshAccounts)
                    .padding([8, 16])
                    .style(|_theme, _status| iced::widget::button::Appearance {
                        background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                        text_color: Color::from_rgb(0.3, 0.3, 0.3),
                        border: iced::Border::with_radius(5),
                        ..iced::widget::button::Appearance::default()
                    })
            ]
            .align_items(iced::Alignment::Center),
            Space::with_height(Length::Fixed(15.0)),
            AccountWidget::account_list(accounts),
        ]
        .spacing(10)
        .into()
    }

    fn empty_state<'a>() -> Element<'a, Message> {
        column![
            Space::with_height(Length::Fixed(40.0)),
            text("No Accounts Yet")
                .size(24)
                .style(Color::from_rgb(0.6, 0.6, 0.6)),
            Space::with_height(Length::Fixed(15.0)),
            text("Create your first account to get started with financial management")
                .size(16)
                .style(Color::from_rgb(0.5, 0.5, 0.5)),
            Space::with_height(Length::Fixed(20.0)),
            action_button(
                "Create Your First Account",
                Message::ShowAccountForm,
                SUCCESS_COLOR
            ),
            Space::with_height(Length::Fixed(40.0)),
        ]
        .align_items(iced::Alignment::Center)
        .spacing(10)
        .into()
    }

    pub fn detailed_view<'a>(
        accounts: &[Account],
        selected_account_id: Option<uuid::Uuid>
    ) -> Element<'a, Message> {
        if let Some(account_id) = selected_account_id {
            if let Some(account) = accounts.iter().find(|a| a.id == account_id) {
                return Self::account_detail_view(account);
            }
        }

        Self::accounts_list_with_actions(accounts)
    }

    fn account_detail_view<'a>(account: &Account) -> Element<'a, Message> {
        let status_color = match account.status {
            AccountStatus::Active => SUCCESS_COLOR,
            AccountStatus::Suspended => Color::from_rgb(0.8, 0.6, 0.2),
            AccountStatus::Closed => Color::from_rgb(0.6, 0.6, 0.6),
            AccountStatus::Pending => Color::from_rgb(0.2, 0.6, 0.8),
        };

        column![
            row![
                button(text("← Back to Accounts"))
                    .on_press(Message::NavigateTo(crate::Screen::Accounts))
                    .style(|_theme, _status| iced::widget::button::Appearance {
                        background: Some(iced::Background::Color(Color::from_rgb(0.9, 0.9, 0.9))),
                        text_color: PRIMARY_COLOR,
                        border: iced::Border::with_radius(5),
                        ..iced::widget::button::Appearance::default()
                    }),
                Space::with_width(Length::Fill),
            ],
            Space::with_height(Length::Fixed(20.0)),
            text(&account.name)
                .size(28)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(10.0)),
            text(format!("{:.2} {}", account.balance, account.currency))
                .size(36)
                .style(SUCCESS_COLOR),
            Space::with_height(Length::Fixed(20.0)),
            row![
                column![
                    text("Account Type").size(14).style(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(format!("{:?}", account.account_type)).size(16),
                ].spacing(5),
                Space::with_width(Length::Fixed(40.0)),
                column![
                    text("Status").size(14).style(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(format!("{:?}", account.status)).size(16).style(status_color),
                ].spacing(5),
                Space::with_width(Length::Fixed(40.0)),
                column![
                    text("Currency").size(14).style(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(account.currency.to_string()).size(16),
                ].spacing(5),
            ],
            Space::with_height(Length::Fixed(20.0)),
            row![
                column![
                    text("Created").size(14).style(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(account.created_at.format("%Y-%m-%d %H:%M").to_string()).size(16),
                ].spacing(5),
                Space::with_width(Length::Fixed(40.0)),
                column![
                    text("Last Updated").size(14).style(Color::from_rgb(0.5, 0.5, 0.5)),
                    text(account.updated_at.format("%Y-%m-%d %H:%M").to_string()).size(16),
                ].spacing(5),
            ],
            Space::with_height(Length::Fixed(30.0)),
            row![
                action_button("Edit Account", Message::EditAccount(account.id), PRIMARY_COLOR),
                Space::with_width(Length::Fixed(15.0)),
                action_button("View Transactions", Message::ViewAccountTransactions(account.id), Color::from_rgb(0.2, 0.6, 0.8)),
            ]
        ]
        .spacing(5)
        .padding(20)
        .into()
    }
}