import { useEffect, useState } from "react";
import { useAuthStore } from "../stores/authStore";
import { api, ShiftDto } from "../services/api";

export default function ShiftsPage() {
  const { session } = useAuthStore();
  const [shifts, setShifts] = useState<ShiftDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  const [formData, setFormData] = useState({
    shift_name: "",
    shift_start: "",
    shift_end: "",
  });

  const fetchShifts = async () => {
    setLoading(true);
    try {
      const result = await api.listShifts();
      setShifts(result);
    } catch (err) {
      console.error("Failed to fetch shifts:", err);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchShifts();
  }, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!session) return;

    try {
      await api.createShift({
        shift_name: formData.shift_name,
        shift_start: new Date(formData.shift_start).toISOString(),
        shift_end: new Date(formData.shift_end).toISOString(),
        created_by: session.user_id,
      });
      setShowCreate(false);
      setFormData({ shift_name: "", shift_start: "", shift_end: "" });
      fetchShifts();
    } catch (err) {
      console.error("Failed to create shift:", err);
    }
  };

  const isActive = (shift: ShiftDto) => {
    const now = new Date();
    const start = new Date(shift.shift_start);
    const end = new Date(shift.shift_end);
    return now >= start && now <= end;
  };

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <h2>Shift Management</h2>
        <button className="btn btn-primary btn-md" onClick={() => setShowCreate(!showCreate)}>
          {showCreate ? "Cancel" : "Create Shift"}
        </button>
      </div>

      {showCreate && (
        <div className="card">
          <div className="card-body">
            <h3 style={{ marginBottom: "var(--space-4)" }}>New Shift</h3>
            <form onSubmit={handleCreate} className="flex flex-col gap-4">
              <div className="input-group">
                <label className="input-label">Shift Name</label>
                <select
                  className="input-field"
                  value={formData.shift_name}
                  onChange={(e) => setFormData({ ...formData, shift_name: e.target.value })}
                  required
                >
                  <option value="">Select shift type...</option>
                  <option value="Morning Shift">Morning Shift (06:00-14:00)</option>
                  <option value="Evening Shift">Evening Shift (14:00-22:00)</option>
                  <option value="Night Shift">Night Shift (22:00-06:00)</option>
                  <option value="Incident Response Shift">Incident Response Shift</option>
                </select>
              </div>
              <div className="grid-2">
                <div className="input-group">
                  <label className="input-label">Start Time</label>
                  <input
                    className="input-field"
                    type="datetime-local"
                    value={formData.shift_start}
                    onChange={(e) => setFormData({ ...formData, shift_start: e.target.value })}
                    required
                  />
                </div>
                <div className="input-group">
                  <label className="input-label">End Time</label>
                  <input
                    className="input-field"
                    type="datetime-local"
                    value={formData.shift_end}
                    onChange={(e) => setFormData({ ...formData, shift_end: e.target.value })}
                    required
                  />
                </div>
              </div>
              <button type="submit" className="btn btn-primary btn-md" style={{ alignSelf: "flex-start" }}>
                Create Shift
              </button>
            </form>
          </div>
        </div>
      )}

      {loading ? (
        <p className="text-secondary">Loading shifts...</p>
      ) : shifts.length === 0 ? (
        <div className="card">
          <div className="card-body text-center">
            <p className="text-secondary">No shifts defined yet.</p>
          </div>
        </div>
      ) : (
        <table className="data-table card">
          <thead>
            <tr>
              <th>Shift Name</th>
              <th>Start</th>
              <th>End</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {shifts.map((shift) => (
              <tr key={shift.id}>
                <td style={{ fontWeight: 500 }}>{shift.shift_name}</td>
                <td>{new Date(shift.shift_start).toLocaleString()}</td>
                <td>{new Date(shift.shift_end).toLocaleString()}</td>
                <td>
                  <span className={`chip ${isActive(shift) ? "chip-success" : ""}`}>
                    {isActive(shift) ? "Active" : "Inactive"}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
