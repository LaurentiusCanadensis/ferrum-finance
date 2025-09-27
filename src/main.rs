use iced::{Alignment, Application, Element, Length, Settings, Theme};
use iced::widget::{column, container, text, Image, Space};
use std::time::Duration;
use tokio::time;

mod types;
mod messages;
mod app_state;
mod ui;

// Re-export commonly used types
pub use types::*;
pub use messages::Message;
pub use app_state::AppState;

// Main application struct
pub struct FerrumFinanceApp {
    state: AppState,
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
                state: AppState::new(),
            },
            splash_command,
        )
    }

    fn title(&self) -> String {
        "FerrumFinance - Multi-Currency Financial Management".to_string()
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            // Navigation messages
            Message::NavigateTo(screen) => {
                self.state.current_screen = screen;
                self.state.hide_all_forms();
            }
            Message::SplashTimeout => {
                self.state.current_screen = Screen::Dashboard;
            }

            // Account form messages
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
                self.state.show_account_form();
            }
            Message::HideAccountForm => {
                self.state.show_account_form = false;
                self.state.account_form = AccountForm::default();
            }

            // Transaction form messages
            Message::TransactionTypeSelected(transaction_type) => {
                self.state.transaction_form.transaction_type = Some(transaction_type);
                self.state.transaction_form.errors.clear();
            }
            Message::TransactionAmountChanged(amount) => {
                self.state.transaction_form.amount = amount;
                self.state.transaction_form.errors.clear();
            }
            Message::TransactionCurrencySelected(currency) => {
                self.state.transaction_form.currency = Some(currency);
                self.state.transaction_form.errors.clear();
            }
            Message::TransactionDescriptionChanged(description) => {
                self.state.transaction_form.description = description;
                self.state.transaction_form.errors.clear();
            }
            Message::TransactionAccountSelected(account_id) => {
                self.state.transaction_form.account_id = account_id;
                self.state.transaction_form.errors.clear();
            }
            Message::CreateTransaction => {
                self.state.handle_create_transaction();
            }
            Message::ShowTransactionForm => {
                self.state.show_transaction_form();
            }
            Message::HideTransactionForm => {
                self.state.show_transaction_form = false;
                self.state.transaction_form = TransactionForm::default();
            }

            // Loan form messages
            Message::LoanTypeSelected(loan_type) => {
                self.state.loan_form.loan_type = Some(loan_type);
                self.state.loan_form.errors.clear();
            }
            Message::LoanPrincipalChanged(principal) => {
                self.state.loan_form.principal = principal;
                self.state.loan_form.errors.clear();
            }
            Message::LoanInterestRateChanged(rate) => {
                self.state.loan_form.interest_rate = rate;
                self.state.loan_form.errors.clear();
            }
            Message::LoanTermChanged(term) => {
                self.state.loan_form.term_months = term;
                self.state.loan_form.errors.clear();
            }
            Message::CreateLoan => {
                self.state.handle_create_loan();
            }
            Message::ShowLoanForm => {
                self.state.show_loan_form();
            }
            Message::HideLoanForm => {
                self.state.show_loan_form = false;
                self.state.loan_form = LoanForm::default();
            }

            // Other messages - placeholder implementations
            Message::ClearErrors => {
                self.state.clear_all_errors();
            }
            Message::FormSubmitted => {
                // Handle generic form submission if needed
            }
            Message::RefreshAccounts => {
                // Refresh account data - placeholder
            }
            Message::EditAccount(_id) => {
                // Edit account - placeholder
            }
            Message::ViewAccountTransactions(id) => {
                self.state.selected_account_id = Some(id);
                self.state.current_screen = Screen::Transactions;
            }
            Message::ShowTransactionFilter => {
                // Show transaction filter - placeholder
            }
            Message::ExportTransactions => {
                // Export transactions - placeholder
            }
            Message::ShowPaymentCalculator => {
                // Show payment calculator - placeholder
            }
            Message::GenerateLoanReport => {
                // Generate loan report - placeholder
            }
            Message::ExportReportPDF => {
                // Export to PDF - placeholder
            }
            Message::ExportReportCSV => {
                // Export to CSV - placeholder
            }
            Message::EmailReport => {
                // Email report - placeholder
            }
            Message::ShowDashboard => {
                self.state.current_screen = Screen::Dashboard;
            }
            Message::RefreshData => {
                // Refresh all data - placeholder
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

impl FerrumFinanceApp {
    fn splash_view(&self) -> Element<Message> {
        let logo = Image::new(ui::get_splash_logo())
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0));

        let content = column![
            logo,
            Space::with_height(Length::Fixed(30.0)),
            text("FerrumFinance")
                .size(48)
                .style(ui::PRIMARY_COLOR),
            text("Multi-Currency Financial Management")
                .size(18)
                .style(ui::LIGHT_TEXT_COLOR),
        ]
        .align_items(Alignment::Center)
        .spacing(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(|_theme| container::Appearance {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.98, 0.98, 0.98))),
                ..container::Appearance::default()
            })
            .into()
    }

    fn main_app_view(&self) -> Element<Message> {
        let header = ui::widgets::common::HeaderView::new();
        let navigation = ui::widgets::common::NavigationBar::new(&self.state.current_screen);

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
        ui::widgets::DashboardWidget::new(
            &self.state.accounts,
            &self.state.transactions,
            &self.state.loans
        )
    }

    fn accounts_view(&self) -> Element<Message> {
        ui::screens::AccountsScreen::view(
            &self.state.accounts,
            &self.state.account_form,
            self.state.show_account_form,
        )
    }

    fn transactions_view(&self) -> Element<Message> {
        ui::screens::TransactionsScreen::view(
            &self.state.transactions,
            &self.state.transaction_form,
            self.state.show_transaction_form,
        )
    }

    fn loans_view(&self) -> Element<Message> {
        ui::screens::LoansScreen::view(
            &self.state.loans,
            &self.state.loan_form,
            self.state.show_loan_form,
        )
    }

    fn reports_view(&self) -> Element<Message> {
        ui::screens::ReportsScreen::view(
            &self.state.accounts,
            &self.state.transactions,
            &self.state.loans,
        )
    }
}

fn main() -> iced::Result {
    env_logger::init();
    println!("Starting FerrumFinance application...");

    FerrumFinanceApp::run(Settings::default())
}