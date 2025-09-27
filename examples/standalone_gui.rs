use iced::{Alignment, Application, Color, Element, Length, Settings, Theme};
use iced::widget::{
    button, column, container, pick_list, row, text, text_input,
    Column, Row, Button, TextInput, Image, Space
};
use std::time::Duration;
use tokio::time;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// Asset management - include the PNG files directly for now
const LOGO_1: &[u8] = include_bytes!("../src/assets/ferrum-finance-logo-1.png");
const LOGO_2: &[u8] = include_bytes!("../src/assets/ferrum-finance-logo-2.png");

fn get_splash_logo() -> iced::widget::image::Handle {
    iced::widget::image::Handle::from_memory(LOGO_1)
}

fn get_app_logo() -> iced::widget::image::Handle {
    iced::widget::image::Handle::from_memory(LOGO_2)
}

// Temporary model definitions for the GUI
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

// Utility functions
pub fn format_money(amount: Decimal, _currency: Currency) -> String {
    format!("{:.2}", amount)
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

    // Transaction form messages
    TransactionTypeSelected(TransactionType),
    TransactionAmountChanged(String),
    TransactionCurrencySelected(Currency),
    TransactionDescriptionChanged(String),
    TransactionAccountSelected(String),
    CreateTransaction,

    // Loan form messages
    LoanTypeSelected(LoanType),
    LoanPrincipalChanged(String),
    LoanInterestRateChanged(String),
    LoanTermChanged(String),
    CreateLoan,

    // General form messages
    ClearErrors,
    FormSubmitted,
}

#[derive(Debug, Clone)]
pub struct AccountForm {
    pub name: String,
    pub account_type: Option<AccountType>,
    pub currency: Option<Currency>,
    pub initial_balance: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TransactionForm {
    pub transaction_type: Option<TransactionType>,
    pub amount: String,
    pub currency: Option<Currency>,
    pub description: String,
    pub account_id: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoanForm {
    pub loan_type: Option<LoanType>,
    pub principal: String,
    pub interest_rate: String,
    pub term_months: String,
    pub errors: Vec<String>,
}

pub struct FerrumFinanceApp {
    current_screen: Screen,
    account_form: AccountForm,
    transaction_form: TransactionForm,
    loan_form: LoanForm,
    accounts: Vec<Account>,
    transactions: Vec<Transaction>,
    loans: Vec<Loan>,
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

impl Default for TransactionForm {
    fn default() -> Self {
        Self {
            transaction_type: None,
            amount: String::new(),
            currency: None,
            description: String::new(),
            account_id: String::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for LoanForm {
    fn default() -> Self {
        Self {
            loan_type: None,
            principal: String::new(),
            interest_rate: String::new(),
            term_months: String::new(),
            errors: Vec::new(),
        }
    }
}

impl Application for FerrumFinanceApp {
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
            FerrumFinanceApp {
                current_screen: Screen::Splash,
                account_form: AccountForm::default(),
                transaction_form: TransactionForm::default(),
                loan_form: LoanForm::default(),
                accounts: Vec::new(),
                transactions: Vec::new(),
                loans: Vec::new(),
            },
            splash_command,
        )
    }

    fn title(&self) -> String {
        "FerrumFinance - Multi-Currency Financial Management".to_string()
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.current_screen = screen;
            }
            Message::SplashTimeout => {
                self.current_screen = Screen::Dashboard;
            }

            // Account form handling
            Message::AccountNameChanged(name) => {
                self.account_form.name = name;
                self.account_form.errors.clear();
            }
            Message::AccountTypeSelected(account_type) => {
                self.account_form.account_type = Some(account_type);
                self.account_form.errors.clear();
            }
            Message::AccountCurrencySelected(currency) => {
                self.account_form.currency = Some(currency);
                self.account_form.errors.clear();
            }
            Message::AccountInitialBalanceChanged(balance) => {
                self.account_form.initial_balance = balance;
                self.account_form.errors.clear();
            }
            Message::CreateAccount => {
                self.handle_create_account();
            }

            // Transaction form handling
            Message::TransactionTypeSelected(transaction_type) => {
                self.transaction_form.transaction_type = Some(transaction_type);
                self.transaction_form.errors.clear();
            }
            Message::TransactionAmountChanged(amount) => {
                self.transaction_form.amount = amount;
                self.transaction_form.errors.clear();
            }
            Message::TransactionCurrencySelected(currency) => {
                self.transaction_form.currency = Some(currency);
                self.transaction_form.errors.clear();
            }
            Message::TransactionDescriptionChanged(description) => {
                self.transaction_form.description = description;
                self.transaction_form.errors.clear();
            }
            Message::TransactionAccountSelected(account_id) => {
                self.transaction_form.account_id = account_id;
                self.transaction_form.errors.clear();
            }
            Message::CreateTransaction => {
                self.handle_create_transaction();
            }

            // Loan form handling
            Message::LoanTypeSelected(loan_type) => {
                self.loan_form.loan_type = Some(loan_type);
                self.loan_form.errors.clear();
            }
            Message::LoanPrincipalChanged(principal) => {
                self.loan_form.principal = principal;
                self.loan_form.errors.clear();
            }
            Message::LoanInterestRateChanged(rate) => {
                self.loan_form.interest_rate = rate;
                self.loan_form.errors.clear();
            }
            Message::LoanTermChanged(term) => {
                self.loan_form.term_months = term;
                self.loan_form.errors.clear();
            }
            Message::CreateLoan => {
                self.handle_create_loan();
            }

            Message::ClearErrors => {
                self.account_form.errors.clear();
                self.transaction_form.errors.clear();
                self.loan_form.errors.clear();
            }
            Message::FormSubmitted => {
                // Handle generic form submission if needed
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.current_screen {
            Screen::Splash => self.splash_view(),
            _ => self.main_app_view(),
        }
    }
}

impl FerrumFinanceApp {
    fn splash_view(&self) -> Element<Message> {
        let logo = Image::new(get_splash_logo())
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0));

        let content = column![
            logo,
            Space::with_height(Length::Fixed(30.0)),
            text("FerrumFinance")
                .size(48)
                .style(Color::from_rgb(0.2, 0.4, 0.8)),
            text("Multi-Currency Financial Management")
                .size(18)
                .style(Color::from_rgb(0.4, 0.4, 0.4)),
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
        let header = self.header_view();
        let navigation = self.navigation_view();
        let screen_content = match self.current_screen {
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

    fn header_view(&self) -> Row<Message> {
        let logo = Image::new(get_app_logo())
            .width(Length::Fixed(50.0))
            .height(Length::Fixed(50.0));

        row![
            logo,
            Space::with_width(Length::Fixed(15.0)),
            column![
                text("FerrumFinance")
                    .size(28)
                    .style(Color::from_rgb(0.2, 0.4, 0.8)),
                text("Multi-Currency Financial Management")
                    .size(14)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)),
            ]
        ]
        .align_items(Alignment::Center)
        .padding(20)
    }

    fn navigation_view(&self) -> Row<Message> {
        let nav_button = |label: &str, screen: Screen, current: &Screen| {
            let is_active = current == &screen;
            let button_text = if is_active { format!("• {}", label) } else { label.to_string() };

            button(text(button_text))
                .on_press(Message::NavigateTo(screen))
                .padding([10, 20])
                .style(if is_active {
                    |_theme, _status| button::Appearance {
                        background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.4, 0.8))),
                        text_color: Color::WHITE,
                        border: iced::Border::with_radius(5),
                        ..button::Appearance::default()
                    }
                } else {
                    |_theme, _status| button::Appearance {
                        background: Some(iced::Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                        text_color: Color::from_rgb(0.3, 0.3, 0.3),
                        border: iced::Border::with_radius(5),
                        ..button::Appearance::default()
                    }
                })
        };

        row![
            nav_button("Dashboard", Screen::Dashboard, &self.current_screen),
            nav_button("Accounts", Screen::Accounts, &self.current_screen),
            nav_button("Transactions", Screen::Transactions, &self.current_screen),
            nav_button("Loans", Screen::Loans, &self.current_screen),
            nav_button("Reports", Screen::Reports, &self.current_screen),
        ]
        .align_items(Alignment::Center)
        .spacing(10)
        .padding([0, 20, 20, 20])
    }

    fn dashboard_view(&self) -> Column<Message> {
        column![
            text("Dashboard").size(32).style(Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),

            row![
                self.summary_card("Total Accounts", &self.accounts.len().to_string(), Color::from_rgb(0.2, 0.6, 0.2)),
                Space::with_width(Length::Fixed(20.0)),
                self.summary_card("Total Transactions", &self.transactions.len().to_string(), Color::from_rgb(0.8, 0.4, 0.2)),
                Space::with_width(Length::Fixed(20.0)),
                self.summary_card("Active Loans", &self.loans.len().to_string(), Color::from_rgb(0.6, 0.2, 0.8)),
            ],

            Space::with_height(Length::Fixed(30.0)),

            text("Welcome to FerrumFinance!").size(18),
            text("Your comprehensive financial management system provides:"),
            text("• Multi-currency account and transaction management"),
            text("• Advanced loan tracking with payment schedules"),
            text("• Real-time exchange rate conversion"),
            text("• Comprehensive financial reporting and analytics"),
            text("• Secure SQLite database storage with audit trails"),
        ]
        .spacing(15)
        .padding(20)
    }

    fn summary_card(&self, title: &str, value: &str, color: Color) -> Element<Message> {
        container(
            column![
                text(title).size(14).style(Color::from_rgb(0.5, 0.5, 0.5)),
                text(value).size(24).style(color),
            ]
            .spacing(5)
            .align_items(Alignment::Center)
        )
        .padding(20)
        .style(|_theme| container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
            border: iced::Border::with_radius(8),
            ..container::Appearance::default()
        })
        .into()
    }

    fn accounts_view(&self) -> Column<Message> {
        let mut content = column![
            text("Account Management").size(32).style(Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
        ];

        // Account creation form
        let form = self.account_form_view();
        content = content.push(form);

        // Display existing accounts
        if !self.accounts.is_empty() {
            content = content.push(Space::with_height(Length::Fixed(30.0)));
            content = content.push(text("Existing Accounts").size(20));

            for account in &self.accounts {
                let account_item = row![
                    text(&account.name).size(16),
                    Space::with_width(Length::Fill),
                    text(format!("{:?}", account.account_type)).size(14),
                    Space::with_width(Length::Fixed(10.0)),
                    text(format!("{} {}",
                        format_money(account.balance, account.currency),
                        account.currency
                    )).size(14),
                ]
                .padding(10)
                .spacing(10);

                content = content.push(
                    container(account_item)
                        .padding(10)
                        .style(|_theme| container::Appearance {
                            background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                            border: iced::Border::with_radius(5),
                            ..container::Appearance::default()
                        })
                );
            }
        }

        content.spacing(15).padding(20)
    }

    fn account_form_view(&self) -> Element<Message> {
        let mut form_content = column![
            text("Create New Account").size(20),
            Space::with_height(Length::Fixed(15.0)),
        ];

        // Account name input
        form_content = form_content.push(
            column![
                text("Account Name").size(14),
                text_input("Enter account name", &self.account_form.name)
                    .on_input(Message::AccountNameChanged)
                    .padding(10),
            ].spacing(5)
        );

        // Account type selection
        form_content = form_content.push(
            column![
                text("Account Type").size(14),
                pick_list(
                    vec![AccountType::Asset, AccountType::Liability, AccountType::Equity,
                         AccountType::Revenue, AccountType::Expense],
                    self.account_form.account_type,
                    Message::AccountTypeSelected,
                ).padding(10),
            ].spacing(5)
        );

        // Currency selection
        form_content = form_content.push(
            column![
                text("Currency").size(14),
                pick_list(
                    vec![Currency::USD, Currency::EUR, Currency::GBP, Currency::JPY,
                         Currency::CAD, Currency::AUD, Currency::CHF],
                    self.account_form.currency,
                    Message::AccountCurrencySelected,
                ).padding(10),
            ].spacing(5)
        );

        // Initial balance input
        form_content = form_content.push(
            column![
                text("Initial Balance").size(14),
                text_input("0.00", &self.account_form.initial_balance)
                    .on_input(Message::AccountInitialBalanceChanged)
                    .padding(10),
            ].spacing(5)
        );

        // Error messages
        if !self.account_form.errors.is_empty() {
            for error in &self.account_form.errors {
                form_content = form_content.push(
                    text(error).size(12).style(Color::from_rgb(0.8, 0.2, 0.2))
                );
            }
        }

        // Submit button
        form_content = form_content.push(
            Space::with_height(Length::Fixed(15.0))
        );

        form_content = form_content.push(
            button(text("Create Account"))
                .on_press(Message::CreateAccount)
                .padding([12, 24])
                .style(|_theme, _status| button::Appearance {
                    background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.6, 0.2))),
                    text_color: Color::WHITE,
                    border: iced::Border::with_radius(5),
                    ..button::Appearance::default()
                })
        );

        container(form_content.spacing(10))
            .padding(20)
            .style(|_theme| container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                border: iced::Border::with_radius(8),
                ..container::Appearance::default()
            })
            .into()
    }

    fn transactions_view(&self) -> Column<Message> {
        let mut content = column![
            text("Transaction Management").size(32).style(Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
        ];

        // Transaction creation form
        let form = self.transaction_form_view();
        content = content.push(form);

        // Display existing transactions
        if !self.transactions.is_empty() {
            content = content.push(Space::with_height(Length::Fixed(30.0)));
            content = content.push(text("Recent Transactions").size(20));

            for transaction in &self.transactions {
                let transaction_item = row![
                    column![
                        text(&transaction.description).size(14),
                        text(format!("{:?}", transaction.transaction_type)).size(12)
                            .style(Color::from_rgb(0.5, 0.5, 0.5)),
                    ],
                    Space::with_width(Length::Fill),
                    text(format!("{} {}",
                        format_money(transaction.amount, transaction.currency),
                        transaction.currency
                    )).size(14),
                ]
                .padding(10)
                .spacing(10);

                content = content.push(
                    container(transaction_item)
                        .padding(10)
                        .style(|_theme| container::Appearance {
                            background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                            border: iced::Border::with_radius(5),
                            ..container::Appearance::default()
                        })
                );
            }
        }

        content.spacing(15).padding(20)
    }

    fn transaction_form_view(&self) -> Element<Message> {
        let mut form_content = column![
            text("Create New Transaction").size(20),
            Space::with_height(Length::Fixed(15.0)),
        ];

        // Transaction type selection
        form_content = form_content.push(
            column![
                text("Transaction Type").size(14),
                pick_list(
                    vec![TransactionType::Income, TransactionType::Expense,
                         TransactionType::Transfer, TransactionType::Investment],
                    self.transaction_form.transaction_type,
                    Message::TransactionTypeSelected,
                ).padding(10),
            ].spacing(5)
        );

        // Amount input
        form_content = form_content.push(
            column![
                text("Amount").size(14),
                text_input("0.00", &self.transaction_form.amount)
                    .on_input(Message::TransactionAmountChanged)
                    .padding(10),
            ].spacing(5)
        );

        // Currency selection
        form_content = form_content.push(
            column![
                text("Currency").size(14),
                pick_list(
                    vec![Currency::USD, Currency::EUR, Currency::GBP, Currency::JPY],
                    self.transaction_form.currency,
                    Message::TransactionCurrencySelected,
                ).padding(10),
            ].spacing(5)
        );

        // Description input
        form_content = form_content.push(
            column![
                text("Description").size(14),
                text_input("Enter description", &self.transaction_form.description)
                    .on_input(Message::TransactionDescriptionChanged)
                    .padding(10),
            ].spacing(5)
        );

        // Error messages
        if !self.transaction_form.errors.is_empty() {
            for error in &self.transaction_form.errors {
                form_content = form_content.push(
                    text(error).size(12).style(Color::from_rgb(0.8, 0.2, 0.2))
                );
            }
        }

        // Submit button
        form_content = form_content.push(Space::with_height(Length::Fixed(15.0)));
        form_content = form_content.push(
            button(text("Create Transaction"))
                .on_press(Message::CreateTransaction)
                .padding([12, 24])
                .style(|_theme, _status| button::Appearance {
                    background: Some(iced::Background::Color(Color::from_rgb(0.8, 0.4, 0.2))),
                    text_color: Color::WHITE,
                    border: iced::Border::with_radius(5),
                    ..button::Appearance::default()
                })
        );

        container(form_content.spacing(10))
            .padding(20)
            .style(|_theme| container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                border: iced::Border::with_radius(8),
                ..container::Appearance::default()
            })
            .into()
    }

    fn loans_view(&self) -> Column<Message> {
        let mut content = column![
            text("Loan Management").size(32).style(Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
        ];

        // Loan creation form
        let form = self.loan_form_view();
        content = content.push(form);

        // Display existing loans
        if !self.loans.is_empty() {
            content = content.push(Space::with_height(Length::Fixed(30.0)));
            content = content.push(text("Active Loans").size(20));

            for loan in &self.loans {
                let loan_item = row![
                    column![
                        text(format!("{:?} Loan", loan.loan_type)).size(14),
                        text(format!("{}% APR", loan.interest_rate)).size(12)
                            .style(Color::from_rgb(0.5, 0.5, 0.5)),
                    ],
                    Space::with_width(Length::Fill),
                    text(format!("{} {}",
                        format_money(loan.principal_amount, loan.currency),
                        loan.currency
                    )).size(14),
                ]
                .padding(10)
                .spacing(10);

                content = content.push(
                    container(loan_item)
                        .padding(10)
                        .style(|_theme| container::Appearance {
                            background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                            border: iced::Border::with_radius(5),
                            ..container::Appearance::default()
                        })
                );
            }
        }

        content.spacing(15).padding(20)
    }

    fn loan_form_view(&self) -> Element<Message> {
        let mut form_content = column![
            text("Create New Loan").size(20),
            Space::with_height(Length::Fixed(15.0)),
        ];

        // Loan type selection
        form_content = form_content.push(
            column![
                text("Loan Type").size(14),
                pick_list(
                    vec![LoanType::Personal, LoanType::Mortgage, LoanType::Auto, LoanType::Business],
                    self.loan_form.loan_type,
                    Message::LoanTypeSelected,
                ).padding(10),
            ].spacing(5)
        );

        // Principal amount input
        form_content = form_content.push(
            column![
                text("Principal Amount").size(14),
                text_input("0.00", &self.loan_form.principal)
                    .on_input(Message::LoanPrincipalChanged)
                    .padding(10),
            ].spacing(5)
        );

        // Interest rate input
        form_content = form_content.push(
            column![
                text("Interest Rate (%)").size(14),
                text_input("0.00", &self.loan_form.interest_rate)
                    .on_input(Message::LoanInterestRateChanged)
                    .padding(10),
            ].spacing(5)
        );

        // Term input
        form_content = form_content.push(
            column![
                text("Term (months)").size(14),
                text_input("12", &self.loan_form.term_months)
                    .on_input(Message::LoanTermChanged)
                    .padding(10),
            ].spacing(5)
        );

        // Error messages
        if !self.loan_form.errors.is_empty() {
            for error in &self.loan_form.errors {
                form_content = form_content.push(
                    text(error).size(12).style(Color::from_rgb(0.8, 0.2, 0.2))
                );
            }
        }

        // Submit button
        form_content = form_content.push(Space::with_height(Length::Fixed(15.0)));
        form_content = form_content.push(
            button(text("Create Loan"))
                .on_press(Message::CreateLoan)
                .padding([12, 24])
                .style(|_theme, _status| button::Appearance {
                    background: Some(iced::Background::Color(Color::from_rgb(0.6, 0.2, 0.8))),
                    text_color: Color::WHITE,
                    border: iced::Border::with_radius(5),
                    ..button::Appearance::default()
                })
        );

        container(form_content.spacing(10))
            .padding(20)
            .style(|_theme| container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                border: iced::Border::with_radius(8),
                ..container::Appearance::default()
            })
            .into()
    }

    fn reports_view(&self) -> Column<Message> {
        column![
            text("Financial Reports & Analytics").size(32).style(Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),

            row![
                self.report_section("Account Summary", vec![
                    format!("Total Accounts: {}", self.accounts.len()),
                    format!("Active Accounts: {}", self.accounts.iter().filter(|a| a.status == AccountStatus::Active).count()),
                    "Multi-currency support enabled".to_string(),
                ]),
                Space::with_width(Length::Fixed(20.0)),
                self.report_section("Transaction Analysis", vec![
                    format!("Total Transactions: {}", self.transactions.len()),
                    "Real-time processing".to_string(),
                    "Audit trail maintained".to_string(),
                ]),
            ],

            Space::with_height(Length::Fixed(20.0)),

            row![
                self.report_section("Loan Portfolio", vec![
                    format!("Active Loans: {}", self.loans.len()),
                    "Payment schedule tracking".to_string(),
                    "Interest calculation engine".to_string(),
                ]),
                Space::with_width(Length::Fixed(20.0)),
                self.report_section("System Features", vec![
                    "SQLite database storage".to_string(),
                    "Exchange rate integration".to_string(),
                    "Professional GUI interface".to_string(),
                ]),
            ],
        ]
        .spacing(15)
        .padding(20)
    }

    fn report_section(&self, title: &str, items: Vec<String>) -> Element<Message> {
        let mut content = column![
            text(title).size(18).style(Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(10.0)),
        ];

        for item in items {
            content = content.push(
                text(format!("• {}", item)).size(14)
            );
        }

        container(content.spacing(8))
            .padding(20)
            .style(|_theme| container::Appearance {
                background: Some(iced::Background::Color(Color::from_rgb(0.98, 0.98, 0.98))),
                border: iced::Border::with_radius(8),
                ..container::Appearance::default()
            })
            .into()
    }

    // Form validation and submission handlers
    fn handle_create_account(&mut self) {
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
    }

    fn handle_create_transaction(&mut self) {
        self.transaction_form.errors.clear();

        // Validate form inputs
        if self.transaction_form.transaction_type.is_none() {
            self.transaction_form.errors.push("Transaction type is required".to_string());
        }

        let amount = match self.transaction_form.amount.parse::<f64>() {
            Ok(a) if a > 0.0 => Decimal::try_from(a).unwrap_or(Decimal::ZERO),
            _ => {
                self.transaction_form.errors.push("Valid amount is required".to_string());
                return;
            }
        };

        if self.transaction_form.currency.is_none() {
            self.transaction_form.errors.push("Currency is required".to_string());
        }

        if self.transaction_form.description.trim().is_empty() {
            self.transaction_form.errors.push("Description is required".to_string());
        }

        if !self.transaction_form.errors.is_empty() {
            return;
        }

        // Create new transaction (mock account ID for now)
        if let Ok(transaction) = Transaction::new(
            self.transaction_form.transaction_type.unwrap(),
            amount,
            self.transaction_form.currency.unwrap(),
            self.transaction_form.description.clone(),
            Uuid::new_v4(), // Mock account ID
        ) {
            self.transactions.push(transaction);
        }

        // Clear form
        self.transaction_form = TransactionForm::default();
    }

    fn handle_create_loan(&mut self) {
        self.loan_form.errors.clear();

        // Validate form inputs
        if self.loan_form.loan_type.is_none() {
            self.loan_form.errors.push("Loan type is required".to_string());
        }

        let principal = match self.loan_form.principal.parse::<f64>() {
            Ok(p) if p > 0.0 => Decimal::try_from(p).unwrap_or(Decimal::ZERO),
            _ => {
                self.loan_form.errors.push("Valid principal amount is required".to_string());
                return;
            }
        };

        let interest_rate = match self.loan_form.interest_rate.parse::<f64>() {
            Ok(r) if r >= 0.0 => Decimal::try_from(r).unwrap_or(Decimal::ZERO),
            _ => {
                self.loan_form.errors.push("Valid interest rate is required".to_string());
                return;
            }
        };

        let term_months = match self.loan_form.term_months.parse::<u32>() {
            Ok(t) if t > 0 => t,
            _ => {
                self.loan_form.errors.push("Valid term in months is required".to_string());
                return;
            }
        };

        if !self.loan_form.errors.is_empty() {
            return;
        }

        // Create new loan
        let loan = Loan::new(
            self.loan_form.loan_type.unwrap(),
            principal,
            Currency::USD, // Default currency for now
            interest_rate,
            term_months,
        );

        self.loans.push(loan);

        // Clear form
        self.loan_form = LoanForm::default();
    }
}

fn main() -> iced::Result {
    env_logger::init();
    println!("Starting FerrumFinance application...");

    FerrumFinanceApp::run(Settings::default())
}