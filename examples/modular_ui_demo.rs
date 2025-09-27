use iced::{Alignment, Application, Color, Element, Length, Settings, Theme};
use iced::widget::{button, column, container, pick_list, row, text, text_input, Image, Space};
use std::time::Duration;
use tokio::time;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// ===== TYPES MODULE =====
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    USD, EUR, GBP, JPY, CAD, AUD, CHF, CNY, INR, BRL,
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::GBP => write!(f, "GBP"),
            Currency::JPY => write!(f, "JPY"),
            Currency::CAD => write!(f, "CAD"),
            Currency::AUD => write!(f, "AUD"),
            Currency::CHF => write!(f, "CHF"),
            Currency::CNY => write!(f, "CNY"),
            Currency::INR => write!(f, "INR"),
            Currency::BRL => write!(f, "BRL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Asset, Liability, Equity, Revenue, Expense,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active, Suspended, Closed, Pending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Income, Expense, Transfer, Investment, Refund,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending, Completed, Failed, Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanType {
    Personal, Mortgage, Auto, Business, Student,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanStatus {
    Active, Paid, Defaulted, Suspended,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Splash,
    Dashboard,
    Accounts,
    Transactions,
    Loans,
    Reports,
}

// Model structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub currency: Currency,
    pub balance: Decimal,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(name: String, account_type: AccountType, currency: Currency, balance: Decimal) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            account_type,
            currency,
            balance,
            status: AccountStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: Decimal,
    pub currency: Currency,
    pub description: String,
    pub account_id: Uuid,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        amount: Decimal,
        currency: Currency,
        description: String,
        account_id: Uuid,
    ) -> Result<Self, String> {
        if amount <= Decimal::ZERO {
            return Err("Amount must be positive".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            transaction_type,
            amount,
            currency,
            description,
            account_id,
            status: TransactionStatus::Pending,
            created_at: Utc::now(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    pub id: Uuid,
    pub loan_type: LoanType,
    pub principal_amount: Decimal,
    pub currency: Currency,
    pub interest_rate: Decimal,
    pub term_months: u32,
    pub status: LoanStatus,
    pub created_at: DateTime<Utc>,
}

impl Loan {
    pub fn new(
        loan_type: LoanType,
        principal_amount: Decimal,
        currency: Currency,
        interest_rate: Decimal,
        term_months: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            loan_type,
            principal_amount,
            currency,
            interest_rate,
            term_months,
            status: LoanStatus::Active,
            created_at: Utc::now(),
        }
    }
}

// Form structs
#[derive(Debug, Clone)]
pub struct AccountForm {
    pub name: String,
    pub account_type: Option<AccountType>,
    pub currency: Option<Currency>,
    pub initial_balance: String,
    pub errors: Vec<String>,
}

impl Default for AccountForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            account_type: None,
            currency: None,
            initial_balance: String::new(),
            errors: Vec::new(),
        }
    }
}

// ===== MESSAGES MODULE =====
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateTo(Screen),
    SplashTimeout,

    // Account form messages
    AccountNameChanged(String),
    AccountTypeSelected(AccountType),
    AccountCurrencySelected(Currency),
    AccountInitialBalanceChanged(String),
    CreateAccount,
    ShowAccountForm,

    // Form messages
    ClearErrors,
    FormSubmitted,
}

// ===== UI CONSTANTS =====
pub const PRIMARY_COLOR: Color = Color::from_rgb(0.2, 0.4, 0.8);
pub const SUCCESS_COLOR: Color = Color::from_rgb(0.2, 0.6, 0.2);
pub const WARNING_COLOR: Color = Color::from_rgb(0.8, 0.4, 0.2);
pub const ERROR_COLOR: Color = Color::from_rgb(0.8, 0.2, 0.2);
pub const SECONDARY_COLOR: Color = Color::from_rgb(0.6, 0.2, 0.8);
pub const BACKGROUND_COLOR: Color = Color::from_rgb(0.98, 0.98, 0.98);
pub const TEXT_COLOR: Color = Color::from_rgb(0.3, 0.3, 0.3);
pub const LIGHT_TEXT_COLOR: Color = Color::from_rgb(0.5, 0.5, 0.5);

// Asset management
const LOGO_1: &[u8] = include_bytes!("../src/assets/ferrum-finance-logo-1.png");
const LOGO_2: &[u8] = include_bytes!("../src/assets/ferrum-finance-logo-2.png");

pub fn get_splash_logo() -> iced::widget::image::Handle {
    iced::widget::image::Handle::from_memory(LOGO_1)
}

pub fn get_app_logo() -> iced::widget::image::Handle {
    iced::widget::image::Handle::from_memory(LOGO_2)
}

// ===== UI WIDGETS =====
pub struct SummaryCard;

impl SummaryCard {
    pub fn new<'a>(title: &str, value: &str, color: Color) -> Element<'a, Message> {
        container(
            column![
                text(title).size(14).style(LIGHT_TEXT_COLOR),
                text(value).size(24).style(color),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        )
        .padding(20)
        .style(|_theme| container::Appearance {
            background: Some(iced::Background::Color(BACKGROUND_COLOR)),
            border: iced::Border::with_radius(8),
            ..container::Appearance::default()
        })
        .into()
    }

    pub fn clickable<'a>(
        title: &str,
        value: &str,
        color: Color,
        message: Message
    ) -> Element<'a, Message> {
        button(
            column![
                text(title).size(14).style(LIGHT_TEXT_COLOR),
                text(value).size(24).style(color),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        )
        .on_press(message)
        .padding(20)
        .style(|_theme, _status| button::Appearance {
            background: Some(iced::Background::Color(BACKGROUND_COLOR)),
            text_color: TEXT_COLOR,
            border: iced::Border::with_radius(8),
            ..button::Appearance::default()
        })
        .into()
    }
}

pub fn navigation_bar<'a>(current_screen: &Screen) -> row::Row<'a, Message> {
    let nav_button = |label: &str, screen: Screen, current: &Screen| {
        let is_active = current == &screen;
        let button_text = if is_active {
            format!("• {}", label)
        } else {
            label.to_string()
        };

        button(text(button_text))
            .on_press(Message::NavigateTo(screen))
            .padding([10, 20])
            .style(if is_active {
                |_theme, _status| button::Appearance {
                    background: Some(iced::Background::Color(PRIMARY_COLOR)),
                    text_color: Color::WHITE,
                    border: iced::Border::with_radius(5),
                    ..button::Appearance::default()
                }
            } else {
                |_theme, _status| button::Appearance {
                    background: Some(iced::Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    text_color: TEXT_COLOR,
                    border: iced::Border::with_radius(5),
                    ..button::Appearance::default()
                }
            })
    };

    row![
        nav_button("Dashboard", Screen::Dashboard, current_screen),
        nav_button("Accounts", Screen::Accounts, current_screen),
        nav_button("Transactions", Screen::Transactions, current_screen),
        nav_button("Loans", Screen::Loans, current_screen),
        nav_button("Reports", Screen::Reports, current_screen),
    ]
    .align_items(Alignment::Center)
    .spacing(10)
    .padding([0, 20, 20, 20])
}

pub fn header_view<'a>() -> row::Row<'a, Message> {
    let logo = Image::new(get_app_logo())
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0));

    row![
        logo,
        Space::with_width(Length::Fixed(15.0)),
        column![
            text("FerrumFinance")
                .size(28)
                .style(PRIMARY_COLOR),
            text("Multi-Currency Financial Management")
                .size(14)
                .style(LIGHT_TEXT_COLOR),
        ]
    ]
    .align_items(Alignment::Center)
    .padding(20)
}

// ===== APP STATE =====
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_screen: Screen,
    pub show_account_form: bool,
    pub account_form: AccountForm,
    pub accounts: Vec<Account>,
    pub transactions: Vec<Transaction>,
    pub loans: Vec<Loan>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Splash,
            show_account_form: false,
            account_form: AccountForm::default(),
            accounts: Vec::new(),
            transactions: Vec::new(),
            loans: Vec::new(),
        }
    }
}

impl AppState {
    pub fn handle_create_account(&mut self) {
        self.account_form.errors.clear();

        // Validate form inputs
        if self.account_form.name.trim().is_empty() {
            self.account_form.errors.push("Account name is required".to_string());
        }

        if self.account_form.account_type.is_none() {
            self.account_form.errors.push("Account type is required".to_string());
        }

        if self.account_form.currency.is_none() {
            self.account_form.errors.push("Currency is required".to_string());
        }

        let balance = match self.account_form.initial_balance.parse::<f64>() {
            Ok(b) if b >= 0.0 => Decimal::try_from(b).unwrap_or(Decimal::ZERO),
            _ => {
                self.account_form.errors.push("Valid initial balance is required".to_string());
                return;
            }
        };

        if !self.account_form.errors.is_empty() {
            return;
        }

        // Create new account
        let account = Account::new(
            self.account_form.name.clone(),
            self.account_form.account_type.unwrap(),
            self.account_form.currency.unwrap(),
            balance,
        );

        self.accounts.push(account);

        // Clear form
        self.account_form = AccountForm::default();
        self.show_account_form = false;
    }
}

// ===== MAIN APPLICATION =====
pub struct ModularUIDemo {
    state: AppState,
}

impl Application for ModularUIDemo {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, iced::Command<Message>) {
        let splash_command = iced::Command::perform(
            async {
                time::sleep(Duration::from_secs(2)).await;
            },
            |_| Message::SplashTimeout,
        );

        (
            ModularUIDemo {
                state: AppState::default(),
            },
            splash_command,
        )
    }

    fn title(&self) -> String {
        "FerrumFinance - Modular UI Demo".to_string()
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.state.current_screen = screen;
                self.state.show_account_form = false;
            }
            Message::SplashTimeout => {
                self.state.current_screen = Screen::Dashboard;
            }
            Message::AccountNameChanged(name) => {
                self.state.account_form.name = name;
                self.state.account_form.errors.clear();
            }
            Message::AccountTypeSelected(account_type) => {
                self.state.account_form.account_type = Some(account_type);
                self.state.account_form.errors.clear();
            }
            Message::AccountCurrencySelected(currency) => {
                self.state.account_form.currency = Some(currency);
                self.state.account_form.errors.clear();
            }
            Message::AccountInitialBalanceChanged(balance) => {
                self.state.account_form.initial_balance = balance;
                self.state.account_form.errors.clear();
            }
            Message::CreateAccount => {
                self.state.handle_create_account();
            }
            Message::ShowAccountForm => {
                self.state.show_account_form = true;
            }
            Message::ClearErrors => {
                self.state.account_form.errors.clear();
            }
            Message::FormSubmitted => {
                // Handle generic form submission if needed
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.state.current_screen {
            Screen::Splash => self.splash_view(),
            _ => self.main_app_view(),
        }
    }
}

impl ModularUIDemo {
    fn splash_view(&self) -> Element<Message> {
        let logo = Image::new(get_splash_logo())
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0));

        let content = column![
            logo,
            Space::with_height(Length::Fixed(30.0)),
            text("FerrumFinance")
                .size(48)
                .style(PRIMARY_COLOR),
            text("Modular UI Architecture Demo")
                .size(18)
                .style(LIGHT_TEXT_COLOR),
        ]
        .align_items(Alignment::Center)
        .spacing(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(|_theme| container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                ..container::Appearance::default()
            })
            .into()
    }

    fn main_app_view(&self) -> Element<Message> {
        let header = header_view();
        let navigation = navigation_bar(&self.state.current_screen);

        let screen_content = match self.state.current_screen {
            Screen::Dashboard => self.dashboard_view(),
            Screen::Accounts => self.accounts_view(),
            Screen::Transactions => self.transactions_view(),
            Screen::Loans => self.loans_view(),
            Screen::Reports => self.reports_view(),
            Screen::Splash => column![], // Should not reach here
        };

        let content = column![
            header,
            navigation,
            screen_content,
        ]
        .spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn dashboard_view(&self) -> Element<Message> {
        column![
            text("Financial Overview")
                .size(32)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(20.0)),

            // Summary cards row
            row![
                SummaryCard::clickable(
                    "Total Accounts",
                    &self.state.accounts.len().to_string(),
                    SUCCESS_COLOR,
                    Message::NavigateTo(Screen::Accounts)
                ),
                Space::with_width(Length::Fixed(20.0)),
                SummaryCard::clickable(
                    "Recent Transactions",
                    &self.state.transactions.len().to_string(),
                    WARNING_COLOR,
                    Message::NavigateTo(Screen::Transactions)
                ),
                Space::with_width(Length::Fixed(20.0)),
                SummaryCard::clickable(
                    "Active Loans",
                    &self.state.loans.len().to_string(),
                    SECONDARY_COLOR,
                    Message::NavigateTo(Screen::Loans)
                ),
            ],

            Space::with_height(Length::Fixed(30.0)),

            // Welcome message
            text("Welcome to the Modular FerrumFinance UI!")
                .size(24)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),
            text("This demonstrates the new dashboard-first navigation:")
                .size(16),
            Space::with_height(Length::Fixed(10.0)),
            text("• Click on summary cards to navigate to detailed views")
                .size(14),
            text("• Each section has its own dashboard with summary statistics")
                .size(14),
            text("• Forms are modular and can be shown/hidden as needed")
                .size(14),
            text("• Clean separation between UI components, state, and business logic")
                .size(14),
        ]
        .spacing(5)
        .padding(20)
        .into()
    }

    fn accounts_view(&self) -> Element<Message> {
        let mut content = column![
            row![
                column![
                    text("Account Management")
                        .size(32)
                        .style(PRIMARY_COLOR),
                    text("Manage your financial accounts and track balances")
                        .size(16)
                        .style(LIGHT_TEXT_COLOR),
                ],
                Space::with_width(Length::Fill),
                button(text("Add New Account"))
                    .on_press(Message::ShowAccountForm)
                    .padding([12, 24])
                    .style(|_theme, _status| button::Appearance {
                        background: Some(iced::Background::Color(SUCCESS_COLOR)),
                        text_color: Color::WHITE,
                        border: iced::Border::with_radius(5),
                        ..button::Appearance::default()
                    })
            ]
            .align_items(Alignment::Center),
            Space::with_height(Length::Fixed(20.0)),

            // Account summary cards
            row![
                SummaryCard::new(
                    "Total Accounts",
                    &self.state.accounts.len().to_string(),
                    PRIMARY_COLOR
                ),
                Space::with_width(Length::Fixed(20.0)),
                SummaryCard::new(
                    "Active Accounts",
                    &self.state.accounts.iter().filter(|a| a.status == AccountStatus::Active).count().to_string(),
                    SUCCESS_COLOR
                ),
                Space::with_width(Length::Fill),
                Space::with_width(Length::Fill),
            ],
            Space::with_height(Length::Fixed(30.0)),
        ];

        if self.state.show_account_form {
            content = content.push(self.account_form_view());
        } else if self.state.accounts.is_empty() {
            content = content.push(
                column![
                    Space::with_height(Length::Fixed(40.0)),
                    text("No Accounts Yet")
                        .size(24)
                        .style(LIGHT_TEXT_COLOR),
                    text("Create your first account to get started!")
                        .size(16)
                        .style(LIGHT_TEXT_COLOR),
                ]
                .align_items(Alignment::Center)
            );
        } else {
            // Show accounts list
            for account in &self.state.accounts {
                content = content.push(
                    container(
                        row![
                            column![
                                text(&account.name).size(16),
                                text(format!("{:?} • {:?}", account.account_type, account.status))
                                    .size(12)
                                    .style(LIGHT_TEXT_COLOR),
                            ],
                            Space::with_width(Length::Fill),
                            column![
                                text(format!("{:.2}", account.balance)).size(16),
                                text(account.currency.to_string())
                                    .size(12)
                                    .style(SUCCESS_COLOR),
                            ]
                        ]
                        .padding(15)
                        .align_items(Alignment::Center)
                    )
                    .padding(10)
                    .style(|_theme| container::Appearance {
                        background: Some(iced::Background::Color(BACKGROUND_COLOR)),
                        border: iced::Border::with_radius(5),
                        ..container::Appearance::default()
                    })
                );
            }
        }

        content.spacing(15).padding(20).into()
    }

    fn account_form_view(&self) -> Element<Message> {
        let mut form_content = column![
            text("Create New Account").size(20).style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(15.0)),

            column![
                text("Account Name").size(14),
                text_input("Enter account name", &self.state.account_form.name)
                    .on_input(Message::AccountNameChanged)
                    .padding(10),
            ].spacing(5),

            column![
                text("Account Type").size(14),
                pick_list(
                    vec![AccountType::Asset, AccountType::Liability, AccountType::Equity,
                         AccountType::Revenue, AccountType::Expense],
                    self.state.account_form.account_type,
                    Message::AccountTypeSelected,
                ).padding(10),
            ].spacing(5),

            column![
                text("Currency").size(14),
                pick_list(
                    vec![Currency::USD, Currency::EUR, Currency::GBP, Currency::JPY,
                         Currency::CAD, Currency::AUD, Currency::CHF],
                    self.state.account_form.currency,
                    Message::AccountCurrencySelected,
                ).padding(10),
            ].spacing(5),

            column![
                text("Initial Balance").size(14),
                text_input("0.00", &self.state.account_form.initial_balance)
                    .on_input(Message::AccountInitialBalanceChanged)
                    .padding(10),
            ].spacing(5),
        ];

        // Error messages
        for error in &self.state.account_form.errors {
            form_content = form_content.push(
                text(error).size(12).style(ERROR_COLOR)
            );
        }

        form_content = form_content.push(Space::with_height(Length::Fixed(15.0)));
        form_content = form_content.push(
            button(text("Create Account"))
                .on_press(Message::CreateAccount)
                .padding([12, 24])
                .style(|_theme, _status| button::Appearance {
                    background: Some(iced::Background::Color(SUCCESS_COLOR)),
                    text_color: Color::WHITE,
                    border: iced::Border::with_radius(5),
                    ..button::Appearance::default()
                })
        );

        container(form_content.spacing(10))
            .padding(20)
            .style(|_theme| container::Appearance {
                background: Some(iced::Background::Color(BACKGROUND_COLOR)),
                border: iced::Border::with_radius(8),
                ..container::Appearance::default()
            })
            .into()
    }

    fn transactions_view(&self) -> Element<Message> {
        column![
            text("Transaction Management")
                .size(32)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(20.0)),
            row![
                SummaryCard::new("Total Transactions", &self.state.transactions.len().to_string(), PRIMARY_COLOR),
                Space::with_width(Length::Fixed(20.0)),
                SummaryCard::new("This Month", "0", WARNING_COLOR),
            ],
            Space::with_height(Length::Fixed(30.0)),
            text("Transaction dashboard would show recent transactions, filtering options, and add new transaction form.")
                .size(16)
                .style(LIGHT_TEXT_COLOR),
        ]
        .spacing(15)
        .padding(20)
        .into()
    }

    fn loans_view(&self) -> Element<Message> {
        column![
            text("Loan Management")
                .size(32)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(20.0)),
            row![
                SummaryCard::new("Active Loans", &self.state.loans.len().to_string(), SECONDARY_COLOR),
                Space::with_width(Length::Fixed(20.0)),
                SummaryCard::new("Portfolio Health", "Good", SUCCESS_COLOR),
            ],
            Space::with_height(Length::Fixed(30.0)),
            text("Loan dashboard would show loan portfolio, payment schedules, and loan calculator.")
                .size(16)
                .style(LIGHT_TEXT_COLOR),
        ]
        .spacing(15)
        .padding(20)
        .into()
    }

    fn reports_view(&self) -> Element<Message> {
        column![
            text("Financial Reports & Analytics")
                .size(32)
                .style(PRIMARY_COLOR),
            Space::with_height(Length::Fixed(20.0)),
            row![
                SummaryCard::new("Net Worth", "$0.00", SUCCESS_COLOR),
                Space::with_width(Length::Fixed(20.0)),
                SummaryCard::new("Monthly Income", "$0.00", SUCCESS_COLOR),
                Space::with_width(Length::Fixed(20.0)),
                SummaryCard::new("Monthly Expenses", "$0.00", ERROR_COLOR),
            ],
            Space::with_height(Length::Fixed(30.0)),
            text("Reports dashboard would show comprehensive financial analysis, charts, and export options.")
                .size(16)
                .style(LIGHT_TEXT_COLOR),
        ]
        .spacing(15)
        .padding(20)
        .into()
    }
}

fn main() -> iced::Result {
    env_logger::init();
    println!("Starting FerrumFinance Modular UI Demo...");

    ModularUIDemo::run(Settings::default())
}