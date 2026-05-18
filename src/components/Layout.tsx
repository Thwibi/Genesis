import { Outlet, NavLink, useNavigate } from "react-router-dom";
import { useAuthStore } from "../stores/authStore";

const navItems = [
  { to: "/", label: "Dashboard", icon: "◈" },
  { to: "/platforms", label: "Platforms", icon: "⊞" },
  { to: "/audit", label: "Audit Log", icon: "☰" },
  { to: "/shifts", label: "Shifts", icon: "◎" },
  { to: "/monitoring", label: "Monitoring", icon: "⚠" },
];

export default function Layout() {
  const { session, logout } = useAuthStore();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate("/login");
  };

  return (
    <div className="app-layout">
      <aside className="app-sidebar">
        <div className="sidebar-logo">Genesis</div>
        <nav className="sidebar-nav">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === "/"}
              className={({ isActive }) =>
                `sidebar-link${isActive ? " active" : ""}`
              }
            >
              <span>{item.icon}</span>
              <span>{item.label}</span>
            </NavLink>
          ))}
        </nav>
        <div style={{ flex: 1 }} />
        <div style={{ padding: "0 var(--space-3)" }}>
          {session && (
            <div className="flex flex-col gap-2" style={{ padding: "var(--space-3)" }}>
              <span className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
                {session.hostname}
              </span>
              <button className="btn btn-ghost btn-sm" onClick={handleLogout}>
                Sign Out
              </button>
            </div>
          )}
        </div>
      </aside>
      <main className="app-main">
        <header className="app-header">
          <div className="flex items-center gap-3">
            <span style={{ fontSize: "var(--text-body)", fontWeight: 500 }}>
              Genesis SOC Auth
            </span>
          </div>
          <div className="flex items-center gap-3">
            {session && (
              <>
                <span className="chip chip-active">{session.integrity_state}</span>
                <span className="text-secondary" style={{ fontSize: "var(--text-small)" }}>
                  {session.hostname}
                </span>
              </>
            )}
          </div>
        </header>
        <div className="app-content">
          <Outlet />
        </div>
      </main>
    </div>
  );
}
