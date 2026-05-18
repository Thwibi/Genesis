import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "../stores/authStore";
import { usePlatformStore } from "../stores/platformStore";
import { api, ShiftDto, SecurityAlertDto } from "../services/api";

export default function DashboardPage() {
  const { session } = useAuthStore();
  const { platforms } = usePlatformStore();
  const navigate = useNavigate();

  const [activeShifts, setActiveShifts] = useState<ShiftDto[]>([]);
  const [alerts, setAlerts] = useState<SecurityAlertDto[]>([]);

  useEffect(() => {
    api.getActiveShifts().then(setActiveShifts).catch(() => {});
    api.getSecurityAlerts().then(setAlerts).catch(() => {});
  }, []);

  const criticalAlerts = alerts.filter((a) => !a.acknowledged && (a.severity === "HIGH" || a.severity === "CRITICAL"));

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <h2>Dashboard</h2>
        <span className="text-secondary">
          {session?.hostname} — {session?.operating_system}
        </span>
      </div>

      {/* Stats Grid */}
      <div className="grid-4">
        <div className="card">
          <div className="card-body">
            <div className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
              Protected Platforms
            </div>
            <div style={{ fontSize: "var(--text-section)", fontWeight: 700, marginTop: "var(--space-2)" }}>
              {platforms.length}
            </div>
          </div>
        </div>
        <div className="card">
          <div className="card-body">
            <div className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
              Active Shifts
            </div>
            <div style={{ fontSize: "var(--text-section)", fontWeight: 700, marginTop: "var(--space-2)" }}>
              {activeShifts.length}
            </div>
          </div>
        </div>
        <div className="card">
          <div className="card-body">
            <div className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
              Security Alerts
            </div>
            <div style={{ fontSize: "var(--text-section)", fontWeight: 700, marginTop: "var(--space-2)", color: criticalAlerts.length > 0 ? "var(--color-error)" : undefined }}>
              {alerts.length}
            </div>
          </div>
        </div>
        <div className="card">
          <div className="card-body">
            <div className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
              Session Status
            </div>
            <div className="chip chip-success mt-2" style={{ display: "inline-flex" }}>
              Active
            </div>
          </div>
        </div>
      </div>

      {/* Critical Alerts */}
      {criticalAlerts.length > 0 && (
        <div className="card">
          <div className="card-header">
            <h3>Critical Security Alerts</h3>
          </div>
          <div className="card-body">
            {criticalAlerts.map((alert) => (
              <div key={alert.id} className="flex items-center justify-between" style={{ padding: "var(--space-2) 0" }}>
                <div className="flex items-center gap-3">
                  <span className="status-dot error" />
                  <span>{alert.description}</span>
                </div>
                <span className="chip chip-error">{alert.severity}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Quick Actions */}
      <div className="card">
        <div className="card-header">
          <h3>Quick Actions</h3>
        </div>
        <div className="card-body">
          <div className="flex gap-3">
            <button className="btn btn-primary btn-md" onClick={() => navigate("/platforms")}>
              Access Platform
            </button>
            <button className="btn btn-secondary btn-md" onClick={() => navigate("/audit")}>
              View Audit Log
            </button>
            <button className="btn btn-secondary btn-md" onClick={() => navigate("/monitoring")}>
              Security Alerts
            </button>
          </div>
        </div>
      </div>

      {/* Active Shifts */}
      <div className="card">
        <div className="card-header">
          <h3>Active Shifts</h3>
        </div>
        <div className="card-body">
          {activeShifts.length === 0 ? (
            <p className="text-secondary">No active shifts</p>
          ) : (
            <table className="data-table">
              <thead>
                <tr>
                  <th>Shift</th>
                  <th>Start</th>
                  <th>End</th>
                </tr>
              </thead>
              <tbody>
                {activeShifts.map((shift) => (
                  <tr key={shift.id}>
                    <td>{shift.shift_name}</td>
                    <td>{new Date(shift.shift_start).toLocaleTimeString()}</td>
                    <td>{new Date(shift.shift_end).toLocaleTimeString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
}
