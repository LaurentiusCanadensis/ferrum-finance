use crate::types::{Account, Transaction, Loan, AccountForm, TransactionForm, LoanForm, Screen};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_screen: Screen,
    pub show_account_form: bool,
    pub show_transaction_form: bool,
    pub show_loan_form: bool,
    pub account_form: AccountForm,
    pub transaction_form: TransactionForm,
    pub loan_form: LoanForm,
    pub accounts: Vec<Account>,
    pub transactions: Vec<Transaction>,
    pub loans: Vec<Loan>,
    pub selected_account_id: Option<Uuid>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Splash,
            show_account_form: false,
            show_transaction_form: false,
            show_loan_form: false,
            account_form: AccountForm::default(),
            transaction_form: TransactionForm::default(),
            loan_form: LoanForm::default(),
            accounts: Vec::new(),
            transactions: Vec::new(),
            loans: Vec::new(),
            selected_account_id: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    // Account management methods
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

        // Clear form and hide it
        self.account_form = AccountForm::default();
        self.show_account_form = false;
    }

    // Transaction management methods
    pub fn handle_create_transaction(&mut self) {
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

        // Clear form and hide it
        self.transaction_form = TransactionForm::default();
        self.show_transaction_form = false;
    }

    // Loan management methods
    pub fn handle_create_loan(&mut self) {
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
            crate::types::Currency::USD, // Default currency for now
            interest_rate,
            term_months,
        );

        self.loans.push(loan);

        // Clear form and hide it
        self.loan_form = LoanForm::default();
        self.show_loan_form = false;
    }

    // UI state management
    pub fn show_account_form(&mut self) {
        self.show_account_form = true;
        self.show_transaction_form = false;
        self.show_loan_form = false;
        self.account_form.errors.clear();
    }

    pub fn show_transaction_form(&mut self) {
        self.show_transaction_form = true;
        self.show_account_form = false;
        self.show_loan_form = false;
        self.transaction_form.errors.clear();
    }

    pub fn show_loan_form(&mut self) {
        self.show_loan_form = true;
        self.show_account_form = false;
        self.show_transaction_form = false;
        self.loan_form.errors.clear();
    }

    pub fn hide_all_forms(&mut self) {
        self.show_account_form = false;
        self.show_transaction_form = false;
        self.show_loan_form = false;
    }

    pub fn clear_all_errors(&mut self) {
        self.account_form.errors.clear();
        self.transaction_form.errors.clear();
        self.loan_form.errors.clear();
    }
}

use crate::types::*;