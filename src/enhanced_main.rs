use iced::{Alignment, Element, Length, Settings, Theme, Color, Task};
use iced::widget::{button, column, container, row, text, text_input, pick_list, Image, Space, Svg};
use colorum::widgets::color_wheel::ColorWheel;
use std::time::Duration;
use tokio::time;

mod types;
mod messages;
mod app_state;
mod assets;

// Re-export commonly used types
pub use types::*;
pub use messages::Message;
pub use app_state::AppState;

// Enhanced message enum
#[derive(Debug, Clone)]
pub enum EnhancedMessage {
    SplashTimeout,
    MenuToggle,
    NavigateTo(Screen),

    // Account form messages
    ShowAccountForm,
    HideAccountForm,
    AccountNameChanged(String),
    AccountTypeSelected(AccountType),
    AccountCurrencySelected(Currency),
    AccountInitialBalanceChanged(String),
    CreateAccount,

    // Transaction form messages
    ShowTransactionForm,
    HideTransactionForm,
    TransactionTypeSelected(TransactionType),
    TransactionAmountChanged(String),
    TransactionCurrencySelected(Currency),
    TransactionDescriptionChanged(String),
    CreateTransaction,

    // Loan form messages
    ShowLoanForm,
    HideLoanForm,
    LoanTypeSelected(LoanType),
    LoanPrincipalChanged(String),
    LoanInterestRateChanged(String),
    LoanTermChanged(String),
    CreateLoan,

    // Settings messages
    ShowPrimaryColorPicker,
    HidePrimaryColorPicker,
    PrimaryColorChanged(Color),
    ShowSecondaryColorPicker,
    HideSecondaryColorPicker,
    SecondaryColorChanged(Color),
    ShowBackgroundColorPicker,
    HideBackgroundColorPicker,
    BackgroundColorChanged(Color),
    SaveThemeSettings,
}

// Enhanced application struct
pub struct EnhancedFerrumFinanceApp {
    state: AppState,
    menu_open: bool,
    primary_color: Color,
    secondary_color: Color,
    background_color: Color,
    show_primary_picker: bool,
    show_secondary_picker: bool,
    show_background_picker: bool,
}

impl iced::Application for EnhancedFerrumFinanceApp {
    type Message = EnhancedMessage;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Task<EnhancedMessage>) {
        let splash_command = Task::perform(
            async {
                time::sleep(Duration::from_secs(2)).await;
            },
            |_| EnhancedMessage::SplashTimeout,
        );

        (
            EnhancedFerrumFinanceApp {
                state: AppState::new(),
                menu_open: false,
                primary_color: Color::from_rgb(0.2, 0.4, 0.8),
                secondary_color: Color::from_rgb(0.1, 0.7, 0.4),
                background_color: Color::from_rgb(0.98, 0.98, 0.98),
                show_primary_picker: false,
                show_secondary_picker: false,
                show_background_picker: false,
            },
            splash_command,
        )
    }

    fn title(&self) -> String {
        "FerrumFinance - Enhanced Multi-Currency Financial Management".to_string()
    }

    fn update(&mut self, message: EnhancedMessage) -> Task<EnhancedMessage> {
        match message {
            EnhancedMessage::SplashTimeout => {
                self.state.current_screen = Screen::Dashboard;
            }
            EnhancedMessage::MenuToggle => {
                self.menu_open = !self.menu_open;
            }
            EnhancedMessage::NavigateTo(screen) => {
                self.state.current_screen = screen;
                self.state.hide_all_forms();
            }

            // Account form messages
            EnhancedMessage::ShowAccountForm => {
                self.state.show_account_form();
            }
            EnhancedMessage::HideAccountForm => {
                self.state.show_account_form = false;
                self.state.account_form = AccountForm::default();
            }
            EnhancedMessage::AccountNameChanged(name) => {
                self.state.account_form.name = name;
                self.state.account_form.errors.clear();
            }
            EnhancedMessage::AccountTypeSelected(account_type) => {
                self.state.account_form.account_type = Some(account_type);
                self.state.account_form.errors.clear();
            }
            EnhancedMessage::AccountCurrencySelected(currency) => {
                self.state.account_form.currency = Some(currency);
                self.state.account_form.errors.clear();
            }
            EnhancedMessage::AccountInitialBalanceChanged(balance) => {
                self.state.account_form.initial_balance = balance;
                self.state.account_form.errors.clear();
            }
            EnhancedMessage::CreateAccount => {
                self.state.handle_create_account();
            }

            // Transaction form messages
            EnhancedMessage::ShowTransactionForm => {
                self.state.show_transaction_form();
            }
            EnhancedMessage::HideTransactionForm => {
                self.state.show_transaction_form = false;
                self.state.transaction_form = TransactionForm::default();
            }
            EnhancedMessage::TransactionTypeSelected(transaction_type) => {
                self.state.transaction_form.transaction_type = Some(transaction_type);
                self.state.transaction_form.errors.clear();
            }
            EnhancedMessage::TransactionAmountChanged(amount) => {
                self.state.transaction_form.amount = amount;
                self.state.transaction_form.errors.clear();
            }
            EnhancedMessage::TransactionCurrencySelected(currency) => {
                self.state.transaction_form.currency = Some(currency);
                self.state.transaction_form.errors.clear();
            }
            EnhancedMessage::TransactionDescriptionChanged(description) => {
                self.state.transaction_form.description = description;
                self.state.transaction_form.errors.clear();
            }
            EnhancedMessage::CreateTransaction => {
                self.state.handle_create_transaction();
            }

            // Loan form messages
            EnhancedMessage::ShowLoanForm => {
                self.state.show_loan_form();
            }
            EnhancedMessage::HideLoanForm => {
                self.state.show_loan_form = false;
                self.state.loan_form = LoanForm::default();
            }
            EnhancedMessage::LoanTypeSelected(loan_type) => {
                self.state.loan_form.loan_type = Some(loan_type);
                self.state.loan_form.errors.clear();
            }
            EnhancedMessage::LoanPrincipalChanged(principal) => {
                self.state.loan_form.principal = principal;
                self.state.loan_form.errors.clear();
            }
            EnhancedMessage::LoanInterestRateChanged(rate) => {
                self.state.loan_form.interest_rate = rate;
                self.state.loan_form.errors.clear();
            }
            EnhancedMessage::LoanTermChanged(term) => {
                self.state.loan_form.term_months = term;
                self.state.loan_form.errors.clear();
            }
            EnhancedMessage::CreateLoan => {
                self.state.handle_create_loan();
            }

            // Settings messages
            EnhancedMessage::ShowPrimaryColorPicker => {
                self.show_primary_picker = true;
            }
            EnhancedMessage::HidePrimaryColorPicker => {
                self.show_primary_picker = false;
            }
            EnhancedMessage::PrimaryColorChanged(color) => {
                self.primary_color = color;
            }
            EnhancedMessage::ShowSecondaryColorPicker => {
                self.show_secondary_picker = true;
            }
            EnhancedMessage::HideSecondaryColorPicker => {
                self.show_secondary_picker = false;
            }
            EnhancedMessage::SecondaryColorChanged(color) => {
                self.secondary_color = color;
            }
            EnhancedMessage::ShowBackgroundColorPicker => {
                self.show_background_picker = true;
            }
            EnhancedMessage::HideBackgroundColorPicker => {
                self.show_background_picker = false;
            }
            EnhancedMessage::BackgroundColorChanged(color) => {
                self.background_color = color;
            }
            EnhancedMessage::SaveThemeSettings => {
                println!("Theme settings saved!");
                println!("Primary: {:?}", self.primary_color);
                println!("Secondary: {:?}", self.secondary_color);
                println!("Background: {:?}", self.background_color);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<EnhancedMessage> {
        match self.state.current_screen {
            Screen::Splash => self.splash_view(),
            _ => self.main_layout_view(),
        }
    }
}

impl EnhancedFerrumFinanceApp {
    fn splash_view(&self) -> Element<EnhancedMessage> {
        let logo = Image::new(assets::get_splash_logo())
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0));

        let content = column![
            logo,
            Space::with_height(Length::Fixed(30.0)),
            text("FerrumFinance")
                .size(48)
                .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            text("Enhanced Multi-Currency Financial Management")
                .size(18)
                .style(iced::Color::from_rgb(0.4, 0.4, 0.4)),
        ]
        .align_items(Alignment::Center)
        .spacing(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn main_layout_view(&self) -> Element<EnhancedMessage> {
        let menu_bar = self.menu_bar();
        let left_panel = self.left_panel();
        let main_content = self.main_content();
        let right_panel = self.right_panel();

        let content_row = row![
            left_panel,
            main_content,
            right_panel,
        ]
        .spacing(0);

        column![
            menu_bar,
            content_row,
        ]
        .spacing(0)
        .into()
    }

    fn menu_bar(&self) -> Element<EnhancedMessage> {
        let logo = Image::new(assets::get_app_logo())
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0));

        let menu_button = button("☰ Menu")
            .on_press(EnhancedMessage::MenuToggle);

        let title = text("FerrumFinance")
            .size(24)
            .style(iced::Color::from_rgb(0.2, 0.4, 0.8));

        let menu_content = row![
            Space::with_width(Length::Fixed(10.0)),
            logo,
            Space::with_width(Length::Fixed(15.0)),
            title,
            Space::with_width(Length::Fill),
            menu_button,
            Space::with_width(Length::Fixed(10.0)),
        ]
        .align_items(Alignment::Center)
        .spacing(10);

        container(menu_content)
            .width(Length::Fill)
            .height(Length::Fixed(60.0))
            .style(|_theme: &Theme| container::Style {
                background: Some(iced::Color::from_rgb(0.95, 0.95, 0.95).into()),
                border: iced::Border {
                    width: 0.0,
                    radius: [0.0; 4].into(),
                    color: iced::Color::TRANSPARENT,
                },
                shadow: iced::Shadow::default(),
            })
            .into()
    }

    fn left_panel(&self) -> Element<EnhancedMessage> {
        let sidebar_svg: Svg<Theme> = Svg::new(assets::get_sidebar_svg())
            .width(Length::Fixed(200.0))
            .height(Length::Fill);

        let navigation = column![
            Space::with_height(Length::Fixed(20.0)),
            button("Dashboard")
                .width(Length::Fixed(180.0))
                .on_press(EnhancedMessage::NavigateTo(Screen::Dashboard)),
            Space::with_height(Length::Fixed(10.0)),
            button("Accounts")
                .width(Length::Fixed(180.0))
                .on_press(EnhancedMessage::NavigateTo(Screen::Accounts)),
            Space::with_height(Length::Fixed(10.0)),
            button("Transactions")
                .width(Length::Fixed(180.0))
                .on_press(EnhancedMessage::NavigateTo(Screen::Transactions)),
            Space::with_height(Length::Fixed(10.0)),
            button("Loans")
                .width(Length::Fixed(180.0))
                .on_press(EnhancedMessage::NavigateTo(Screen::Loans)),
            Space::with_height(Length::Fixed(10.0)),
            button("Reports")
                .width(Length::Fixed(180.0))
                .on_press(EnhancedMessage::NavigateTo(Screen::Reports)),
            Space::with_height(Length::Fixed(10.0)),
            button("Settings")
                .width(Length::Fixed(180.0))
                .on_press(EnhancedMessage::NavigateTo(Screen::Settings)),
        ]
        .align_items(Alignment::Center)
        .spacing(5);

        // Layer the navigation buttons over the SVG background
        container(
            container(navigation)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
        )
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Color::from_rgb(0.12, 0.16, 0.23).into()),
            border: iced::Border {
                width: 1.0,
                radius: [0.0; 4].into(),
                color: iced::Color::from_rgb(0.2, 0.2, 0.3),
            },
            shadow: iced::Shadow::default(),
        })
        .into()
    }

    fn main_content(&self) -> Element<EnhancedMessage> {
        let content = match self.state.current_screen {
            Screen::Dashboard => self.dashboard_content(),
            Screen::Accounts => self.accounts_content(),
            Screen::Transactions => self.transactions_content(),
            Screen::Loans => self.loans_content(),
            Screen::Reports => self.reports_content(),
            Screen::Settings => self.settings_content(),
            Screen::Splash => column![].into(), // Should not reach here
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn right_panel(&self) -> Element<EnhancedMessage> {
        let pattern_svg: Svg<Theme> = Svg::new(assets::get_pattern_svg())
            .width(Length::Fixed(200.0))
            .height(Length::Fill);

        let panel_content = column![
            Space::with_height(Length::Fixed(20.0)),
            text("Financial Insights")
                .size(16)
                .style(iced::Color::from_rgb(0.3, 0.3, 0.3)),
            Space::with_height(Length::Fixed(20.0)),
            text("• Multi-currency support")
                .size(12)
                .style(iced::Color::from_rgb(0.4, 0.4, 0.4)),
            Space::with_height(Length::Fixed(10.0)),
            text("• Real-time analytics")
                .size(12)
                .style(iced::Color::from_rgb(0.4, 0.4, 0.4)),
            Space::with_height(Length::Fixed(10.0)),
            text("• Comprehensive reporting")
                .size(12)
                .style(iced::Color::from_rgb(0.4, 0.4, 0.4)),
        ]
        .align_items(Alignment::Center)
        .spacing(5);

        container(
            container(panel_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
        )
        .width(Length::Fixed(200.0))
        .height(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Color::from_rgb(0.98, 0.98, 0.98).into()),
            border: iced::Border {
                width: 1.0,
                radius: [0.0; 4].into(),
                color: iced::Color::from_rgb(0.9, 0.9, 0.9),
            },
            shadow: iced::Shadow::default(),
        })
        .into()
    }

    fn dashboard_content(&self) -> Element<EnhancedMessage> {
        column![
            text("FerrumFinance Dashboard")
                .size(32)
                .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
            row![
                self.dashboard_card("Accounts", &self.state.accounts.len().to_string(), "💼"),
                Space::with_width(Length::Fixed(20.0)),
                self.dashboard_card("Transactions", &self.state.transactions.len().to_string(), "💸"),
                Space::with_width(Length::Fixed(20.0)),
                self.dashboard_card("Loans", &self.state.loans.len().to_string(), "🏦"),
            ],
            Space::with_height(Length::Fixed(30.0)),
            text("Quick Actions")
                .size(20)
                .style(iced::Color::from_rgb(0.3, 0.3, 0.3)),
            Space::with_height(Length::Fixed(15.0)),
            row![
                button("Add Account")
                    .on_press(EnhancedMessage::ShowAccountForm),
                Space::with_width(Length::Fixed(15.0)),
                button("New Transaction")
                    .on_press(EnhancedMessage::ShowTransactionForm),
                Space::with_width(Length::Fixed(15.0)),
                button("Create Loan")
                    .on_press(EnhancedMessage::ShowLoanForm),
            ],
        ]
        .spacing(15)
        .into()
    }

    fn dashboard_card(&self, title: &str, value: &str, icon: &str) -> Element<EnhancedMessage> {
        container(
            column![
                text(icon).size(32),
                Space::with_height(Length::Fixed(10.0)),
                text(title).size(14).style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                text(value).size(24).style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            ]
            .align_items(Alignment::Center)
            .spacing(5)
        )
        .width(Length::Fixed(150.0))
        .height(Length::Fixed(120.0))
        .center_x()
        .center_y()
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Color::WHITE.into()),
            border: iced::Border {
                width: 1.0,
                radius: [8.0; 4].into(),
                color: iced::Color::from_rgb(0.9, 0.9, 0.9),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
        })
        .into()
    }

    fn accounts_content(&self) -> Element<EnhancedMessage> {
        let header = column![
            text("Accounts Management")
                .size(28)
                .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
            button("Add New Account")
                .on_press(EnhancedMessage::ShowAccountForm),
            Space::with_height(Length::Fixed(20.0)),
        ];

        let mut content = column![header];

        // Show account form if requested
        if self.state.show_account_form {
            content = content.push(self.account_form_widget());
        }

        // Show existing accounts
        content = content.push(
            text(format!("Total Accounts: {}", self.state.accounts.len()))
                .size(16)
                .style(iced::Color::from_rgb(0.3, 0.3, 0.3))
        );

        content.into()
    }

    fn transactions_content(&self) -> Element<EnhancedMessage> {
        column![
            text("Transactions")
                .size(28)
                .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
            text("Track and manage all your financial transactions")
                .size(14)
                .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .into()
    }

    fn loans_content(&self) -> Element<EnhancedMessage> {
        column![
            text("Loans Management")
                .size(28)
                .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
            text("Comprehensive loan tracking and payment scheduling")
                .size(14)
                .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .into()
    }

    fn reports_content(&self) -> Element<EnhancedMessage> {
        column![
            text("Financial Reports")
                .size(28)
                .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
            text("Generate comprehensive financial reports and analytics")
                .size(14)
                .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .into()
    }

    fn settings_content(&self) -> Element<EnhancedMessage> {
        column![
            text("Theme Settings")
                .size(28)
                .style(self.primary_color),
            Space::with_height(Length::Fixed(20.0)),
            text("Customize your application colors")
                .size(14)
                .style(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            Space::with_height(Length::Fixed(30.0)),

            // Primary Color Settings
            {
                let primary_color = self.primary_color;
                row![
                    text("Primary Color:")
                        .size(18)
                        .style(iced::Color::from_rgb(0.3, 0.3, 0.3))
                        .width(Length::Fixed(150.0)),
                    button("Choose Color")
                        .on_press(EnhancedMessage::ShowPrimaryColorPicker)
                        .width(Length::Fixed(120.0)),
                    container(text(""))
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(30.0))
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(primary_color.into()),
                            border: iced::Border::with_radius(4),
                            ..container::Style::default()
                        }),
                ].spacing(10).align_items(Alignment::Center)
            },
            {
                let color_picker_element: Element<EnhancedMessage> = if self.show_primary_picker {
                    ColorPicker::<EnhancedMessage, Theme>::new(
                        self.show_primary_picker,
                        self.primary_color,
                        button("Cancel").on_press(EnhancedMessage::HidePrimaryColorPicker),
                        EnhancedMessage::HidePrimaryColorPicker,
                        EnhancedMessage::PrimaryColorChanged
                    ).into()
                } else {
                    Space::with_height(Length::Fixed(0.0)).into()
                };
                color_picker_element
            },
            Space::with_height(Length::Fixed(20.0)),

            // Secondary Color Settings
            {
                let secondary_color = self.secondary_color;
                row![
                    text("Secondary Color:")
                        .size(18)
                        .style(iced::Color::from_rgb(0.3, 0.3, 0.3))
                        .width(Length::Fixed(150.0)),
                    button("Choose Color")
                        .on_press(EnhancedMessage::ShowSecondaryColorPicker)
                        .width(Length::Fixed(120.0)),
                    container(text(""))
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(30.0))
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(secondary_color.into()),
                            border: iced::Border::with_radius(4),
                            ..container::Style::default()
                        }),
                ].spacing(10).align_items(Alignment::Center)
            },
            {
                let color_picker_element: Element<EnhancedMessage> = if self.show_secondary_picker {
                    ColorPicker::<EnhancedMessage, Theme>::new(
                        self.show_secondary_picker,
                        self.secondary_color,
                        button("Cancel").on_press(EnhancedMessage::HideSecondaryColorPicker),
                        EnhancedMessage::HideSecondaryColorPicker,
                        EnhancedMessage::SecondaryColorChanged
                    ).into()
                } else {
                    Space::with_height(Length::Fixed(0.0)).into()
                };
                color_picker_element
            },
            Space::with_height(Length::Fixed(20.0)),

            // Background Color Settings
            {
                let background_color = self.background_color;
                row![
                    text("Background Color:")
                        .size(18)
                        .style(iced::Color::from_rgb(0.3, 0.3, 0.3))
                        .width(Length::Fixed(150.0)),
                    button("Choose Color")
                        .on_press(EnhancedMessage::ShowBackgroundColorPicker)
                        .width(Length::Fixed(120.0)),
                    container(text(""))
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(30.0))
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(background_color.into()),
                            border: iced::Border::with_radius(4),
                            ..container::Style::default()
                        }),
                ].spacing(10).align_items(Alignment::Center)
            },
            {
                let color_picker_element: Element<EnhancedMessage> = if self.show_background_picker {
                    ColorPicker::<EnhancedMessage, Theme>::new(
                        self.show_background_picker,
                        self.background_color,
                        button("Cancel").on_press(EnhancedMessage::HideBackgroundColorPicker),
                        EnhancedMessage::HideBackgroundColorPicker,
                        EnhancedMessage::BackgroundColorChanged
                    ).into()
                } else {
                    Space::with_height(Length::Fixed(0.0)).into()
                };
                color_picker_element
            },
            Space::with_height(Length::Fixed(20.0)),

            // Save Button
            button("Save Theme Settings")
                .on_press(EnhancedMessage::SaveThemeSettings),

            Space::with_height(Length::Fixed(20.0)),
            text("Theme Preview")
                .size(16)
                .style(iced::Color::from_rgb(0.3, 0.3, 0.3)),
            {
                let primary_color = self.primary_color;
                let secondary_color = self.secondary_color;
                let background_color = self.background_color;

                container(
                    row![
                        container(text("Primary").style(iced::Color::WHITE))
                            .width(Length::Fixed(100.0))
                            .height(Length::Fixed(50.0))
                            .center_x()
                            .center_y()
                            .style(move |_theme: &Theme| container::Style {
                                background: Some(primary_color.into()),
                                border: iced::Border::with_radius(8),
                                ..container::Style::default()
                            }),
                        Space::with_width(Length::Fixed(10.0)),
                        container(text("Secondary").style(iced::Color::WHITE))
                            .width(Length::Fixed(100.0))
                            .height(Length::Fixed(50.0))
                            .center_x()
                            .center_y()
                            .style(move |_theme: &Theme| container::Style {
                                background: Some(secondary_color.into()),
                                border: iced::Border::with_radius(8),
                                ..container::Style::default()
                            }),
                        Space::with_width(Length::Fixed(10.0)),
                        container(text("Background").style(iced::Color::BLACK))
                            .width(Length::Fixed(100.0))
                            .height(Length::Fixed(50.0))
                            .center_x()
                            .center_y()
                            .style(move |_theme: &Theme| container::Style {
                                background: Some(background_color.into()),
                                border: iced::Border::with_radius(8),
                                ..container::Style::default()
                            }),
                    ]
                )
            }
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    // Form widgets
    fn account_form_widget(&self) -> Element<EnhancedMessage> {
        let form = container(
            column![
                text("Create New Account")
                    .size(20)
                    .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
                Space::with_height(Length::Fixed(15.0)),

                text("Account Name:"),
                text_input("Enter account name", &self.state.account_form.name)
                    .on_input(EnhancedMessage::AccountNameChanged)
                    .width(Length::Fixed(300.0)),
                Space::with_height(Length::Fixed(10.0)),

                text("Account Type:"),
                pick_list(
                    vec![AccountType::Asset, AccountType::Liability, AccountType::Equity, AccountType::Revenue, AccountType::Expense],
                    self.state.account_form.account_type.clone(),
                    EnhancedMessage::AccountTypeSelected
                )
                .width(Length::Fixed(300.0)),
                Space::with_height(Length::Fixed(10.0)),

                text("Currency:"),
                pick_list(
                    vec![Currency::USD, Currency::EUR, Currency::GBP, Currency::JPY, Currency::CAD, Currency::AUD, Currency::CHF, Currency::CNY],
                    self.state.account_form.currency.clone(),
                    EnhancedMessage::AccountCurrencySelected
                )
                .width(Length::Fixed(300.0)),
                Space::with_height(Length::Fixed(10.0)),

                text("Initial Balance:"),
                text_input("0.00", &self.state.account_form.initial_balance)
                    .on_input(EnhancedMessage::AccountInitialBalanceChanged)
                    .width(Length::Fixed(300.0)),
                Space::with_height(Length::Fixed(20.0)),

                row![
                    button("Create Account")
                        .on_press(EnhancedMessage::CreateAccount),
                    Space::with_width(Length::Fixed(10.0)),
                    button("Cancel")
                        .on_press(EnhancedMessage::HideAccountForm),
                ]
                .spacing(10),

                // Show errors if any
                if !self.state.account_form.errors.is_empty() {
                    text(self.state.account_form.errors.join(", "))
                        .style(iced::Color::from_rgb(0.8, 0.2, 0.2))
                } else {
                    text("")
                }
            ]
            .spacing(8)
            .padding(20)
        )
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Color::WHITE.into()),
            border: iced::Border {
                width: 1.0,
                radius: [8.0; 4].into(),
                color: iced::Color::from_rgb(0.8, 0.8, 0.8),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
        });

        form.into()
    }
}

fn main() -> iced::Result {
    env_logger::init();
    println!("Starting Enhanced FerrumFinance application...");

    EnhancedFerrumFinanceApp::run(Settings::default())
}