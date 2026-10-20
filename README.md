# Genesis SOC Auth

Security-First Desktop MFA Operations Platform.

## Security Architecture

### Vault / Master Key

- A 32-byte master key is generated at first run and stored in the platform data directory as `master_key.bin`.
- An integrity checksum (`master_key.bin.sha256`) is written alongside the key and verified on every startup.
- The master key is used to derive purpose-specific sub-keys via HKDF-SHA256:
  - **Vault encryption key**: encrypts/decrypts TOTP seed material stored in the database.
  - **SQLCipher DB key**: encrypts the SQLite database at rest.
- Key derivation ensures that compromise of one derived key does not compromise the master key or other derived keys.

**Hardening recommendations (production):**
1. Replace file-based master key with a TPM-backed or HSM-backed key store.
2. Use Windows DPAPI or macOS Keychain to wrap the master key file.
3. Enable BitLocker (Windows) or FileVault (macOS) for full-disk encryption as a defense-in-depth layer.
4. Restrict file permissions on `master_key.bin` to the application user only.
5. Consider rotating the master key periodically using a key ceremony process.

### Database Encryption (SQLCipher)

The SQLite database is encrypted at rest using **SQLCipher** (AES-256-CBC with per-page HMAC). The DB encryption key is derived from the master key using HKDF with a distinct salt/info context string.

**Connection flow:**
1. Master key loaded from `master_key.bin`
2. DB key = `HKDF(master_key, salt="genesis-db-key-v1", info="sqlcipher-encryption")`
3. `PRAGMA key = "x'<hex_derived_key>'"` issued before any operations
4. WAL journal mode + foreign keys enabled

### Audit Integrity

The audit log uses a **SHA-256 hash chain**:
- Each event's `integrity_hash` covers all event fields + the previous event's hash.
- Tampering with any event breaks the chain and is detectable.
- Audit writes occur **before** clipboard operations (fail-closed on write failure).
- The `audit_events` table is append-only, enforced by SQL triggers that reject `UPDATE`/`DELETE`.

Platform lifecycle changes are audited (`PlatformCreated`, `PlatformUpdated`, `PlatformDeleted`),
attributed to the initiating session. As with clipboard operations, platform writes fail closed:
if the audit write fails, the operation returns `AuditWriteFailed`.

Constant-time secret comparison uses the `subtle` crate (`subtle::ConstantTimeEq`)
rather than `ring::constant_time`, whose verification helper is deprecated and
documents no side-channel guarantees.

### Clipboard Security

- OTP text is written to clipboard via the Tauri clipboard-manager plugin.
- An audit event (`OTP_COPIED`) is written to the append-only log **before** the clipboard write.
- If the audit write fails, the clipboard operation is aborted (fail-closed).
- Auto-clear is scheduled server-side: after 30 seconds, a `ClipboardTimeout` audit event is logged and the frontend should clear the clipboard via the plugin's `clear()` API.

## Build Requirements

### Prerequisites

- **Rust** (stable) — install via [rustup.rs](https://rustup.rs/)
- **Node.js** 20+ and npm
- **Tauri v2 prerequisites** per platform

### Windows Native Dependencies for SQLCipher

SQLCipher is built from source via rusqlite's `bundled-sqlcipher` feature (enabled in `Cargo.toml`). This requires:

1. **Visual Studio Build Tools** (or Visual Studio) with the "Desktop development with C++" workload.
   - Install from https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio
   - Ensure the following components are selected:
     - MSVC v143 - VS 2022 C++ x64/x86 build tools
     - Windows 10/11 SDK
     - C++ CMake tools for Windows

2. **CMake** (required by SQLCipher build system).
   - Install from https://cmake.org/download/ or via `winget install CMake`

3. **Tauri CLI**:
   ```bash
   cargo install tauri-cli --version "^2"
   ```

4. **npm dependencies**:
   ```bash
   npm install
   ```

### Building

```bash
# Build the Rust backend
cd src-tauri
cargo build

# Or build full Tauri desktop app
cd ..
npm run tauri build
```

### Development

```bash
# Start dev server with hot-reload
npm run tauri dev
```

## CI Configuration

The CI pipeline (`.github/workflows/ci.yml`) runs on `windows-latest`. It includes:

- **Clippy Lint**: Rust code linting
- **Unit Tests**: `cargo test --all`
- **Format Check**: `cargo fmt --check`
- **Security Audit**: `cargo audit`
- **Frontend**: TypeScript type check + lint

**Note:** SQLCipher requires MSVC build tools on the CI runner. The `windows-latest` GitHub Actions runner includes these by default. CMake is also pre-installed.

### Configuration

See `src-tauri/src/config/settings.rs` for configurable parameters:

| Parameter | Default | Description |
|---|---|---|
| `database_path` | `%APPDATA%/GenesisSOC/genesis_soc_auth.db` | SQLCipher-encrypted database path |
| `master_key_path` | `%APPDATA%/GenesisSOC/master_key.bin` | 32-byte vault master key path |
| `clipboard_timeout_secs` | `30` | Auto-clear timeout for clipboard OTP |
| `session_timeout_minutes` | `480` | Session expiry (8 hours) |
| `max_login_attempts` | `5` | Failed attempts before account lockout |
| `otp_period_seconds` | `30` | TOTP time step |
| `otp_digits` | `6` | TOTP output length |
