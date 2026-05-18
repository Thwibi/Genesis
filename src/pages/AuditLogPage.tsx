import { useEffect, useState } from "react";
import { api, AuditEventDto } from "../services/api";

const EVENT_TYPES = [
  "",
  "LOGIN_SUCCESS",
  "LOGIN_FAILURE",
  "OTP_GENERATED",
  "OTP_COPIED",
  "SECRET_ACCESSED",
  "SESSION_REVOKED",
  "PERMISSION_CHANGED",
  "SHIFT_ASSIGNMENT_CHANGED",
  "SUSPICIOUS_ACTIVITY_DETECTED",
  "FAILED_ACCESS_ATTEMPT",
  "CLIPBOARD_CLEARED",
  "CLIPBOARD_TIMEOUT",
];

const SEVERITIES = ["", "INFO", "WARN", "HIGH", "CRITICAL"];

export default function AuditLogPage() {
  const [events, setEvents] = useState<AuditEventDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterType, setFilterType] = useState("");
  const [filterSeverity, setFilterSeverity] = useState("");
  const [page, setPage] = useState(0);
  const pageSize = 50;

  const fetchEvents = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.queryAuditLogs({
        event_type: filterType || undefined,
        severity: filterSeverity || undefined,
        limit: pageSize,
        offset: page * pageSize,
      });
      setEvents(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch audit logs");
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchEvents();
  }, [filterType, filterSeverity, page]);

  const getSeverityClass = (severity: string) => {
    switch (severity) {
      case "CRITICAL": return "chip-error";
      case "HIGH": return "chip-warning";
      case "WARN": return "chip-warning";
      default: return "chip-success";
    }
  };

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <h2>Audit Log</h2>
        <button className="btn btn-secondary btn-md" onClick={fetchEvents}>
          Refresh
        </button>
      </div>

      {/* Filters */}
      <div className="card">
        <div className="card-body">
          <div className="flex gap-4">
            <div className="input-group">
              <label className="input-label">Event Type</label>
              <select
                className="input-field"
                value={filterType}
                onChange={(e) => { setFilterType(e.target.value); setPage(0); }}
                style={{ minWidth: 200 }}
              >
                {EVENT_TYPES.map((t) => (
                  <option key={t} value={t}>{t || "All Events"}</option>
                ))}
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">Severity</label>
              <select
                className="input-field"
                value={filterSeverity}
                onChange={(e) => { setFilterSeverity(e.target.value); setPage(0); }}
                style={{ minWidth: 120 }}
              >
                {SEVERITIES.map((s) => (
                  <option key={s} value={s}>{s || "All Severities"}</option>
                ))}
              </select>
            </div>
          </div>
        </div>
      </div>

      {error && (
        <div className="alert alert-error">
          <span>{error}</span>
        </div>
      )}

      {/* Audit Events Table */}
      {loading ? (
        <p className="text-secondary">Loading audit events...</p>
      ) : events.length === 0 ? (
        <div className="card">
          <div className="card-body text-center">
            <p className="text-secondary">No audit events found.</p>
          </div>
        </div>
      ) : (
        <table className="data-table card">
          <thead>
            <tr>
              <th>Timestamp</th>
              <th>Event Type</th>
              <th>Severity</th>
              <th>User</th>
              <th>Workstation</th>
              <th>Details</th>
            </tr>
          </thead>
          <tbody>
            {events.map((event) => (
              <tr key={event.id}>
                <td style={{ fontSize: "var(--text-small)", whiteSpace: "nowrap" }}>
                  {new Date(event.timestamp).toLocaleString()}
                </td>
                <td>
                  <span className="chip">{event.event_type}</span>
                </td>
                <td>
                  <span className={`chip ${getSeverityClass(event.severity)}`}>
                    {event.severity}
                  </span>
                </td>
                <td style={{ fontSize: "var(--text-small)" }}>
                  {event.user_id?.slice(0, 8) || "—"}
                </td>
                <td style={{ fontSize: "var(--text-small)" }}>
                  {event.workstation_id || "—"}
                </td>
                <td style={{ fontSize: "var(--text-small)", maxWidth: 300, overflow: "hidden", textOverflow: "ellipsis" }}>
                  {JSON.stringify(event.metadata)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {/* Pagination */}
      <div className="flex items-center justify-between">
        <span className="text-secondary">
          Page {page + 1}
        </span>
        <div className="flex gap-2">
          <button
            className="btn btn-secondary btn-sm"
            onClick={() => setPage(Math.max(0, page - 1))}
            disabled={page === 0}
          >
            Previous
          </button>
          <button
            className="btn btn-secondary btn-sm"
            onClick={() => setPage(page + 1)}
            disabled={events.length < pageSize}
          >
            Next
          </button>
        </div>
      </div>
    </div>
  );
}
