-- Migration 004: Create views and analytical functions
-- This migration creates comprehensive views for reporting and analytics

-- Account balance summary with multi-currency support
CREATE VIEW IF NOT EXISTS v_account_balance_summary AS
SELECT
    a.id as account_id,
    a.name as account_name,
    a.account_type,
    a.status,
    ab.currency_code,
    c.symbol as currency_symbol,
    ab.balance,
    ab.available_balance,
    ab.pending_balance,
    CASE
        WHEN ab.balance > 0 THEN 'Credit'
        WHEN ab.balance < 0 THEN 'Debit'
        ELSE 'Zero'
    END as balance_type,
    ab.last_updated as balance_last_updated,
    a.created_at as account_created_at
FROM accounts a
LEFT JOIN account_balances ab ON a.id = ab.account_id
LEFT JOIN currencies c ON ab.currency_code = c.code
WHERE a.status = 'Active'
ORDER BY a.account_type, a.name, ab.currency_code;

-- Transaction analytics view
CREATE VIEW IF NOT EXISTS v_transaction_analytics AS
SELECT
    t.id,
    t.transaction_type,
    t.status,
    t.amount,
    t.currency_code,
    c.symbol as currency_symbol,
    fa.name as from_account_name,
    fa.account_type as from_account_type,
    ta.name as to_account_name,
    ta.account_type as to_account_type,
    t.description,
    t.category,
    t.reference_number,
    t.processed_date,
    t.created_at,
    strftime('%Y-%m', t.processed_date) as processed_month,
    strftime('%Y', t.processed_date) as processed_year,
    strftime('%w', t.processed_date) as day_of_week,
    CASE
        WHEN t.amount < 100 THEN 'Small'
        WHEN t.amount < 1000 THEN 'Medium'
        WHEN t.amount < 10000 THEN 'Large'
        ELSE 'Very Large'
    END as amount_category,
    julianday('now') - julianday(t.processed_date) as days_since_processed
FROM transactions t
LEFT JOIN accounts fa ON t.from_account_id = fa.id
LEFT JOIN accounts ta ON t.to_account_id = ta.id
LEFT JOIN currencies c ON t.currency_code = c.code
WHERE t.status = 'Completed';

-- Monthly transaction summary
CREATE VIEW IF NOT EXISTS v_monthly_transaction_summary AS
SELECT
    strftime('%Y-%m', processed_date) as month,
    transaction_type,
    currency_code,
    COUNT(*) as transaction_count,
    SUM(amount) as total_amount,
    AVG(amount) as average_amount,
    MIN(amount) as min_amount,
    MAX(amount) as max_amount,
    COUNT(DISTINCT from_account_id) as unique_from_accounts,
    COUNT(DISTINCT to_account_id) as unique_to_accounts
FROM transactions
WHERE status = 'Completed'
  AND processed_date IS NOT NULL
GROUP BY strftime('%Y-%m', processed_date), transaction_type, currency_code
ORDER BY month DESC, transaction_type;

-- Account activity summary
CREATE VIEW IF NOT EXISTS v_account_activity_summary AS
SELECT
    a.id as account_id,
    a.name as account_name,
    a.account_type,
    COUNT(DISTINCT t1.id) as outgoing_transactions,
    COUNT(DISTINCT t2.id) as incoming_transactions,
    COUNT(DISTINCT t1.id) + COUNT(DISTINCT t2.id) as total_transactions,
    COALESCE(SUM(CASE WHEN t1.status = 'Completed' THEN t1.amount ELSE 0 END), 0) as total_outgoing,
    COALESCE(SUM(CASE WHEN t2.status = 'Completed' THEN t2.amount ELSE 0 END), 0) as total_incoming,
    COALESCE(MAX(t1.processed_date), MAX(t2.processed_date)) as last_transaction_date,
    julianday('now') - julianday(COALESCE(MAX(t1.processed_date), MAX(t2.processed_date))) as days_since_last_activity
FROM accounts a
LEFT JOIN transactions t1 ON a.id = t1.from_account_id AND t1.processed_date >= date('now', '-90 days')
LEFT JOIN transactions t2 ON a.id = t2.to_account_id AND t2.processed_date >= date('now', '-90 days')
WHERE a.status = 'Active'
GROUP BY a.id, a.name, a.account_type;

-- Loan portfolio summary
CREATE VIEW IF NOT EXISTS v_loan_portfolio_summary AS
SELECT
    l.id as loan_id,
    l.loan_type,
    l.status,
    l.borrower_account_id,
    ba.name as borrower_name,
    l.principal_amount,
    l.currency_code,
    c.symbol as currency_symbol,
    l.interest_rate,
    l.interest_rate_type,
    l.term_months,
    l.origination_date,
    l.maturity_date,
    COALESCE(SUM(lp.amount_paid), 0) as total_paid,
    COALESCE(SUM(lp.principal_paid), 0) as principal_paid,
    COALESCE(SUM(lp.interest_paid), 0) as interest_paid,
    l.principal_amount - COALESCE(SUM(lp.principal_paid), 0) as remaining_balance,
    COUNT(lp.id) as total_payments_made,
    COUNT(CASE WHEN lp.status = 'Paid' THEN 1 END) as payments_made,
    COUNT(CASE WHEN lp.status = 'Late' THEN 1 END) as late_payments,
    COUNT(CASE WHEN lp.status = 'Missed' THEN 1 END) as missed_payments,
    MAX(lp.due_date) as last_payment_due,
    MIN(CASE WHEN lp.status IN ('Pending', 'Partial') THEN lp.due_date END) as next_payment_due,
    CASE
        WHEN l.status = 'Paid Off' THEN 'Paid Off'
        WHEN l.status = 'Defaulted' THEN 'Defaulted'
        WHEN MIN(CASE WHEN lp.status IN ('Pending', 'Partial') THEN lp.due_date END) < date('now') THEN 'Overdue'
        WHEN MIN(CASE WHEN lp.status IN ('Pending', 'Partial') THEN lp.due_date END) <= date('now', '+7 days') THEN 'Due Soon'
        ELSE 'Current'
    END as loan_status_detail,
    julianday(l.maturity_date) - julianday('now') as days_until_maturity
FROM loans l
LEFT JOIN accounts ba ON l.borrower_account_id = ba.id
LEFT JOIN currencies c ON l.currency_code = c.code
LEFT JOIN loan_payments lp ON l.id = lp.loan_id
GROUP BY l.id, l.loan_type, l.status, l.borrower_account_id, ba.name,
         l.principal_amount, l.currency_code, c.symbol, l.interest_rate,
         l.interest_rate_type, l.term_months, l.origination_date, l.maturity_date;

-- Exchange rate trends
CREATE VIEW IF NOT EXISTS v_exchange_rate_trends AS
SELECT
    er.base_currency,
    er.target_currency,
    er.rate as current_rate,
    er.effective_date as current_date,
    prev.rate as previous_rate,
    prev.effective_date as previous_date,
    CASE
        WHEN prev.rate IS NULL THEN 0
        ELSE (er.rate - prev.rate) / prev.rate * 100
    END as percentage_change,
    CASE
        WHEN prev.rate IS NULL THEN 'New'
        WHEN er.rate > prev.rate THEN 'Strengthening'
        WHEN er.rate < prev.rate THEN 'Weakening'
        ELSE 'Stable'
    END as trend_direction,
    julianday(er.effective_date) - julianday(prev.effective_date) as days_between_rates
FROM exchange_rates er
LEFT JOIN exchange_rates prev ON er.base_currency = prev.base_currency
    AND er.target_currency = prev.target_currency
    AND prev.effective_date = (
        SELECT MAX(effective_date)
        FROM exchange_rates
        WHERE base_currency = er.base_currency
          AND target_currency = er.target_currency
          AND effective_date < er.effective_date
          AND is_active = TRUE
    )
WHERE er.is_active = TRUE;

-- Financial health indicators
CREATE VIEW IF NOT EXISTS v_financial_health_indicators AS
SELECT
    'Total Assets' as indicator_name,
    COALESCE(SUM(CASE WHEN a.account_type = 'Asset' THEN ab.balance ELSE 0 END), 0) as amount,
    'USD' as currency, -- Assuming USD as base currency
    'Assets' as category
FROM accounts a
LEFT JOIN account_balances ab ON a.id = ab.account_id
WHERE a.status = 'Active' AND ab.currency_code = 'USD'

UNION ALL

SELECT
    'Total Liabilities' as indicator_name,
    COALESCE(SUM(CASE WHEN a.account_type = 'Liability' THEN ab.balance ELSE 0 END), 0) as amount,
    'USD' as currency,
    'Liabilities' as category
FROM accounts a
LEFT JOIN account_balances ab ON a.id = ab.account_id
WHERE a.status = 'Active' AND ab.currency_code = 'USD'

UNION ALL

SELECT
    'Net Worth' as indicator_name,
    COALESCE(SUM(CASE
        WHEN a.account_type = 'Asset' THEN ab.balance
        WHEN a.account_type = 'Liability' THEN -ab.balance
        ELSE 0
    END), 0) as amount,
    'USD' as currency,
    'Net Worth' as category
FROM accounts a
LEFT JOIN account_balances ab ON a.id = ab.account_id
WHERE a.status = 'Active' AND ab.currency_code = 'USD'

UNION ALL

SELECT
    'Active Loans Outstanding' as indicator_name,
    COALESCE(SUM(l.principal_amount - COALESCE(lp_summary.principal_paid, 0)), 0) as amount,
    'USD' as currency,
    'Loans' as category
FROM loans l
LEFT JOIN (
    SELECT loan_id, SUM(principal_paid) as principal_paid
    FROM loan_payments
    GROUP BY loan_id
) lp_summary ON l.id = lp_summary.loan_id
WHERE l.status = 'Active' AND l.currency_code = 'USD'

UNION ALL

SELECT
    'Monthly Transaction Volume' as indicator_name,
    COALESCE(SUM(amount), 0) as amount,
    'USD' as currency,
    'Activity' as category
FROM transactions
WHERE status = 'Completed'
  AND currency_code = 'USD'
  AND processed_date >= date('now', 'start of month');

-- Risk assessment view
CREATE VIEW IF NOT EXISTS v_risk_assessment AS
SELECT
    'Account Concentration Risk' as risk_type,
    CASE
        WHEN MAX(account_balance_pct) > 50 THEN 'High'
        WHEN MAX(account_balance_pct) > 30 THEN 'Medium'
        ELSE 'Low'
    END as risk_level,
    'Single account holds ' || ROUND(MAX(account_balance_pct), 1) || '% of total assets' as description,
    MAX(account_balance_pct) as risk_value
FROM (
    SELECT
        a.name,
        ab.balance,
        ab.balance * 100.0 / SUM(ab.balance) OVER () as account_balance_pct
    FROM accounts a
    JOIN account_balances ab ON a.id = ab.account_id
    WHERE a.account_type = 'Asset'
      AND a.status = 'Active'
      AND ab.currency_code = 'USD'
      AND ab.balance > 0
) account_concentration

UNION ALL

SELECT
    'Loan Default Risk' as risk_type,
    CASE
        WHEN overdue_loan_pct > 10 THEN 'High'
        WHEN overdue_loan_pct > 5 THEN 'Medium'
        ELSE 'Low'
    END as risk_level,
    ROUND(overdue_loan_pct, 1) || '% of loans are overdue' as description,
    overdue_loan_pct as risk_value
FROM (
    SELECT
        COUNT(CASE WHEN loan_status_detail IN ('Overdue', 'Defaulted') THEN 1 END) * 100.0 / COUNT(*) as overdue_loan_pct
    FROM v_loan_portfolio_summary
    WHERE status IN ('Active', 'Defaulted')
) loan_risk

UNION ALL

SELECT
    'Currency Concentration Risk' as risk_type,
    CASE
        WHEN MAX(currency_pct) > 80 THEN 'High'
        WHEN MAX(currency_pct) > 60 THEN 'Medium'
        ELSE 'Low'
    END as risk_level,
    'Single currency represents ' || ROUND(MAX(currency_pct), 1) || '% of total balance' as description,
    MAX(currency_pct) as risk_value
FROM (
    SELECT
        ab.currency_code,
        SUM(ab.balance) as total_balance,
        SUM(ab.balance) * 100.0 / SUM(SUM(ab.balance)) OVER () as currency_pct
    FROM account_balances ab
    JOIN accounts a ON ab.account_id = a.id
    WHERE a.status = 'Active' AND ab.balance > 0
    GROUP BY ab.currency_code
) currency_concentration;

-- Performance metrics view
CREATE VIEW IF NOT EXISTS v_performance_metrics AS
SELECT
    'Transaction Processing Time' as metric_name,
    AVG(julianday(processed_date) - julianday(created_at)) * 24 as avg_hours,
    'hours' as unit,
    'Operational' as category
FROM transactions
WHERE processed_date IS NOT NULL
  AND created_at IS NOT NULL
  AND processed_date >= date('now', '-30 days')

UNION ALL

SELECT
    'Daily Transaction Volume' as metric_name,
    AVG(daily_count) as avg_hours,
    'transactions' as unit,
    'Volume' as category
FROM (
    SELECT
        date(processed_date) as transaction_date,
        COUNT(*) as daily_count
    FROM transactions
    WHERE processed_date >= date('now', '-30 days')
      AND status = 'Completed'
    GROUP BY date(processed_date)
) daily_volumes

UNION ALL

SELECT
    'Average Transaction Amount' as metric_name,
    AVG(amount) as avg_hours,
    currency_code as unit,
    'Financial' as category
FROM transactions
WHERE processed_date >= date('now', '-30 days')
  AND status = 'Completed'
  AND currency_code = 'USD'

UNION ALL

SELECT
    'Loan Payment Success Rate' as metric_name,
    COUNT(CASE WHEN status = 'Paid' THEN 1 END) * 100.0 / COUNT(*) as avg_hours,
    'percentage' as unit,
    'Loans' as category
FROM loan_payments
WHERE due_date >= date('now', '-30 days');

-- Audit summary view
CREATE VIEW IF NOT EXISTS v_audit_summary AS
SELECT
    'Data Changes' as audit_category,
    COUNT(*) as total_events,
    COUNT(DISTINCT table_name) as affected_tables,
    COUNT(DISTINCT changed_by) as unique_users,
    MIN(changed_at) as earliest_change,
    MAX(changed_at) as latest_change
FROM audit_trail
WHERE changed_at >= date('now', '-30 days')

UNION ALL

SELECT
    'Financial Actions' as audit_category,
    COUNT(*) as total_events,
    COUNT(DISTINCT entity_type) as affected_tables,
    COUNT(DISTINCT performed_by) as unique_users,
    MIN(performed_at) as earliest_change,
    MAX(performed_at) as latest_change
FROM audit_financial_actions
WHERE performed_at >= date('now', '-30 days')

UNION ALL

SELECT
    'Security Events' as audit_category,
    COUNT(*) as total_events,
    COUNT(DISTINCT event_type) as affected_tables,
    COUNT(DISTINCT user_id) as unique_users,
    MIN(event_timestamp) as earliest_change,
    MAX(event_timestamp) as latest_change
FROM audit_security
WHERE event_timestamp >= date('now', '-30 days');

-- Cash flow analysis view
CREATE VIEW IF NOT EXISTS v_cash_flow_analysis AS
SELECT
    strftime('%Y-%m', processed_date) as month,
    currency_code,
    SUM(CASE
        WHEN transaction_type IN ('Deposit', 'Income', 'LoanDisbursement') THEN amount
        ELSE 0
    END) as cash_inflow,
    SUM(CASE
        WHEN transaction_type IN ('Withdrawal', 'Payment', 'Expense', 'LoanPayment') THEN amount
        ELSE 0
    END) as cash_outflow,
    SUM(CASE
        WHEN transaction_type IN ('Deposit', 'Income', 'LoanDisbursement') THEN amount
        WHEN transaction_type IN ('Withdrawal', 'Payment', 'Expense', 'LoanPayment') THEN -amount
        ELSE 0
    END) as net_cash_flow,
    COUNT(*) as total_transactions
FROM transactions
WHERE status = 'Completed'
  AND processed_date IS NOT NULL
  AND processed_date >= date('now', '-12 months')
GROUP BY strftime('%Y-%m', processed_date), currency_code
ORDER BY month DESC, currency_code;