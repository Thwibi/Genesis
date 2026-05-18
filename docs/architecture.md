# Genesis SOC Auth — Architecture

## 1. Layer Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                        PRESENTATION LAYER                            │
│  React (TypeScript) · Tauri WebView · DESIGN.md Visual Language     │
│  Pages: Login, Dashboard, Platforms, OTP View, Audit, Shifts,       │
│          Monitoring                                                  │
├──────────────────────────────────────────────────────────────────────┤
│                        APPLICATION LAYER                             │
│  Tauri Command Handlers (thin wrappers)                              │
│  ┌───────────┬──────────┬──────────┬──────────┬──────────┬─────────┐│
│  │  auth.rs  │platforms │  otp.rs  │secrets.rs│ audit.rs │shifts.rs││
│  │           │  .rs     │          │          │          │         ││
│  └───────────┴──────────┴──────────┴──────────┴──────────┴─────────┘│
├──────────────────────────────────────────────────────────────────────┤
│                        DOMAIN LAYER                                  │
│  Services: AuthService, OtpService, PlatformService,                 │
│            SecretService, ShiftService, MonitoringService            │
│  Entities: User, Session, ProtectedPlatform, MfaSecret,             │
│            AuditEvent, Shift                                         │
├──────────────────────────────────────────────────────────────────────┤
│                        SECURITY LAYER                                │
│  Vault (AES-256-GCM) · Password (Argon2id) · Crypto (HKDF-SHA256)  │
│  Clipboard Manager · SecretBytes (zeroize)                          │
├──────────────────────────────────────────────────────────────────────┤
│                        PERSISTENCE LAYER                             │
│  SQLite + SQLCipher · Migrations · Repositories                     │
│  Audit: Append-only triggers · SHA-256 hash chain                    │
└──────────────────────────────────────────────────────────────────────┘
```

## 2. Core OTP Copy Workflow (End to End)

```
Analyst
  │
  ├─ 1. Click "Copy OTP" button
  │
  ▼
Frontend (OtpViewPage.tsx)
  │
  ├─ 2. api.copyOtp(sessionId, platformId, otp)
  │     → Tauri invoke "cmd_copy_otp"
  │
  ▼
Command Handler (commands/otp.rs)
  │
  ├─ 3. Calls otp_service.copy_otp_to_clipboard()
  │
  ▼
Domain Service (otp_service.rs)
  │
  ├─ 4. Validates session (active, not revoked)
  │
  ├─ 5. Writes OTP_COPIED AuditEvent via AuditLogger.write()
  │     → THIS MUST SUCCEED BEFORE proceeding
  │     → Integrity hash computed via HashChain
  │     → Written to append-only audit_events table
  │
  ├─ 6. Returns Ok() to command handler
  │
  ▼
Frontend
  │
  ├─ 7. navigator.clipboard.writeText(otp)
  │
  ├─ 8. Starts clipboard countdown timer (visible to analyst)
  │
  ▼
Auto-Clear (after 30s)
  │
  ├─ 9. Clipboard cleared
  │
  ├─ 10. CLIPBOARD_TIMEOUT AuditEvent written
  │
  ▼
Done
```

**Security Properties:**
- Step 5 (audit write) occurs BEFORE step 7 (clipboard write)
- If audit write fails, the copy MUST NOT proceed
- The secret (TOTP seed) is decrypted in memory using SecretBytes and zeroized after use
- The clipboard is auto-cleared after 30s default

## 3. Encryption Key Hierarchy

```
Master Key (256-bit, generated at vault initialization)
    │
    ├─ HKDF-SHA256(info="auth-key") ──→ Authentication Key (Argon2id verification)
    │
    ├─ HKDF-SHA256(info="vault-key") ──→ AES-256-GCM Key (secret encryption)
    │
    ├─ HKDF-SHA256(info="db-key") ────→ SQLCipher Key (database encryption)
    │
    └─ HKDF-SHA256(info="integrity-key") → HMAC Key (audit chain verification)
```

**Key Management Rules:**
- Master key is stored on disk at the configured `master_key_path`
- All purpose-specific keys are derived using HKDF-SHA256 with unique `info` parameters
- Keys are never reused across purposes
- Sensitive key material in memory is wrapped in `SecretBytes` for zeroize-on-drop

## 4. Audit Integrity Chain

```
┌──────────────────────────────────────────────────────────┐
│                     audit_events table                    │
├────────────┬──────────────┬──────────────┬───────────────┤
│  event_1   │  integrity   │  SHA256(     │               │
│  fields    │  _hash =     │  fields_1 || │               │
│            │              │  "" )        │               │
├────────────┼──────────────┼──────────────┼───────────────┤
│  event_2   │  integrity   │  SHA256(     │               │
│  fields    │  _hash =     │  fields_2 || │               │
│            │              │  hash_1 )    │               │
├────────────┼──────────────┼──────────────┼───────────────┤
│  event_3   │  integrity   │  SHA256(     │               │
│  fields    │  _hash =     │  fields_3 || │               │
│            │              │  hash_2 )    │               │
├────────────┼──────────────┼──────────────┼───────────────┤
│    ...     │     ...      │     ...      │               │
└────────────┴──────────────┴──────────────┴───────────────┘
```

**Properties:**
- Each row's hash covers all event fields plus the previous row's hash
- Any modification to a previous row breaks all subsequent hashes
- Database-level triggers enforce INSERT-only (UPDATE and DELETE are rejected)
- The chain can be verified by recomputing all hashes from the first event

## 5. Session Lifecycle

```
Login ──→ Session Created ──→ Active ──→ Expired (inactivity timeout)
  │                                            │
  │                                            │
  └──→ Revoked (manual/logout) ────────────────┘
```

**Session States:**
- **Created**: Session record written to DB and in-memory cache
- **Active**: Session is valid; last_activity_timestamp updated on each command
- **Expired**: No activity for 8 hours; session removed from cache
- **Revoked**: Explicit logout or admin revocation; `revoked_at` set

**Session Metadata Captured:**
- `workstationId`: Device fingerprint
- `hostname`: Machine hostname from OS
- `operatingSystem`: OS version string
- `loginTimestamp`: Session start time
- `lastActivityTimestamp`: Updated on each API call
- `integrityState`: INTACT / COMPROMISED / SUSPENDED

## 6. Security Boundaries

```
┌─────────────────────────────────────────────────────────────────────┐
│  BOUNDARY 1: Presentation ↔ Application                             │
│  ────────────────────────────────────────────────────────────────   │
│  Crosses: Tauri IPC (serialized JSON)                               │
│  Rules:  • No secrets cross this boundary                           │
│          • Only 6-digit OTP strings (never raw seeds)               │
│          • All inputs validated at application layer               │
│          • Frontend sees only what it needs to display              │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│  BOUNDARY 2: Application ↔ Domain                                   │
│  ────────────────────────────────────────────────────────────────   │
│  Crosses: Function calls within Rust process                        │
│  Rules:  • Command handlers extract params, call domain services    │
│          • All authorization checks happen in domain layer          │
│          • Session validation happens before any operation          │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│  BOUNDARY 3: Domain ↔ Security                                      │
│  ────────────────────────────────────────────────────────────────   │
│  Crosses: Function calls with explicit key material                 │
│  Rules:  • Security layer has no business logic                     │
│          • Domain services never handle raw encryption              │
│          • Security layer enforces key isolation                    │
│          • SecretBytes ensures zeroization after use                │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│  BOUNDARY 4: Domain ↔ Persistence                                   │
│  ────────────────────────────────────────────────────────────────   │
│  Crosses: Repository trait calls                                    │
│  Rules:  • SQLCipher encrypts data at rest                          │
│          • Audit events are INSERT-only (triggers enforce)          │
│          • Repositories return domain entities, never raw rows      │
│          • DB connection errors propagate as DomainErrors           │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Module Inventory

| Module | Requirements Section | Status |
|--------|---------------------|--------|
| 1. User Auth & Session Management | Module 1 | Implemented |
| 2. Protected Platform Registry | Module 2 | Implemented |
| 3. Shared MFA Secret Management | Module 3 | Implemented |
| 4. OTP Generation & Access | Module 4 | Implemented |
| 5. Clipboard Monitoring & Protection | Module 5 | Implemented |
| 6. Analyst Accountability & Monitoring | Module 6 | Implemented |
| 7. Shift Operations Management | Module 7 | Implemented |
| 8. Audit Logging & Security Events | Module 8 | Implemented |
| 9. Operational Security Monitoring | Module 9 | Implemented |
| 10. Vault & Encryption Management | Module 10 | Implemented |

## Screens (from DESIGN.md visual language applied to SOC Auth)

| Screen | Route | Description |
|--------|-------|-------------|
| Login | `/login` | Username/password authentication |
| Dashboard | `/` | Stats, quick actions, active shifts, alerts |
| Platforms | `/platforms` | CRUD for protected platform registry |
| OTP View | `/platforms/:id/otp` | OTP generation, copy, clipboard countdown |
| Audit Log | `/audit` | Filterable, paginated audit event viewer |
| Shifts | `/shifts` | Shift creation and management |
| Monitoring | `/monitoring` | Security alerts dashboard with acknowledge |
