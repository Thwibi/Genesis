import { useState, FormEvent } from "react";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "../stores/authStore";

export default function LoginPage() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const { login, loading, error, clearError } = useAuthStore();
  const navigate = useNavigate();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    clearError();
    try {
      await login(username, password);
      navigate("/");
    } catch {
      // Error is handled by the store
    }
  };

  return (
    <div className="login-page">
      <div className="card login-card">
        <div className="card-body">
          <h1 className="login-title">Genesis</h1>
          <p className="login-subtitle">SOC Auth Platform</p>

          <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            <div className="input-group">
              <label className="input-label" htmlFor="username">
                Username
              </label>
              <input
                id="username"
                className="input-field"
                type="text"
                placeholder="Enter your username"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                required
                autoFocus
              />
            </div>

            <div className="input-group">
              <label className="input-label" htmlFor="password">
                Password
              </label>
              <input
                id="password"
                className="input-field"
                type="password"
                placeholder="Enter your password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
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
            >
              {loading ? "Authenticating..." : "Sign In"}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
