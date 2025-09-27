-- Migration 001: Create core financial tables
-- This migration creates all the fundamental tables for the FerrumFinance system

-- Currencies table
CREATE TABLE IF NOT EXISTS currencies (
    code TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    decimal_places INTEGER NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CHECK (decimal_places >= 0 AND decimal_places <= 8),
    CHECK (LENGTH(code) = 3),
    CHECK (LENGTH(name) > 0),
    CHECK (LENGTH(symbol) > 0)
);

-- Insert default currencies
INSERT OR IGNORE INTO currencies (code, name, symbol, decimal_places) VALUES
('USD', 'US Dollar', '$', 2),
('EUR', 'Euro', '€', 2),
('GBP', 'British Pound', '£', 2),
('JPY', 'Japanese Yen', '¥', 0),
('CHF', 'Swiss Franc', 'Fr', 2),
('CAD', 'Canadian Dollar', 'C$', 2),
('AUD', 'Australian Dollar', 'A$', 2),
('CNY', 'Chinese Yuan', '¥', 2),
('INR', 'Indian Rupee', '₹', 2),
('BTC', 'Bitcoin', '₿', 8);

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    account_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Active',
    description TEXT,
    parent_account_id TEXT,
    default_currency TEXT NOT NULL DEFAULT 'USD',
    institution_name TEXT,
    account_number TEXT,
    routing_number TEXT,
    is_closed BOOLEAN NOT NULL DEFAULT FALSE,
    closed_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (parent_account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (default_currency) REFERENCES currencies(code) ON DELETE RESTRICT,

    CHECK (account_type IN ('Asset', 'Liability', 'Equity', 'Revenue', 'Expense')),
    CHECK (status IN ('Active', 'Suspended', 'Closed', 'Pending')),
    CHECK (LENGTH(name) > 0),
    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK ((is_closed = FALSE AND closed_at IS NULL) OR (is_closed = TRUE AND closed_at IS NOT NULL))
);

-- Account balances table for multi-currency support
CREATE TABLE IF NOT EXISTS account_balances (
    account_id TEXT NOT NULL,
    currency_code TEXT NOT NULL,
    balance DECIMAL(20,8) NOT NULL DEFAULT 0,
    available_balance DECIMAL(20,8) NOT NULL DEFAULT 0,
    pending_balance DECIMAL(20,8) NOT NULL DEFAULT 0,
    last_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (account_id, currency_code),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (currency_code) REFERENCES currencies(code) ON DELETE RESTRICT,

    CHECK (balance >= -999999999999.99999999 AND balance <= 999999999999.99999999),
    CHECK (available_balance >= -999999999999.99999999 AND available_balance <= 999999999999.99999999),
    CHECK (pending_balance >= -999999999999.99999999 AND pending_balance <= 999999999999.99999999)
);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id TEXT PRIMARY KEY NOT NULL,
    transaction_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    priority TEXT NOT NULL DEFAULT 'Normal',
    from_account_id TEXT,
    to_account_id TEXT,
    amount DECIMAL(20,8) NOT NULL,
    currency_code TEXT NOT NULL,
    exchange_rate DECIMAL(15,8),
    converted_amount DECIMAL(20,8),
    converted_currency TEXT,
    description TEXT,
    reference_number TEXT,
    external_id TEXT,
    category TEXT,
    tags TEXT, -- JSON array of strings
    metadata TEXT, -- JSON object for additional data
    scheduled_date DATETIME,
    processed_date DATETIME,
    settlement_date DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (from_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (to_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (currency_code) REFERENCES currencies(code) ON DELETE RESTRICT,
    FOREIGN KEY (converted_currency) REFERENCES currencies(code) ON DELETE RESTRICT,

    CHECK (transaction_type IN ('Deposit', 'Withdrawal', 'Transfer', 'Payment', 'Income',
                               'Expense', 'LoanDisbursement', 'LoanPayment', 'Interest',
                               'Fee', 'Dividend', 'Investment', 'CurrencyExchange',
                               'Adjustment', 'Refund')),
    CHECK (status IN ('Pending', 'Processing', 'Completed', 'Failed', 'Cancelled',
                     'Scheduled', 'Declined')),
    CHECK (priority IN ('Low', 'Normal', 'High', 'Urgent')),
    CHECK (amount > 0),
    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (from_account_id IS NOT NULL OR to_account_id IS NOT NULL), -- At least one account
    CHECK (exchange_rate IS NULL OR exchange_rate > 0),
    CHECK (converted_amount IS NULL OR converted_amount > 0)
);

-- Recurring transactions table
CREATE TABLE IF NOT EXISTS recurring_transactions (
    id TEXT PRIMARY KEY NOT NULL,
    template_transaction_id TEXT NOT NULL,
    pattern TEXT NOT NULL,
    frequency TEXT NOT NULL,
    interval_count INTEGER NOT NULL DEFAULT 1,
    start_date DATE NOT NULL,
    end_date DATE,
    max_occurrences INTEGER,
    current_occurrences INTEGER NOT NULL DEFAULT 0,
    next_execution_date DATE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (template_transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,

    CHECK (pattern IN ('Fixed', 'Flexible')),
    CHECK (frequency IN ('Daily', 'Weekly', 'Monthly', 'Quarterly', 'Yearly')),
    CHECK (interval_count > 0),
    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (max_occurrences IS NULL OR max_occurrences > 0),
    CHECK (current_occurrences >= 0),
    CHECK (end_date IS NULL OR end_date >= start_date)
);

-- Exchange rates table
CREATE TABLE IF NOT EXISTS exchange_rates (
    id TEXT PRIMARY KEY NOT NULL,
    base_currency TEXT NOT NULL,
    target_currency TEXT NOT NULL,
    rate DECIMAL(15,8) NOT NULL,
    inverse_rate DECIMAL(15,8) NOT NULL,
    source TEXT NOT NULL,
    effective_date DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    FOREIGN KEY (base_currency) REFERENCES currencies(code) ON DELETE RESTRICT,
    FOREIGN KEY (target_currency) REFERENCES currencies(code) ON DELETE RESTRICT,

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (rate > 0),
    CHECK (inverse_rate > 0),
    CHECK (base_currency != target_currency),
    CHECK (source IN ('Manual', 'API', 'Bank', 'Market', 'Central Bank')),
    CHECK (LENGTH(source) > 0)
);

-- Loans table
CREATE TABLE IF NOT EXISTS loans (
    id TEXT PRIMARY KEY NOT NULL,
    loan_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Pending',
    borrower_account_id TEXT NOT NULL,
    lender_account_id TEXT,
    principal_amount DECIMAL(20,8) NOT NULL,
    currency_code TEXT NOT NULL,
    interest_rate DECIMAL(8,6) NOT NULL,
    interest_rate_type TEXT NOT NULL DEFAULT 'Fixed',
    term_months INTEGER NOT NULL,
    payment_frequency TEXT NOT NULL DEFAULT 'Monthly',
    payment_calculation_method TEXT NOT NULL DEFAULT 'EqualPayments',
    payment_amount DECIMAL(20,8),
    balloon_payment DECIMAL(20,8),
    origination_date DATE NOT NULL,
    first_payment_date DATE NOT NULL,
    maturity_date DATE NOT NULL,
    description TEXT,
    collateral_description TEXT,
    guarantor_info TEXT, -- JSON object
    terms_and_conditions TEXT,
    is_compound_interest BOOLEAN NOT NULL DEFAULT TRUE,
    grace_period_days INTEGER DEFAULT 0,
    late_fee_percentage DECIMAL(5,4) DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (borrower_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (lender_account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (currency_code) REFERENCES currencies(code) ON DELETE RESTRICT,

    CHECK (loan_type IN ('Personal', 'Mortgage', 'Auto', 'Business', 'Student', 'Line of Credit')),
    CHECK (status IN ('Pending', 'Active', 'Paid Off', 'Defaulted', 'Cancelled')),
    CHECK (interest_rate_type IN ('Fixed', 'Variable', 'Floating')),
    CHECK (payment_frequency IN ('Weekly', 'BiWeekly', 'Monthly', 'Quarterly', 'SemiAnnually', 'Annually')),
    CHECK (payment_calculation_method IN ('EqualPayments', 'EqualPrincipal', 'InterestOnly', 'BalloonPayment')),
    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (principal_amount > 0),
    CHECK (interest_rate >= 0 AND interest_rate <= 1), -- 0% to 100%
    CHECK (term_months > 0),
    CHECK (payment_amount IS NULL OR payment_amount > 0),
    CHECK (balloon_payment IS NULL OR balloon_payment >= 0),
    CHECK (grace_period_days >= 0),
    CHECK (late_fee_percentage >= 0 AND late_fee_percentage <= 1), -- 0% to 100%
    CHECK (first_payment_date >= origination_date),
    CHECK (maturity_date > origination_date)
);

-- Loan payments table
CREATE TABLE IF NOT EXISTS loan_payments (
    id TEXT PRIMARY KEY NOT NULL,
    loan_id TEXT NOT NULL,
    payment_number INTEGER NOT NULL,
    due_date DATE NOT NULL,
    amount_due DECIMAL(20,8) NOT NULL,
    principal_due DECIMAL(20,8) NOT NULL,
    interest_due DECIMAL(20,8) NOT NULL,
    fees_due DECIMAL(20,8) NOT NULL DEFAULT 0,
    amount_paid DECIMAL(20,8) NOT NULL DEFAULT 0,
    principal_paid DECIMAL(20,8) NOT NULL DEFAULT 0,
    interest_paid DECIMAL(20,8) NOT NULL DEFAULT 0,
    fees_paid DECIMAL(20,8) NOT NULL DEFAULT 0,
    payment_date DATE,
    status TEXT NOT NULL DEFAULT 'Pending',
    late_fee DECIMAL(20,8) NOT NULL DEFAULT 0,
    remaining_balance DECIMAL(20,8) NOT NULL,
    transaction_id TEXT,
    notes TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (loan_id) REFERENCES loans(id) ON DELETE CASCADE,
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE SET NULL,

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (payment_number > 0),
    CHECK (amount_due >= 0),
    CHECK (principal_due >= 0),
    CHECK (interest_due >= 0),
    CHECK (fees_due >= 0),
    CHECK (amount_paid >= 0),
    CHECK (principal_paid >= 0),
    CHECK (interest_paid >= 0),
    CHECK (fees_paid >= 0),
    CHECK (late_fee >= 0),
    CHECK (remaining_balance >= 0),
    CHECK (status IN ('Pending', 'Partial', 'Paid', 'Late', 'Missed')),
    CHECK (payment_date IS NULL OR payment_date >= due_date OR status != 'Pending')
);

-- Account hierarchy view
CREATE VIEW IF NOT EXISTS account_hierarchy AS
WITH RECURSIVE account_tree(id, name, account_type, parent_id, level, path) AS (
    -- Base case: root accounts
    SELECT id, name, account_type, parent_account_id, 0, name
    FROM accounts
    WHERE parent_account_id IS NULL

    UNION ALL

    -- Recursive case: child accounts
    SELECT a.id, a.name, a.account_type, a.parent_account_id,
           at.level + 1, at.path || ' > ' || a.name
    FROM accounts a
    INNER JOIN account_tree at ON a.parent_account_id = at.id
)
SELECT * FROM account_tree;

-- Account summary view
CREATE VIEW IF NOT EXISTS account_summary AS
SELECT
    a.id,
    a.name,
    a.account_type,
    a.status,
    a.default_currency,
    COALESCE(ab.balance, 0) as balance,
    COALESCE(ab.available_balance, 0) as available_balance,
    COALESCE(ab.pending_balance, 0) as pending_balance,
    a.created_at,
    a.updated_at
FROM accounts a
LEFT JOIN account_balances ab ON a.id = ab.account_id AND ab.currency_code = a.default_currency;

-- Transaction summary view
CREATE VIEW IF NOT EXISTS transaction_summary AS
SELECT
    t.id,
    t.transaction_type,
    t.status,
    t.amount,
    t.currency_code,
    fa.name as from_account_name,
    ta.name as to_account_name,
    t.description,
    t.processed_date,
    t.created_at
FROM transactions t
LEFT JOIN accounts fa ON t.from_account_id = fa.id
LEFT JOIN accounts ta ON t.to_account_id = ta.id;