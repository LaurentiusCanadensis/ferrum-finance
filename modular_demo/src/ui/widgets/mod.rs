pub mod account_widget;
pub mod transaction_widget;
pub mod loan_widget;
pub mod dashboard_widget;
pub mod common;

// Re-export widget components
pub use account_widget::AccountWidget;
pub use transaction_widget::TransactionWidget;
pub use loan_widget::LoanWidget;
pub use dashboard_widget::DashboardWidget;
pub use common::{SummaryCard, NavigationBar, HeaderView};