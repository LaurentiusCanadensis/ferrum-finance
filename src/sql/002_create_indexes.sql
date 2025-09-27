-- Migration 002: Create indexes for performance optimization
-- This migration creates all necessary indexes for optimal query performance

-- Accounts table indexes
CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(account_type);
CREATE INDEX IF NOT EXISTS idx_accounts_status ON accounts(status);
CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_account_id);
CREATE INDEX IF NOT EXISTS idx_accounts_currency ON accounts(default_currency);
CREATE INDEX IF NOT EXISTS idx_accounts_created ON accounts(created_at);
CREATE INDEX IF NOT EXISTS idx_accounts_updated ON accounts(updated_at);
CREATE INDEX IF NOT EXISTS idx_accounts_closed ON accounts(is_closed, closed_at);
CREATE INDEX IF NOT EXISTS idx_accounts_institution ON accounts(institution_name) WHERE institution_name IS NOT NULL;

-- Account balances indexes
CREATE INDEX IF NOT EXISTS idx_account_balances_account ON account_balances(account_id);
CREATE INDEX IF NOT EXISTS idx_account_balances_currency ON account_balances(currency_code);
CREATE INDEX IF NOT EXISTS idx_account_balances_updated ON account_balances(last_updated);
CREATE INDEX IF NOT EXISTS idx_account_balances_balance ON account_balances(balance) WHERE balance != 0;

-- Transactions table indexes
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions(status);
CREATE INDEX IF NOT EXISTS idx_transactions_priority ON transactions(priority);
CREATE INDEX IF NOT EXISTS idx_transactions_from_account ON transactions(from_account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_to_account ON transactions(to_account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_currency ON transactions(currency_code);
CREATE INDEX IF NOT EXISTS idx_transactions_amount ON transactions(amount);
CREATE INDEX IF NOT EXISTS idx_transactions_scheduled ON transactions(scheduled_date) WHERE scheduled_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_processed ON transactions(processed_date) WHERE processed_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_settlement ON transactions(settlement_date) WHERE settlement_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_created ON transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_updated ON transactions(updated_at);
CREATE INDEX IF NOT EXISTS idx_transactions_reference ON transactions(reference_number) WHERE reference_number IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_external_id ON transactions(external_id) WHERE external_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category) WHERE category IS NOT NULL;

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_transactions_account_date ON transactions(from_account_id, processed_date) WHERE from_account_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_to_account_date ON transactions(to_account_id, processed_date) WHERE to_account_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_transactions_type_status ON transactions(transaction_type, status);
CREATE INDEX IF NOT EXISTS idx_transactions_status_date ON transactions(status, processed_date);
CREATE INDEX IF NOT EXISTS idx_transactions_currency_date ON transactions(currency_code, processed_date);

-- Recurring transactions indexes
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_template ON recurring_transactions(template_transaction_id);
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_pattern ON recurring_transactions(pattern);
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_frequency ON recurring_transactions(frequency);
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_active ON recurring_transactions(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_next_exec ON recurring_transactions(next_execution_date) WHERE next_execution_date IS NOT NULL AND is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_start_date ON recurring_transactions(start_date);
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_end_date ON recurring_transactions(end_date) WHERE end_date IS NOT NULL;

-- Exchange rates indexes
CREATE INDEX IF NOT EXISTS idx_exchange_rates_base_currency ON exchange_rates(base_currency);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_target_currency ON exchange_rates(target_currency);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_pair ON exchange_rates(base_currency, target_currency);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_source ON exchange_rates(source);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_effective_date ON exchange_rates(effective_date);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_active ON exchange_rates(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_exchange_rates_created ON exchange_rates(created_at);

-- Composite indexes for exchange rate queries
CREATE INDEX IF NOT EXISTS idx_exchange_rates_pair_date ON exchange_rates(base_currency, target_currency, effective_date);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_active_pair ON exchange_rates(base_currency, target_currency, is_active) WHERE is_active = TRUE;

-- Loans table indexes
CREATE INDEX IF NOT EXISTS idx_loans_type ON loans(loan_type);
CREATE INDEX IF NOT EXISTS idx_loans_status ON loans(status);
CREATE INDEX IF NOT EXISTS idx_loans_borrower ON loans(borrower_account_id);
CREATE INDEX IF NOT EXISTS idx_loans_lender ON loans(lender_account_id) WHERE lender_account_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_loans_currency ON loans(currency_code);
CREATE INDEX IF NOT EXISTS idx_loans_origination_date ON loans(origination_date);
CREATE INDEX IF NOT EXISTS idx_loans_maturity_date ON loans(maturity_date);
CREATE INDEX IF NOT EXISTS idx_loans_first_payment_date ON loans(first_payment_date);
CREATE INDEX IF NOT EXISTS idx_loans_payment_frequency ON loans(payment_frequency);
CREATE INDEX IF NOT EXISTS idx_loans_interest_rate_type ON loans(interest_rate_type);
CREATE INDEX IF NOT EXISTS idx_loans_created ON loans(created_at);
CREATE INDEX IF NOT EXISTS idx_loans_updated ON loans(updated_at);

-- Composite indexes for loan queries
CREATE INDEX IF NOT EXISTS idx_loans_borrower_status ON loans(borrower_account_id, status);
CREATE INDEX IF NOT EXISTS idx_loans_status_maturity ON loans(status, maturity_date);
CREATE INDEX IF NOT EXISTS idx_loans_type_status ON loans(loan_type, status);

-- Loan payments indexes
CREATE INDEX IF NOT EXISTS idx_loan_payments_loan ON loan_payments(loan_id);
CREATE INDEX IF NOT EXISTS idx_loan_payments_payment_number ON loan_payments(loan_id, payment_number);
CREATE INDEX IF NOT EXISTS idx_loan_payments_due_date ON loan_payments(due_date);
CREATE INDEX IF NOT EXISTS idx_loan_payments_payment_date ON loan_payments(payment_date) WHERE payment_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_loan_payments_status ON loan_payments(status);
CREATE INDEX IF NOT EXISTS idx_loan_payments_transaction ON loan_payments(transaction_id) WHERE transaction_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_loan_payments_created ON loan_payments(created_at);
CREATE INDEX IF NOT EXISTS idx_loan_payments_updated ON loan_payments(updated_at);

-- Composite indexes for loan payment queries
CREATE INDEX IF NOT EXISTS idx_loan_payments_loan_status ON loan_payments(loan_id, status);
CREATE INDEX IF NOT EXISTS idx_loan_payments_loan_due_date ON loan_payments(loan_id, due_date);
CREATE INDEX IF NOT EXISTS idx_loan_payments_status_due_date ON loan_payments(status, due_date);
CREATE INDEX IF NOT EXISTS idx_loan_payments_overdue ON loan_payments(due_date, status) WHERE status IN ('Pending', 'Late');

-- Currencies indexes
CREATE INDEX IF NOT EXISTS idx_currencies_active ON currencies(is_active) WHERE is_active = TRUE;
CREATE INDEX IF NOT EXISTS idx_currencies_name ON currencies(name);
CREATE INDEX IF NOT EXISTS idx_currencies_created ON currencies(created_at);

-- Performance optimization indexes for specific query patterns

-- Monthly transaction summaries
CREATE INDEX IF NOT EXISTS idx_transactions_monthly_summary
ON transactions(transaction_type, currency_code, strftime('%Y-%m', processed_date))
WHERE processed_date IS NOT NULL AND status = 'Completed';

-- Account balance changes by month
CREATE INDEX IF NOT EXISTS idx_account_balances_monthly
ON account_balances(account_id, currency_code, strftime('%Y-%m', last_updated));

-- Overdue loan payments
CREATE INDEX IF NOT EXISTS idx_overdue_loan_payments
ON loan_payments(loan_id, due_date, status)
WHERE due_date < date('now') AND status IN ('Pending', 'Partial');

-- Active recurring transactions due for execution
CREATE INDEX IF NOT EXISTS idx_recurring_due_execution
ON recurring_transactions(next_execution_date, is_active)
WHERE is_active = TRUE AND next_execution_date <= date('now');

-- Recent high-value transactions
CREATE INDEX IF NOT EXISTS idx_high_value_transactions
ON transactions(amount, processed_date, status)
WHERE status = 'Completed' AND processed_date >= date('now', '-30 days');

-- Foreign exchange transactions
CREATE INDEX IF NOT EXISTS idx_fx_transactions
ON transactions(transaction_type, currency_code, converted_currency)
WHERE transaction_type = 'CurrencyExchange';

-- Account activity by date range
CREATE INDEX IF NOT EXISTS idx_account_activity_range
ON transactions(from_account_id, processed_date, status)
WHERE processed_date IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_account_activity_range_to
ON transactions(to_account_id, processed_date, status)
WHERE processed_date IS NOT NULL;

-- Loan payment history
CREATE INDEX IF NOT EXISTS idx_loan_payment_history
ON loan_payments(loan_id, payment_date, status)
WHERE payment_date IS NOT NULL;

-- Currency exchange rate lookups
CREATE INDEX IF NOT EXISTS idx_latest_exchange_rates
ON exchange_rates(base_currency, target_currency, effective_date DESC, is_active)
WHERE is_active = TRUE;

-- Text search indexes (for description and reference searches)
-- Note: SQLite FTS would be better for full-text search, but these provide basic support
CREATE INDEX IF NOT EXISTS idx_transactions_description_search
ON transactions(description)
WHERE description IS NOT NULL AND LENGTH(description) > 0;

CREATE INDEX IF NOT EXISTS idx_accounts_name_search
ON accounts(name);

CREATE INDEX IF NOT EXISTS idx_loans_description_search
ON loans(description)
WHERE description IS NOT NULL AND LENGTH(description) > 0;