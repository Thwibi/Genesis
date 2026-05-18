// Typed Tauri invoke wrapper — all backend calls go through this layer
import { invoke } from "@tauri-apps/api/core";

export interface SessionDto {
  id: string;
  user_id: string;
  workstation_id: string;
  hostname: string;
  operating_system: string;
  login_timestamp: string;
  last_activity_timestamp: string;
  integrity_state: string;
}

export interface PlatformDto {
  id: string;
  name: string;
  platform_type: string;
  environment: string;
  criticality_level: string;
  assigned_teams: string[];
  created_at: string;
  updated_at: string;
}

export interface OtpDto {
  otp: string;
  clipboard_timeout_secs: number;
}

export interface AuditEventDto {
  id: string;
  user_id: string | null;
  session_id: string | null;
  platform_id: string | null;
  event_type: string;
  severity: string;
  workstation_id: string | null;
  timestamp: string;
  integrity_hash: string;
  metadata: Record<string, unknown>;
}

export interface ShiftDto {
  id: string;
  shift_name: string;
  shift_start: string;
  shift_end: string;
  created_at: string;
}

export interface SecurityAlertDto {
  id: string;
  event_type: string;
  severity: string;
  description: string;
  detected_at: string;
  acknowledged: boolean;
  metadata: Record<string, unknown>;
}

export interface CommandError {
  code: string;
  message: string;
}

function isTauriAvailable(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function cmd<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriAvailable()) {
    throw new Error("[TAURI_UNAVAILABLE] App is running outside Tauri webview. Use the Tauri window to run this app.");
  }
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    const cmdErr = err as CommandError;
    if (cmdErr?.code) {
      throw new Error(`[${cmdErr.code}] ${cmdErr.message}`);
    }
    throw err instanceof Error ? err : new Error(String(err));
  }
}

// Auth
export const api = {
  login: (username: string, password: string) =>
    cmd<SessionDto>("cmd_login", { username, password }),

  logout: (sessionId: string) =>
    cmd<void>("cmd_logout", { sessionId }),

  revokeSession: (sessionId: string, revokedBy: string) =>
    cmd<void>("cmd_revoke_session", { sessionId, revokedBy }),

  validateSession: (sessionId: string) =>
    cmd<SessionDto>("cmd_validate_session", { sessionId }),

  // Platforms
  listPlatforms: () =>
    cmd<PlatformDto[]>("cmd_list_platforms"),

  createPlatform: (request: {
    name: string;
    platform_type: string;
    environment: string;
    criticality_level: string;
    assigned_teams: string[];
    session_id: string;
  }) => cmd<PlatformDto>("cmd_create_platform", { request }),

  updatePlatform: (platformId: string, updates: {
    name?: string;
    platform_type?: string;
    environment?: string;
    criticality_level?: string;
    assigned_teams?: string[];
    session_id: string;
  }) => cmd<PlatformDto>("cmd_update_platform", { platformId, ...updates }),

  deletePlatform: (platformId: string, sessionId: string) =>
    cmd<void>("cmd_delete_platform", { platformId, sessionId }),

  // OTP
  generateOtp: (request: { session_id: string; platform_id: string }) =>
    cmd<OtpDto>("cmd_generate_otp", { request }),

  copyOtp: (sessionId: string, platformId: string, otp: string) =>
    cmd<void>("cmd_copy_otp", { sessionId, platformId, otp }),

  // Secrets
  storeSecret: (request: { platform_id: string; raw_seed_b64: string; session_id: string }) =>
    cmd<string>("cmd_store_secret", { request }),

  rotateSecret: (request: { platform_id: string; new_seed_b64: string; session_id: string }) =>
    cmd<void>("cmd_rotate_secret", { request }),

  archiveSecret: (request: { platform_id: string; session_id: string }) =>
    cmd<void>("cmd_archive_secret", { request }),

  // Audit
  queryAuditLogs: (params: {
    event_type?: string;
    user_id?: string;
    severity?: string;
    limit?: number;
    offset?: number;
  }) => cmd<AuditEventDto[]>("cmd_query_audit_logs", { params }),

  // Shifts
  createShift: (request: { shift_name: string; shift_start: string; shift_end: string; created_by: string }) =>
    cmd<ShiftDto>("cmd_create_shift", { request }),

  assignAnalyst: (request: { shift_id: string; analyst_id: string; assigned_by: string }) =>
    cmd<void>("cmd_assign_analyst", { request }),

  listShifts: () => cmd<ShiftDto[]>("cmd_list_shifts"),

  getActiveShifts: () => cmd<ShiftDto[]>("cmd_get_active_shifts"),

  // Monitoring
  getSecurityAlerts: () => cmd<SecurityAlertDto[]>("cmd_get_security_alerts"),

  acknowledgeAlert: (alertId: string) =>
    cmd<void>("cmd_acknowledge_alert", { alertId }),

  // Setup (first-run)
  checkUsers: () =>
    cmd<{ is_first_run: boolean }>("cmd_check_users"),

  createFirstUser: (username: string, email: string, password: string) =>
    cmd<{ user_id: string; username: string }>("cmd_create_first_user", { username, email, password }),
};
