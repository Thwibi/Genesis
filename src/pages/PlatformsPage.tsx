import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "../stores/authStore";
import { usePlatformStore } from "../stores/platformStore";
import { api, PlatformDto } from "../services/api";

export default function PlatformsPage() {
  const { session } = useAuthStore();
  const { platforms, loading, fetchPlatforms } = usePlatformStore();
  const navigate = useNavigate();

  const [showCreate, setShowCreate] = useState(false);
  const [storeSecretFor, setStoreSecretFor] = useState<string | null>(null);
  const [seedB64, setSeedB64] = useState("");
  const [secretStatus, setSecretStatus] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    name: "",
    platform_type: "",
    environment: "production",
    criticality_level: "MEDIUM",
    assigned_teams: "",
  });

  useEffect(() => {
    fetchPlatforms();
  }, [fetchPlatforms]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!session) return;

    try {
      await api.createPlatform({
        name: formData.name,
        platform_type: formData.platform_type,
        environment: formData.environment,
        criticality_level: formData.criticality_level,
        assigned_teams: formData.assigned_teams.split(",").map((t) => t.trim()).filter(Boolean),
        session_id: session.id,
      });
      setShowCreate(false);
      setFormData({ name: "", platform_type: "", environment: "production", criticality_level: "MEDIUM", assigned_teams: "" });
      fetchPlatforms();
    } catch (err) {
      console.error("Failed to create platform:", err);
    }
  };

  const handleStoreSecret = async (platformId: string) => {
    if (!session) return;
    setSecretStatus(null);
    try {
      await api.storeSecret({ platform_id: platformId, raw_seed_b64: seedB64, session_id: session.id });
      setSecretStatus("Secret stored successfully");
      setStoreSecretFor(null);
      setSeedB64("");
    } catch (err) {
      setSecretStatus(err instanceof Error ? err.message : "Failed to store secret");
    }
  };

  const generateRandomSeed = () => {
    const bytes = new Uint8Array(20);
    window.crypto.getRandomValues(bytes);
    const b64 = btoa(String.fromCharCode(...bytes));
    setSeedB64(b64);
  };

  const handleDelete = async (platformId: string) => {
    if (!session) return;
    if (!window.confirm("Delete this platform? This action cannot be undone.")) return;

    try {
      await api.deletePlatform(platformId, session.id);
      fetchPlatforms();
    } catch (err) {
      console.error("Failed to delete platform:", err);
    }
  };

  const getCriticalityClass = (level: string) => {
    switch (level) {
      case "CRITICAL": return "chip-error";
      case "HIGH": return "chip-warning";
      default: return "chip-success";
    }
  };

  return (
    <div className="flex flex-col gap-6">
      <div className="flex items-center justify-between">
        <h2>Protected Platforms</h2>
        <button className="btn btn-primary btn-md" onClick={() => setShowCreate(!showCreate)}>
          {showCreate ? "Cancel" : "Add Platform"}
        </button>
      </div>

      {/* Create Form */}
      {showCreate && (
        <div className="card">
          <div className="card-body">
            <h3 style={{ marginBottom: "var(--space-4)" }}>Register New Platform</h3>
            <form onSubmit={handleCreate} className="flex flex-col gap-4">
              <div className="grid-2">
                <div className="input-group">
                  <label className="input-label">Platform Name</label>
                  <input
                    className="input-field"
                    value={formData.name}
                    onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                    required
                    placeholder="e.g. Microsoft Defender XDR"
                  />
                </div>
                <div className="input-group">
                  <label className="input-label">Platform Type</label>
                  <input
                    className="input-field"
                    value={formData.platform_type}
                    onChange={(e) => setFormData({ ...formData, platform_type: e.target.value })}
                    required
                    placeholder="e.g. SIEM, EDR, VPN"
                  />
                </div>
                <div className="input-group">
                  <label className="input-label">Environment</label>
                  <select
                    className="input-field"
                    value={formData.environment}
                    onChange={(e) => setFormData({ ...formData, environment: e.target.value })}
                  >
                    <option value="production">Production</option>
                    <option value="staging">Staging</option>
                    <option value="development">Development</option>
                  </select>
                </div>
                <div className="input-group">
                  <label className="input-label">Criticality</label>
                  <select
                    className="input-field"
                    value={formData.criticality_level}
                    onChange={(e) => setFormData({ ...formData, criticality_level: e.target.value })}
                  >
                    <option value="LOW">Low</option>
                    <option value="MEDIUM">Medium</option>
                    <option value="HIGH">High</option>
                    <option value="CRITICAL">Critical</option>
                  </select>
                </div>
              </div>
              <div className="input-group">
                <label className="input-label">Assigned Teams (comma-separated)</label>
                <input
                  className="input-field"
                  value={formData.assigned_teams}
                  onChange={(e) => setFormData({ ...formData, assigned_teams: e.target.value })}
                  placeholder="SOC Tier 1, SOC Tier 2"
                />
              </div>
              <button type="submit" className="btn btn-primary btn-md" style={{ alignSelf: "flex-start" }}>
                Register Platform
              </button>
            </form>
          </div>
        </div>
      )}

      {/* Platforms List */}
      {loading ? (
        <p className="text-secondary">Loading platforms...</p>
      ) : platforms.length === 0 ? (
        <div className="card">
          <div className="card-body text-center">
            <p className="text-secondary">No platforms registered yet.</p>
          </div>
        </div>
      ) : (
        <table className="data-table card">
          <thead>
            <tr>
              <th>Name</th>
              <th>Type</th>
              <th>Environment</th>
              <th>Criticality</th>
              <th>Teams</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {platforms.map((platform) => (
              <tr key={platform.id}>
                <td style={{ fontWeight: 500 }}>{platform.name}</td>
                <td><span className="chip">{platform.platform_type}</span></td>
                <td>{platform.environment}</td>
                <td>
                  <span className={`chip ${getCriticalityClass(platform.criticality_level)}`}>
                    {platform.criticality_level}
                  </span>
                </td>
                <td>{platform.assigned_teams.join(", ")}</td>
                <td>
                  <div className="flex gap-2" style={{ flexWrap: "wrap" }}>
                    <button
                      className="btn btn-primary btn-sm"
                      onClick={() => navigate(`/platforms/${platform.id}/otp`)}
                    >
                      OTP
                    </button>
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => { setStoreSecretFor(platform.id); setSeedB64(""); setSecretStatus(null); }}
                    >
                      Store Secret
                    </button>
                    <button
                      className="btn btn-destructive btn-sm"
                      onClick={() => handleDelete(platform.id)}
                    >
                      Delete
                    </button>
                  </div>
                  {storeSecretFor === platform.id && (
                    <div className="card" style={{ marginTop: "var(--space-3)", padding: "var(--space-4)" }}>
                      <div className="flex flex-col gap-3">
                        <div className="input-group">
                          <label className="input-label">TOTP Seed (base64)</label>
                          <input
                            className="input-field"
                            value={seedB64}
                            onChange={(e) => setSeedB64(e.target.value)}
                            placeholder="e.g. GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
                          />
                        </div>
                        <div className="flex gap-2">
                          <button className="btn btn-secondary btn-sm" onClick={generateRandomSeed}>
                            Generate Random
                          </button>
                          <button className="btn btn-primary btn-sm" onClick={() => handleStoreSecret(platform.id)}>
                            Save Secret
                          </button>
                          <button className="btn btn-ghost btn-sm" onClick={() => { setStoreSecretFor(null); setSecretStatus(null); }}>
                            Cancel
                          </button>
                        </div>
                        {secretStatus && (
                          <div className={`alert ${secretStatus === "Secret stored successfully" ? "alert-success" : "alert-error"}`}>
                            <span>{secretStatus}</span>
                          </div>
                        )}
                      </div>
                    </div>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
