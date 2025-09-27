-- Migration 003: Create audit trail tables
-- This migration creates comprehensive audit trail functionality

-- Audit trail table for tracking all data changes
CREATE TABLE IF NOT EXISTS audit_trail (
    id TEXT PRIMARY KEY NOT NULL,
    table_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    operation TEXT NOT NULL,
    old_values TEXT, -- JSON representation of old record values
    new_values TEXT, -- JSON representation of new record values
    changed_by TEXT, -- User ID or system identifier
    changed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reason TEXT, -- Optional reason for the change
    session_id TEXT, -- Session identifier for grouping related changes
    ip_address TEXT, -- IP address of the user making changes
    user_agent TEXT, -- Browser/client information

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (operation IN ('INSERT', 'UPDATE', 'DELETE')),
    CHECK (LENGTH(table_name) > 0),
    CHECK (LENGTH(record_id) > 0)
);

-- Login/session audit table
CREATE TABLE IF NOT EXISTS audit_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT,
    session_start DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    session_end DATETIME,
    ip_address TEXT,
    user_agent TEXT,
    login_method TEXT,
    status TEXT NOT NULL DEFAULT 'Active',
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    last_activity DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (status IN ('Active', 'Expired', 'Terminated', 'Failed')),
    CHECK (failed_attempts >= 0),
    CHECK (session_end IS NULL OR session_end >= session_start)
);

-- Financial actions audit (for compliance and risk management)
CREATE TABLE IF NOT EXISTS audit_financial_actions (
    id TEXT PRIMARY KEY NOT NULL,
    action_type TEXT NOT NULL,
    entity_type TEXT NOT NULL, -- 'transaction', 'account', 'loan', etc.
    entity_id TEXT NOT NULL,
    amount DECIMAL(20,8),
    currency_code TEXT,
    risk_score DECIMAL(3,2), -- 0.00 to 1.00
    compliance_flags TEXT, -- JSON array of compliance flags
    performed_by TEXT,
    performed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    approved_by TEXT,
    approved_at DATETIME,
    notes TEXT,
    metadata TEXT, -- JSON object for additional context

    FOREIGN KEY (currency_code) REFERENCES currencies(code) ON DELETE RESTRICT,

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (action_type IN ('CREATE', 'UPDATE', 'DELETE', 'APPROVE', 'REJECT', 'SUSPEND', 'ACTIVATE')),
    CHECK (entity_type IN ('account', 'transaction', 'loan', 'exchange_rate', 'recurring_transaction')),
    CHECK (risk_score IS NULL OR (risk_score >= 0.00 AND risk_score <= 1.00)),
    CHECK (amount IS NULL OR amount >= 0),
    CHECK (approved_at IS NULL OR approved_at >= performed_at)
);

-- Data integrity audit table
CREATE TABLE IF NOT EXISTS audit_data_integrity (
    id TEXT PRIMARY KEY NOT NULL,
    table_name TEXT NOT NULL,
    check_type TEXT NOT NULL,
    check_description TEXT NOT NULL,
    status TEXT NOT NULL,
    issues_found INTEGER NOT NULL DEFAULT 0,
    issues_resolved INTEGER NOT NULL DEFAULT 0,
    details TEXT, -- JSON object with detailed results
    checked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    resolved_at DATETIME,

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (check_type IN ('BALANCE_RECONCILIATION', 'REFERENTIAL_INTEGRITY', 'BUSINESS_RULES', 'DATA_CONSISTENCY')),
    CHECK (status IN ('PASSED', 'FAILED', 'WARNING', 'RESOLVED')),
    CHECK (issues_found >= 0),
    CHECK (issues_resolved >= 0 AND issues_resolved <= issues_found)
);

-- Security audit table
CREATE TABLE IF NOT EXISTS audit_security (
    id TEXT PRIMARY KEY NOT NULL,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    description TEXT NOT NULL,
    source_ip TEXT,
    user_id TEXT,
    session_id TEXT,
    resource_accessed TEXT,
    action_attempted TEXT,
    success BOOLEAN NOT NULL,
    failure_reason TEXT,
    event_timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    detection_method TEXT, -- How this event was detected
    response_action TEXT, -- What action was taken in response

    FOREIGN KEY (session_id) REFERENCES audit_sessions(id) ON DELETE SET NULL,

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (event_type IN ('LOGIN', 'LOGOUT', 'ACCESS_DENIED', 'PRIVILEGE_ESCALATION',
                         'DATA_ACCESS', 'CONFIGURATION_CHANGE', 'SUSPICIOUS_ACTIVITY')),
    CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    CHECK (LENGTH(description) > 0)
);

-- Backup and restore audit table
CREATE TABLE IF NOT EXISTS audit_backup_restore (
    id TEXT PRIMARY KEY NOT NULL,
    operation_type TEXT NOT NULL,
    backup_file_path TEXT,
    file_size INTEGER,
    checksum TEXT,
    started_at DATETIME NOT NULL,
    completed_at DATETIME,
    status TEXT NOT NULL,
    error_message TEXT,
    performed_by TEXT,
    verification_status TEXT,
    retention_policy TEXT,

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (operation_type IN ('BACKUP', 'RESTORE', 'VERIFY', 'CLEANUP')),
    CHECK (status IN ('STARTED', 'IN_PROGRESS', 'COMPLETED', 'FAILED', 'CANCELLED')),
    CHECK (verification_status IN ('PENDING', 'PASSED', 'FAILED', 'SKIPPED') OR verification_status IS NULL),
    CHECK (file_size IS NULL OR file_size >= 0),
    CHECK (completed_at IS NULL OR completed_at >= started_at)
);

-- Configuration changes audit
CREATE TABLE IF NOT EXISTS audit_configuration (
    id TEXT PRIMARY KEY NOT NULL,
    configuration_key TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    changed_by TEXT,
    changed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    change_reason TEXT,
    environment TEXT NOT NULL DEFAULT 'production',
    approval_required BOOLEAN NOT NULL DEFAULT FALSE,
    approved_by TEXT,
    approved_at DATETIME,

    CHECK (LENGTH(id) = 36), -- UUID format
    CHECK (LENGTH(configuration_key) > 0),
    CHECK (environment IN ('development', 'testing', 'staging', 'production')),
    CHECK (approved_at IS NULL OR approved_at >= changed_at)
);

-- Indexes for audit tables (optimized for common queries)

-- Audit trail indexes
CREATE INDEX IF NOT EXISTS idx_audit_trail_table ON audit_trail(table_name);
CREATE INDEX IF NOT EXISTS idx_audit_trail_record ON audit_trail(table_name, record_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_operation ON audit_trail(operation);
CREATE INDEX IF NOT EXISTS idx_audit_trail_changed_at ON audit_trail(changed_at);
CREATE INDEX IF NOT EXISTS idx_audit_trail_changed_by ON audit_trail(changed_by) WHERE changed_by IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_trail_session ON audit_trail(session_id) WHERE session_id IS NOT NULL;

-- Composite indexes for common audit queries
CREATE INDEX IF NOT EXISTS idx_audit_trail_table_date ON audit_trail(table_name, changed_at);
CREATE INDEX IF NOT EXISTS idx_audit_trail_record_date ON audit_trail(table_name, record_id, changed_at);
CREATE INDEX IF NOT EXISTS idx_audit_trail_user_date ON audit_trail(changed_by, changed_at) WHERE changed_by IS NOT NULL;

-- Session audit indexes
CREATE INDEX IF NOT EXISTS idx_audit_sessions_user ON audit_sessions(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_sessions_start ON audit_sessions(session_start);
CREATE INDEX IF NOT EXISTS idx_audit_sessions_status ON audit_sessions(status);
CREATE INDEX IF NOT EXISTS idx_audit_sessions_activity ON audit_sessions(last_activity);
CREATE INDEX IF NOT EXISTS idx_audit_sessions_active ON audit_sessions(status, last_activity) WHERE status = 'Active';

-- Financial actions audit indexes
CREATE INDEX IF NOT EXISTS idx_audit_financial_entity ON audit_financial_actions(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_financial_type ON audit_financial_actions(action_type);
CREATE INDEX IF NOT EXISTS idx_audit_financial_performed_at ON audit_financial_actions(performed_at);
CREATE INDEX IF NOT EXISTS idx_audit_financial_performed_by ON audit_financial_actions(performed_by) WHERE performed_by IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_financial_amount ON audit_financial_actions(amount) WHERE amount IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_financial_risk ON audit_financial_actions(risk_score) WHERE risk_score IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_financial_pending_approval ON audit_financial_actions(approved_at) WHERE approved_at IS NULL;

-- Data integrity audit indexes
CREATE INDEX IF NOT EXISTS idx_audit_integrity_table ON audit_data_integrity(table_name);
CREATE INDEX IF NOT EXISTS idx_audit_integrity_type ON audit_data_integrity(check_type);
CREATE INDEX IF NOT EXISTS idx_audit_integrity_status ON audit_data_integrity(status);
CREATE INDEX IF NOT EXISTS idx_audit_integrity_checked_at ON audit_data_integrity(checked_at);
CREATE INDEX IF NOT EXISTS idx_audit_integrity_failed ON audit_data_integrity(status, checked_at) WHERE status = 'FAILED';

-- Security audit indexes
CREATE INDEX IF NOT EXISTS idx_audit_security_event_type ON audit_security(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_security_severity ON audit_security(severity);
CREATE INDEX IF NOT EXISTS idx_audit_security_timestamp ON audit_security(event_timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_security_user ON audit_security(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_security_session ON audit_security(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_security_ip ON audit_security(source_ip) WHERE source_ip IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_security_success ON audit_security(success, event_timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_security_failures ON audit_security(success, severity) WHERE success = FALSE;

-- Backup/restore audit indexes
CREATE INDEX IF NOT EXISTS idx_audit_backup_type ON audit_backup_restore(operation_type);
CREATE INDEX IF NOT EXISTS idx_audit_backup_started_at ON audit_backup_restore(started_at);
CREATE INDEX IF NOT EXISTS idx_audit_backup_status ON audit_backup_restore(status);
CREATE INDEX IF NOT EXISTS idx_audit_backup_performed_by ON audit_backup_restore(performed_by) WHERE performed_by IS NOT NULL;

-- Configuration audit indexes
CREATE INDEX IF NOT EXISTS idx_audit_config_key ON audit_configuration(configuration_key);
CREATE INDEX IF NOT EXISTS idx_audit_config_changed_at ON audit_configuration(changed_at);
CREATE INDEX IF NOT EXISTS idx_audit_config_changed_by ON audit_configuration(changed_by) WHERE changed_by IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_config_environment ON audit_configuration(environment);
CREATE INDEX IF NOT EXISTS idx_audit_config_approval ON audit_configuration(approval_required, approved_at);

-- Audit retention and cleanup views
CREATE VIEW IF NOT EXISTS audit_retention_policy AS
SELECT
    'audit_trail' as table_name,
    COUNT(*) as total_records,
    MIN(changed_at) as oldest_record,
    MAX(changed_at) as newest_record,
    COUNT(CASE WHEN changed_at < datetime('now', '-90 days') THEN 1 END) as records_to_archive,
    COUNT(CASE WHEN changed_at < datetime('now', '-1 year') THEN 1 END) as records_to_delete
FROM audit_trail

UNION ALL

SELECT
    'audit_sessions' as table_name,
    COUNT(*) as total_records,
    MIN(session_start) as oldest_record,
    MAX(session_start) as newest_record,
    COUNT(CASE WHEN session_start < datetime('now', '-90 days') THEN 1 END) as records_to_archive,
    COUNT(CASE WHEN session_start < datetime('now', '-1 year') THEN 1 END) as records_to_delete
FROM audit_sessions;

-- Recent audit activity view
CREATE VIEW IF NOT EXISTS recent_audit_activity AS
SELECT
    'audit_trail' as source,
    table_name as entity,
    operation as action,
    changed_by as performed_by,
    changed_at as timestamp,
    reason
FROM audit_trail
WHERE changed_at >= datetime('now', '-24 hours')

UNION ALL

SELECT
    'financial_actions' as source,
    entity_type as entity,
    action_type as action,
    performed_by,
    performed_at as timestamp,
    notes as reason
FROM audit_financial_actions
WHERE performed_at >= datetime('now', '-24 hours')

UNION ALL

SELECT
    'security' as source,
    event_type as entity,
    description as action,
    user_id as performed_by,
    event_timestamp as timestamp,
    failure_reason as reason
FROM audit_security
WHERE event_timestamp >= datetime('now', '-24 hours')

ORDER BY timestamp DESC;