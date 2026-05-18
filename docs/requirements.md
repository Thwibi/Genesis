# Genesis SOC Auth — Requirements

> **Project Codename**: Genesis SOC Auth
> **Project Type**: Security-First Desktop MFA Operations Platform
> **Primary Goal**: Replace Android emulator–based MFA workflows in SOC environments with a secure, auditable, desktop-native solution.
> **Target Environment**: Security Operations Centers (SOC)
> **Target Platform**: Windows 10 / Windows 11

---

## Priority Order

1. Security
2. Accountability
3. Reliability
4. Operational Visibility
5. Performance
6. Ease of Use

---

## 1. Project Overview

Genesis SOC Auth is a desktop-native MFA operations platform designed for SOC teams that use shared security platforms requiring MFA authentication.

The platform replaces operational dependency on Android emulators running Microsoft Authenticator or similar mobile authenticators.

The system focuses on:

- Secure shared OTP management
- Analyst accountability
- Operational auditability
- Controlled MFA access
- Session attribution
- Clipboard monitoring
- Secure local storage

The platform is intended for environments where multiple SOC analysts access shared security platforms such as:

- SIEMs
- XDR platforms
- EDR consoles
- VPN portals
- PAM solutions
- Cloud security dashboards
- Firewall administration portals

---

## 2. Core Operational Problem

Current SOC workflows commonly rely on:

- Android emulators
- Shared phones
- Unmanaged OTP sharing
- Weak auditability
- No operator attribution

This creates:

- Security risks
- Operational ambiguity
- Accountability gaps
- Difficult investigations
- Weak MFA governance

Genesis SOC Auth must provide:

- Centralized MFA operations
- Secure OTP access
- Analyst attribution
- Immutable audit logging
- Workstation-level traceability

---

## 3. Core System Objectives

The platform must:

- Eliminate Android emulator dependency
- Centralize MFA operational workflows
- Securely store OTP secrets
- Track who accessed which platform
- Track who copied OTP codes
- Attribute actions to analysts and workstations
- Support SOC shift operations
- Provide immutable security auditing
- Minimize operational friction

---

## 4. User Roles

| Role | Description |
|---|---|
| SOC Analyst | Standard operational user |
| SOC Supervisor | Operational monitoring and oversight |
| Security Administrator | Security policy and platform administration |
| System Administrator | Infrastructure and deployment management |

---

## 5. Core Operational Workflow

```
1. Analyst logs into Genesis SOC Auth
2. Analyst selects protected platform
3. System validates access permissions
4. OTP code is generated/revealed
5. Analyst copies OTP
6. Clipboard monitoring activates
7. Audit event is created
8. Session attribution is recorded
9. Clipboard auto-clears after timeout
```

---

## 6. Functional Modules

### Module 1 — User Authentication & Session Management

**Requirements**

The platform must support:

- Secure user login
- Session creation
- Session expiration
- Session revocation
- Workstation attribution
- Concurrent session tracking
- Device trust validation

**Authentication Requirements**

The system must support:

- Username/password authentication
- Optional biometric unlock
- Role-based access control
- Secure password reset
- Progressive login delays
- Account lockout policies

**Session Metadata**

Each session must store:

- `sessionId`
- `userId`
- `workstationId`
- `hostname`
- `operatingSystem`
- `loginTimestamp`
- `lastActivityTimestamp`
- `integrityState`

---

### Module 2 — Protected Platform Registry

**Goals**

Manage all MFA-protected operational platforms.

**Requirements**

The platform must support:

- Platform registration
- Platform categorization
- Access group assignment
- MFA secret assignment
- Platform ownership metadata
- Environment tagging

**Example Platforms**

- Microsoft Defender XDR
- Trend Vision One
- QRadar
- Splunk
- Fortinet
- Palo Alto
- VPN gateways
- Cloud portals

**Platform Fields**

Each protected platform must contain:

- `platformId`
- `platformName`
- `platformType`
- `environment`
- `assignedTeams`
- `secretReference`
- `criticalityLevel`
- `createdAt`
- `updatedAt`

---

### Module 3 — Shared MFA Secret Management

**Goals**

Securely manage shared OTP secrets used by SOC teams.

**Requirements**

The platform must support:

- TOTP secret storage
- Secret rotation
- Controlled secret visibility
- Encrypted secret storage
- Secret lifecycle management
- Archive workflows

**Security Rules**

- Secrets must remain encrypted at rest
- Secrets must never appear in logs
- Secrets must never be exportable in plaintext
- Secret access must always be audited
- Secret access permissions must be role-based

---

### Module 4 — OTP Generation & Access

**Requirements**

The system must support:

- TOTP generation
- Secure OTP reveal
- One-click OTP copy
- Secure clipboard workflows
- OTP access auditing

**OTP Rules**

- OTP generation must follow RFC 6238
- OTPs must never persist in plaintext storage
- OTP access must require authenticated sessions
- OTP generation events must be audit logged

---

### Module 5 — Clipboard Monitoring & Protection

**Goals**

Track and secure all OTP copy operations.

**Requirements**

The system must:

- Detect OTP copy actions
- Log copy operations
- Support clipboard auto-clearing
- Support configurable expiration timers
- Minimize clipboard exposure duration

**Default Clipboard Policy**

| Setting | Value |
|---|---|
| Auto-clear enabled | Yes |
| Default timeout | 30 seconds |
| Clipboard auditing | Mandatory |

**Clipboard Security Events**

The following events must be logged:

- OTP copied
- Clipboard cleared
- Clipboard timeout expired
- Manual clipboard invalidation

---

### Module 6 — Analyst Accountability & Monitoring

**Goals**

Provide operational visibility into analyst MFA activity.

**Requirements**

The platform must track:

- Which analyst accessed which platform
- Which analyst copied which OTP
- Timestamp of actions
- Workstation attribution
- Session attribution
- Shift attribution

**Example Audit Events**

```
SA1 copied OTP for Microsoft Defender XDR at 2026-05-16 18:23:45
SA2 copied OTP for Trend Vision One at 2026-05-16 18:43:12
SA3 accessed Fortinet VPN MFA at 2026-05-16 19:01:07
```

---

### Module 7 — Shift Operations Management

**Goals**

Support SOC operational workflows across multiple shifts.

**Requirements**

The platform must support:

- Shift creation
- Analyst assignment
- Active shift tracking
- Shift-based attribution
- Concurrent analyst visibility

**Shift Types**

- Morning Shift
- Evening Shift
- Night Shift
- Incident Response Shift

---

### Module 8 — Audit Logging & Security Events

**Goals**

Maintain immutable operational and security audit trails.

**Requirements**

All sensitive operations must generate audit events.

**Mandatory Audit Events**

- Login success
- Login failure
- OTP generation
- OTP copy
- Secret access
- Session revocation
- Permission changes
- Shift assignment changes
- Suspicious activity detection
- Failed access attempts

**Audit Rules**

- Logs must be append-only
- Logs must support integrity verification
- Logs must be immutable after creation
- Administrative users must not silently modify logs
- Secrets must never appear in audit events

---

### Module 9 — Operational Security Monitoring

**Goals**

Detect suspicious MFA operational behavior.

**Detection Requirements**

The platform should detect:

- Excessive OTP copying
- Off-shift access
- Unauthorized workstation usage
- Simultaneous suspicious access
- Repeated failed authentication
- Abnormal platform access frequency

**Alert Severity Levels**

| Level | Description |
|---|---|
| Low | Informational anomaly |
| Medium | Suspicious activity |
| High | Potential compromise |
| Critical | Immediate investigation required |

---

### Module 10 — Vault & Encryption Management

**Goals**

Securely protect all MFA secrets and operational metadata.

**Encryption Requirements**

| Purpose | Requirement |
|---|---|
| Symmetric encryption | AES-256-GCM |
| Password hashing | Argon2id |
| Key derivation | HKDF / Argon2id |
| Integrity verification | SHA-256 or stronger |
| RNG | Cryptographically secure RNG |

**Forbidden**

- Plaintext secret storage
- MD5
- ECB mode
- Hardcoded keys
- Custom cryptographic algorithms

**Key Management Rules**

- Keys must be isolated by purpose
- Master keys must never be reused directly
- Sensitive memory should be zeroized where feasible
- Vault integrity verification must occur during startup

---

## 7. Core Domain Entities

### Entity — User

| Field | |
|---|---|
| `id` | Unique identifier |
| `username` | Login username |
| `email` | Email address |
| `passwordHash` | Hashed credential |
| `role` | Assigned role |
| `assignedShiftId` | Current shift reference |
| `accountStatus` | Active / locked / disabled |
| `createdAt` | Creation timestamp |
| `lastLoginAt` | Last login timestamp |

### Entity — Session

| Field | |
|---|---|
| `id` | Unique identifier |
| `userId` | Owner reference |
| `workstationId` | Device identifier |
| `hostname` | Machine hostname |
| `operatingSystem` | OS info |
| `loginTimestamp` | Session start |
| `lastActivityTimestamp` | Last recorded activity |
| `revokedAt` | Revocation timestamp |
| `integrityState` | Session integrity flag |

### Entity — ProtectedPlatform

| Field | |
|---|---|
| `id` | Unique identifier |
| `name` | Platform name |
| `platformType` | Type classification |
| `environment` | Env tag (prod/staging) |
| `criticalityLevel` | Criticality rating |
| `assignedTeams` | Team references |
| `createdAt` | Creation timestamp |
| `updatedAt` | Last update timestamp |

### Entity — MFASecret

| Field | |
|---|---|
| `id` | Unique identifier |
| `platformId` | Platform reference |
| `encryptedSecret` | Encrypted TOTP seed |
| `algorithm` | TOTP algorithm |
| `digits` | OTP digit count |
| `period` | TOTP period (seconds) |
| `createdAt` | Creation timestamp |
| `rotatedAt` | Last rotation timestamp |
| `archivedAt` | Archive timestamp |

### Entity — AuditEvent

| Field | |
|---|---|
| `id` | Unique identifier |
| `userId` | Actor reference |
| `sessionId` | Session reference |
| `platformId` | Target platform |
| `eventType` | Event classification |
| `severity` | Alert severity |
| `workstationId` | Device reference |
| `timestamp` | Event timestamp |
| `integrityHash` | Tamper-detection hash |
| `metadata` | Additional context |

### Entity — Shift

| Field | |
|---|---|
| `id` | Unique identifier |
| `shiftName` | Descriptive name |
| `shiftStart` | Start timestamp |
| `shiftEnd` | End timestamp |
| `createdAt` | Creation timestamp |

---

## 8. Security Requirements

### Authentication Security

The system must support:

- Brute-force protection
- Secure session handling
- Progressive login delays
- Account lockout
- Secure password hashing
- Suspicious login detection

### Memory Security

The platform should:

- Minimize plaintext exposure
- Avoid unsafe serialization
- Reduce secret lifetime in memory
- Isolate cryptographic operations

### Clipboard Security

The platform must:

- Minimize clipboard persistence
- Support auto-clearing
- Audit clipboard usage
- Warn users about clipboard risks

### Audit Integrity

Audit logs must:

- Remain append-only
- Support tamper detection
- Support chronological verification
- Remain immutable

### Dependency Security

The build pipeline must support:

- Dependency scanning
- SBOM generation
- Vulnerability monitoring
- Signed releases

---

## 9. Compliance & Security Standards

The architecture should align with:

- ISO/IEC 27001 principles
- ISO/IEC 27002 controls
- NIST CSF guidance
- OWASP ASVS Level 3
- CIS secure software guidance

---

## 10. Performance Requirements

| Requirement | Target |
|---|---|
| Application startup | < 2 seconds |
| OTP generation | Instant |
| Clipboard copy | < 100ms |
| Platform search | < 100ms |
| Audit event creation | < 200ms |

---

## 11. Reliability Requirements

The platform must:

- Survive unexpected shutdowns safely
- Preserve vault integrity
- Avoid audit corruption
- Support secure recovery workflows
- Maintain session consistency

---

## 12. Architecture Requirements

### Architectural Style

- Modular architecture
- Security-first design
- Service isolation
- Minimal trusted computing base
- Explicit dependency boundaries

### Required Layers

1. Presentation Layer
2. Application Layer
3. Domain Layer
4. Security Layer
5. Persistence Layer
6. Platform Integration Layer

### Architectural Rules

- UI must never directly access secrets
- Cryptographic services must be centralized
- Audit services must remain immutable
- Secret access must always pass through authorization checks

---

## 13. Windows Platform Requirements

The platform must support:

- Windows 10
- Windows 11

**Optional Integrations**

- Windows Hello
- TPM integration
- Secure desktop prompts
- Windows Credential APIs

---

## 14. Recommended Technical Stack

| Area | Recommendation |
|---|---|
| Backend Core | Rust |
| Desktop Framework | Tauri |
| Local Database | SQLite + SQLCipher |
| Async Runtime | Tokio |
| Cryptography | libsodium / ring |
| Logging | Structured JSON logging |

---

## 15. Threat Model

The system must consider:

- Malicious insiders
- Unauthorized OTP copying
- Session hijacking
- Clipboard malware
- DLL injection
- Memory scraping
- Credential theft
- Replay attacks
- Unauthorized workstation access
- Audit tampering
- Malicious dependency compromise

---

## 16. Acceptance Criteria

The platform is considered production-ready only if:

- Android emulator dependency is fully eliminated
- All OTP copy operations are audited
- Analyst attribution functions correctly
- Session tracking is operational
- Clipboard auto-clearing works correctly
- Secrets remain encrypted at rest
- Audit logs are immutable
- Unauthorized access attempts are detectable
- Security monitoring rules operate correctly
- All authentication workflows are tested
- Release builds are signed
- Dependency scanning is active
