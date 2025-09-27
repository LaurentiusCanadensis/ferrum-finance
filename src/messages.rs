use crate::types::{Screen, AccountType, Currency, TransactionType, LoanType};
use uuid::Uuid;

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
    HideAccountForm,
    RefreshAccounts,
    EditAccount(Uuid),
    ViewAccountTransactions(Uuid),

    // Transaction form messages
    TransactionTypeSelected(TransactionType),
    TransactionAmountChanged(String),
    TransactionCurrencySelected(Currency),
    TransactionDescriptionChanged(String),
    TransactionAccountSelected(String),
    CreateTransaction,
    ShowTransactionForm,
    HideTransactionForm,
    ShowTransactionFilter,
    ExportTransactions,

    // Loan form messages
    LoanTypeSelected(LoanType),
    LoanPrincipalChanged(String),
    LoanInterestRateChanged(String),
    LoanTermChanged(String),
    CreateLoan,
    ShowLoanForm,
    HideLoanForm,
    ShowPaymentCalculator,
    GenerateLoanReport,

    // Reports messages
    ExportReportPDF,
    ExportReportCSV,
    EmailReport,

    // General form messages
    ClearErrors,
    FormSubmitted,

    // New dashboard and navigation messages
    ShowDashboard,
    RefreshData,
}