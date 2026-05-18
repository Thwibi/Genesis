-- V001: Initial Schema for Genesis SOC Auth
-- This schema defines all domain entities with encryption at rest,
-- append-only audit logging, and hash-chain integrity verification.

-- ============================================================
-- USERS
-- ============================================================
CREATE TABLE IF NOT EXISTS users (
    id              TEXT    NOT NULL PRIMARY KEY,
    username        TEXT    NOT NULL UNIQUE,
    email           TEXT    NOT NULL,
    password_hash   TEXT    NOT NULL,
    role            TEXT    NOT NULL DEFAULT 'SOC_ANALYST',
    assigned_shift_id TEXT,
    account_status  TEXT    NOT NULL DEFAULT 'ACTIVE',
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login_at   TIMESTAMP,

    FOREIGN KEY (assigned_shift_id) REFERENCES shifts(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_account_status ON users(account_status);

-- ============================================================
-- SESSIONS
-- ============================================================
CREATE TABLE IF NOT EXISTS sessions (
    id                      TEXT    NOT NULL PRIMARY KEY,
    user_id                 TEXT    NOT NULL,
    workstation_id          TEXT    NOT NULL,
    hostname                TEXT    NOT NULL,
    operating_system        TEXT    NOT NULL,
    login_timestamp         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity_timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_at              TIMESTAMP,
    integrity_state         TEXT    NOT NULL DEFAULT 'INTACT',

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_workstation_id ON sessions(workstation_id);
CREATE INDEX IF NOT EXISTS idx_sessions_revoked_at ON sessions(revoked_at);

-- ============================================================
-- PROTECTED PLATFORMS
-- ============================================================
CREATE TABLE IF NOT EXISTS protected_platforms (
    id              TEXT    NOT NULL PRIMARY KEY,
    name            TEXT    NOT NULL,
    platform_type   TEXT    NOT NULL,
    environment     TEXT    NOT NULL DEFAULT 'production',
    criticality_level TEXT  NOT NULL DEFAULT 'MEDIUM',
    assigned_teams  TEXT    NOT NULL DEFAULT '[]',
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_platforms_name ON protected_platforms(name);
CREATE INDEX IF NOT EXISTS idx_platforms_type ON protected_platforms(platform_type);
CREATE INDEX IF NOT EXISTS idx_platforms_criticality ON protected_platforms(criticality_level);

-- ============================================================
-- MFA SECRETS (encrypted at rest)
-- ============================================================
CREATE TABLE IF NOT EXISTS mfa_secrets (
    id              TEXT    NOT NULL PRIMARY KEY,
    platform_id     TEXT    NOT NULL,
    encrypted_secret BLOB  NOT NULL,
    algorithm       TEXT    NOT NULL DEFAULT 'SHA1',
    digits          INTEGER NOT NULL DEFAULT 6,
    period          INTEGER NOT NULL DEFAULT 30,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    rotated_at      TIMESTAMP,
    archived_at     TIMESTAMP,

    FOREIGN KEY (platform_id) REFERENCES protected_platforms(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_secrets_platform_id ON mfa_secrets(platform_id);
CREATE INDEX IF NOT EXISTS idx_secrets_archived_at ON mfa_secrets(archived_at);

-- ============================================================
-- AUDIT EVENTS (append-only - INSERT trigger enforces immutability)
-- ============================================================
CREATE TABLE IF NOT EXISTS audit_events (
    id              TEXT    NOT NULL PRIMARY KEY,
    user_id         TEXT,
    session_id      TEXT,
    platform_id     TEXT,
    event_type      TEXT    NOT NULL,
    severity        TEXT    NOT NULL DEFAULT 'INFO',
    workstation_id  TEXT,
    timestamp       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    integrity_hash  TEXT    NOT NULL,
    metadata        TEXT    NOT NULL DEFAULT '{}',

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE SET NULL,
    FOREIGN KEY (platform_id) REFERENCES protected_platforms(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_user_id ON audit_events(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_events(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_severity ON audit_events(severity);

-- ============================================================
-- AUDIT EVENTS: Append-Only Enforcement
-- SQL triggers prevent any UPDATE or DELETE on audit_events
-- ============================================================
CREATE TRIGGER IF NOT EXISTS trg_audit_events_prevent_update
BEFORE UPDATE ON audit_events
BEGIN
    SELECT RAISE(ABORT, 'Audit events are append-only: UPDATE is forbidden');
END;

CREATE TRIGGER IF NOT EXISTS trg_audit_events_prevent_delete
BEFORE DELETE ON audit_events
BEGIN
    SELECT RAISE(ABORT, 'Audit events are append-only: DELETE is forbidden');
END;

-- ============================================================
-- SHIFTS
-- ============================================================
CREATE TABLE IF NOT EXISTS shifts (
    id          TEXT    NOT NULL PRIMARY KEY,
    shift_name  TEXT    NOT NULL,
    shift_start TIMESTAMP NOT NULL,
    shift_end   TIMESTAMP NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_shifts_name ON shifts(shift_name);
CREATE INDEX IF NOT EXISTS idx_shifts_start ON shifts(shift_start);
CREATE INDEX IF NOT EXISTS idx_shifts_end ON shifts(shift_end);

-- ============================================================
-- SHIFT ASSIGNMENTS
-- ============================================================
CREATE TABLE IF NOT EXISTS shift_assignments (
    id          TEXT    NOT NULL PRIMARY KEY,
    shift_id    TEXT    NOT NULL,
    user_id     TEXT    NOT NULL,
    assigned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (shift_id) REFERENCES shifts(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(shift_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_shift_assignments_shift_id ON shift_assignments(shift_id);
CREATE INDEX IF NOT EXISTS idx_shift_assignments_user_id ON shift_assignments(user_id);

-- ============================================================
-- LOGIN ATTEMPT TRACKING (for progressive delay + lockout)
-- ============================================================
CREATE TABLE IF NOT EXISTS login_attempts (
    id          TEXT    NOT NULL PRIMARY KEY,
    username    TEXT    NOT NULL,
    attempted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success     INTEGER NOT NULL DEFAULT 0,
    workstation_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_login_attempts_username ON login_attempts(username);
CREATE INDEX IF NOT EXISTS idx_login_attempts_at ON login_attempts(attempted_at);
