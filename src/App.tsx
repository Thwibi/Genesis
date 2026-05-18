import { useEffect, useState } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { useAuthStore } from "./stores/authStore";
import { api } from "./services/api";
import Layout from "./components/Layout";
import LoginPage from "./pages/LoginPage";
import SetupPage from "./pages/SetupPage";
import DashboardPage from "./pages/DashboardPage";
import PlatformsPage from "./pages/PlatformsPage";
import OtpViewPage from "./pages/OtpViewPage";
import AuditLogPage from "./pages/AuditLogPage";
import ShiftsPage from "./pages/ShiftsPage";
import MonitoringPage from "./pages/MonitoringPage";

function ProtectedRoute({ children, firstRun }: { children: React.ReactNode; firstRun: boolean }) {
  const { session } = useAuthStore();
  if (firstRun) {
    return <Navigate to="/setup" replace />;
  }
  if (!session) {
    return <Navigate to="/login" replace />;
  }
  return <>{children}</>;
}

export default function App() {
  const { session } = useAuthStore();
  const [firstRun, setFirstRun] = useState<boolean | null>(null);

  useEffect(() => {
    api.checkUsers().then((status) => {
      setFirstRun(status.is_first_run);
    }).catch(() => {
      setFirstRun(false);
    });
  }, []);

  if (firstRun === null) {
    return (
      <div className="login-page">
        <div className="card login-card" style={{ textAlign: "center" }}>
          <div className="card-body">
            <p className="text-secondary">Initializing...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <Routes>
      <Route
        path="/setup"
        element={firstRun ? <SetupPage /> : <Navigate to="/login" replace />}
      />
      <Route
        path="/login"
        element={firstRun ? <Navigate to="/setup" replace /> : session ? <Navigate to="/" replace /> : <LoginPage />}
      />
      <Route
        path="/"
        element={
          <ProtectedRoute firstRun={firstRun}>
            <Layout />
          </ProtectedRoute>
        }
      >
        <Route index element={<DashboardPage />} />
        <Route path="platforms" element={<PlatformsPage />} />
        <Route path="platforms/:platformId/otp" element={<OtpViewPage />} />
        <Route path="audit" element={<AuditLogPage />} />
        <Route path="shifts" element={<ShiftsPage />} />
        <Route path="monitoring" element={<MonitoringPage />} />
      </Route>
    </Routes>
  );
}
