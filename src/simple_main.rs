use iced::{Alignment, Application, Element, Length, Settings, Theme};
use iced::widget::{column, container, text, Image, Space};
use std::time::Duration;
use tokio::time;

mod types;
mod messages;
mod app_state;

// Re-export commonly used types
pub use types::*;
pub use messages::Message;
pub use app_state::AppState;

// Asset management functions
const LOGO_1: &[u8] = include_bytes!("assets/ferrum-finance-logo-1.png");

fn get_splash_logo() -> iced::widget::image::Handle {
    iced::widget::image::Handle::from_memory(LOGO_1)
}

// Simple application struct for testing
pub struct SimpleFerrumFinanceApp {
    state: AppState,
}

impl Application for SimpleFerrumFinanceApp {
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
            SimpleFerrumFinanceApp {
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
            Message::NavigateTo(screen) => {
                self.state.current_screen = screen;
                self.state.hide_all_forms();
            }
            Message::SplashTimeout => {
                self.state.current_screen = Screen::Dashboard;
            }
            Message::CreateAccount => {
                self.state.handle_create_account();
            }
            Message::CreateTransaction => {
                self.state.handle_create_transaction();
            }
            Message::CreateLoan => {
                self.state.handle_create_loan();
            }
            _ => {
                // Handle other messages as needed
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> Element<Message> {
        match self.state.current_screen {
            Screen::Splash => self.splash_view(),
            _ => self.dashboard_view(),
        }
    }
}

impl SimpleFerrumFinanceApp {
    fn splash_view(&self) -> Element<Message> {
        let logo = Image::new(get_splash_logo())
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(200.0));

        let content = column![
            logo,
            Space::with_height(Length::Fixed(30.0)),
            text("FerrumFinance")
                .size(48)
                .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            text("Multi-Currency Financial Management")
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
            .style(|_theme| container::Appearance {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.98, 0.98, 0.98))),
                ..container::Appearance::default()
            })
            .into()
    }

    fn dashboard_view(&self) -> Element<Message> {
        column![
            text("FerrumFinance Dashboard")
                .size(32)
                .style(iced::Color::from_rgb(0.2, 0.4, 0.8)),
            Space::with_height(Length::Fixed(20.0)),
            text(format!("Accounts: {}", self.state.accounts.len())),
            text(format!("Transactions: {}", self.state.transactions.len())),
            text(format!("Loans: {}", self.state.loans.len())),
        ]
        .spacing(15)
        .padding(20)
        .into()
    }
}

fn main() -> iced::Result {
    env_logger::init();
    println!("Starting Simple FerrumFinance application...");

    SimpleFerrumFinanceApp::run(Settings::default())
}