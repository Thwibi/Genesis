import { useEffect, useState } from "react";
import { api, SecurityAlertDto } from "../services/api";

export default function MonitoringPage() {
  const [alerts, setAlerts] = useState<SecurityAlertDto[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchAlerts = async () => {
    setLoading(true);
    try {
      const result = await api.getSecurityAlerts();
      setAlerts(result);
    } catch (err) {
      console.error("Failed to fetch alerts:", err);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchAlerts();
  }, []);

  const handleAcknowledge = async (alertId: string) => {
    try {
      await api.acknowledgeAlert(alertId);
      fetchAlerts();
    } catch (err) {
      console.error("Failed to acknowledge alert:", err);
    }
  };

  const getSeverityClass = (severity: string) => {
    switch (severity) {
      case "CRITICAL": return "chip-error";
      case "HIGH": return "chip-warning";
      case "MEDIUM": return "chip-warning";
      default: return "chip-success";
    }
  };

  const unacknowledgedAlerts = alerts.filter((a) => !a.acknowledged);
  const acknowledgedAlerts = alerts.filter((a) => a.acknowledged);

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <h2>Security Monitoring</h2>
        <button className="btn btn-secondary btn-md" onClick={fetchAlerts}>
          Refresh
        </button>
      </div>

      {/* Stats */}
      <div className="grid-3">
        <div className="card">
          <div className="card-body">
            <div className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
              Total Alerts
            </div>
            <div style={{ fontSize: "var(--text-section)", fontWeight: 700, marginTop: "var(--space-2)" }}>
              {alerts.length}
            </div>
          </div>
        </div>
        <div className="card">
          <div className="card-body">
            <div className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
              Unacknowledged
            </div>
            <div style={{ fontSize: "var(--text-section)", fontWeight: 700, marginTop: "var(--space-2)", color: unacknowledgedAlerts.length > 0 ? "var(--color-error)" : undefined }}>
              {unacknowledgedAlerts.length}
            </div>
          </div>
        </div>
        <div className="card">
          <div className="card-body">
            <div className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
              Acknowledged
            </div>
            <div style={{ fontSize: "var(--text-section)", fontWeight: 700, marginTop: "var(--space-2)" }}>
              {acknowledgedAlerts.length}
            </div>
          </div>
        </div>
      </div>

      {/* Unacknowledged Alerts */}
      {unacknowledgedAlerts.length > 0 && (
        <div className="card">
          <div className="card-header">
            <h3>Unacknowledged Alerts</h3>
          </div>
          <div className="card-body" style={{ padding: 0 }}>
            {unacknowledgedAlerts.map((alert) => (
              <div
                key={alert.id}
                className="flex items-center justify-between"
                style={{ padding: "var(--space-3) var(--space-6)", borderBottom: "1px solid var(--color-border)" }}
              >
                <div className="flex items-center gap-3" style={{ flex: 1 }}>
                  <span className={`status-dot ${alert.severity === "CRITICAL" || alert.severity === "HIGH" ? "error" : "warning"}`} />
                  <div>
                    <div style={{ fontWeight: 500 }}>{alert.description}</div>
                    <div className="text-secondary" style={{ fontSize: "var(--text-caption)" }}>
                      {new Date(alert.detected_at).toLocaleString()}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <span className={`chip ${getSeverityClass(alert.severity)}`}>
                    {alert.severity}
                  </span>
                  <button
                    className="btn btn-ghost btn-sm"
                    onClick={() => handleAcknowledge(alert.id)}
                  >
                    Acknowledge
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Acknowledged Alerts */}
      {acknowledgedAlerts.length > 0 && (
        <div className="card">
          <div className="card-header">
            <h3>Acknowledged Alerts</h3>
          </div>
          <div className="card-body" style={{ padding: 0 }}>
            {acknowledgedAlerts.map((alert) => (
              <div
                key={alert.id}
                className="flex items-center justify-between"
                style={{ padding: "var(--space-3) var(--space-6)", borderBottom: "1px solid var(--color-border)", opacity: 0.6 }}
              >
                <div className="flex items-center gap-3" style={{ flex: 1 }}>
                  <span className="status-dot neutral" />
                  <div>
                    <div style={{ fontWeight: 500 }}>{alert.description}</div>
                    <div className="text-secondary" style={{ fontSize: "var(--text-caption)" }}>
                      {new Date(alert.detected_at).toLocaleString()}
                    </div>
                  </div>
                </div>
                <span className={`chip ${getSeverityClass(alert.severity)}`}>
                  {alert.severity}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {!loading && alerts.length === 0 && (
        <div className="card">
          <div className="card-body text-center">
            <p className="text-secondary">No security alerts detected.</p>
          </div>
        </div>
      )}
    </div>
  );
}
