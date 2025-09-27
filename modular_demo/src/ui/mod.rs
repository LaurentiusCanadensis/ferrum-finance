pub mod widgets;
pub mod screens;

// Re-export commonly used UI components
pub use screens::{
    accounts_screen::AccountsScreen,
    transactions_screen::TransactionsScreen,
    loans_screen::LoansScreen,
    reports_screen::ReportsScreen,
};

pub use widgets::{
    account_widget::AccountWidget,
    transaction_widget::TransactionWidget,
    loan_widget::LoanWidget,
    dashboard_widget::DashboardWidget,
    common::{SummaryCard, NavigationBar, HeaderView},
};

// Common UI types and constants
use iced::{Color, Length};

pub const PRIMARY_COLOR: Color = Color::from_rgb(0.2, 0.4, 0.8);
pub const SUCCESS_COLOR: Color = Color::from_rgb(0.2, 0.6, 0.2);
pub const WARNING_COLOR: Color = Color::from_rgb(0.8, 0.4, 0.2);
pub const ERROR_COLOR: Color = Color::from_rgb(0.8, 0.2, 0.2);
pub const SECONDARY_COLOR: Color = Color::from_rgb(0.6, 0.2, 0.8);
pub const BACKGROUND_COLOR: Color = Color::from_rgb(0.98, 0.98, 0.98);
pub const TEXT_COLOR: Color = Color::from_rgb(0.3, 0.3, 0.3);
pub const LIGHT_TEXT_COLOR: Color = Color::from_rgb(0.5, 0.5, 0.5);

// Common spacing constants
pub const SPACING_SMALL: u16 = 10;
pub const SPACING_MEDIUM: u16 = 20;
pub const SPACING_LARGE: u16 = 30;

// Asset management functions
const LOGO_1: &[u8] = include_bytes!("../assets/ferrum-finance-logo-1.png");
const LOGO_2: &[u8] = include_bytes!("../assets/ferrum-finance-logo-2.png");

pub fn get_splash_logo() -> iced::widget::image::Handle {
    iced::widget::image::Handle::from_memory(LOGO_1)
}

pub fn get_app_logo() -> iced::widget::image::Handle {
    iced::widget::image::Handle::from_memory(LOGO_2)
}