import { useState, FormEvent, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "../services/api";

export default function SetupPage() {
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    api.checkUsers().then((status) => {
      if (!status.is_first_run) {
        navigate("/login", { replace: true });
      }
    }).catch(() => {
      // If the command doesn't exist yet, just show the form
    });
  }, [navigate]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);

    if (password !== confirmPassword) {
      setError("Passwords do not match.");
      return;
    }

    setLoading(true);
    try {
      await api.createFirstUser(username, email, password);
      setSuccess(true);
      setTimeout(() => navigate("/login"), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Setup failed");
    } finally {
      setLoading(false);
    }
  };

  if (success) {
    return (
      <div className="login-page">
        <div className="card login-card">
          <div className="card-body">
            <h1 className="login-title">Genesis</h1>
            <p className="login-subtitle">SOC Auth Platform</p>
            <div className="alert alert-success" style={{ marginTop: "var(--space-6)" }}>
              <span>Admin user created successfully! Redirecting to login...</span>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="login-page">
      <div className="card login-card">
        <div className="card-body">
          <h1 className="login-title">Genesis</h1>
          <p className="login-subtitle">First-Time Setup — Create Admin User</p>

          <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            <div className="input-group">
              <label className="input-label" htmlFor="setup-username">
                Username
              </label>
              <input
                id="setup-username"
                className="input-field"
                type="text"
                placeholder="e.g. admin"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                required
                autoFocus
              />
            </div>

            <div className="input-group">
              <label className="input-label" htmlFor="setup-email">
                Email
              </label>
              <input
                id="setup-email"
                className="input-field"
                type="email"
                placeholder="e.g. admin@soc.genesis"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
              />
            </div>

            <div className="input-group">
              <label className="input-label" htmlFor="setup-password">
                Password
              </label>
              <input
                id="setup-password"
                className="input-field"
                type="password"
                placeholder="At least 8 characters"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                minLength={8}
              />
            </div>

            <div className="input-group">
              <label className="input-label" htmlFor="setup-confirm-password">
                Confirm Password
              </label>
              <input
                id="setup-confirm-password"
                className="input-field"
                type="password"
                placeholder="Re-enter password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                required
                minLength={8}
              />
            </div>

            {error && (
              <div className="alert alert-error">
                <span>{error}</span>
              </div>
            )}

            <button
              type="submit"
              className="btn btn-primary btn-lg w-full"
              disabled={loading}
              style={{ marginTop: "var(--space-2)" }}
            >
              {loading ? "Creating Admin User..." : "Create Admin & Get Started"}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
