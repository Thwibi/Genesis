import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useAuthStore } from "../stores/authStore";
import { usePlatformStore } from "../stores/platformStore";

export default function OtpViewPage() {
  const { platformId } = useParams<{ platformId: string }>();
  const { session } = useAuthStore();
  const { platforms, selectedOtp, clipboardCountdown, clipboardActive, generateOtp, copyOtp, clearOtp } = usePlatformStore();
  const navigate = useNavigate();

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const platform = platforms.find((p) => p.id === platformId);

  useEffect(() => {
    if (!platformId || !session) return;

    const handleGenerate = async () => {
      setLoading(true);
      setError(null);
      try {
        await generateOtp(session.id, platformId);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to generate OTP");
      }
      setLoading(false);
    };

    handleGenerate();
  }, [platformId, session]);

  const handleCopy = async () => {
    if (!selectedOtp || !session || !platformId) return;

    try {
      setCopied(true);
      await copyOtp(session.id, platformId, selectedOtp.otp);

      // Write to clipboard
      try {
        await navigator.clipboard.writeText(selectedOtp.otp);
      } catch {
        // Fallback if clipboard API fails
      }

      // Start countdown timer (visible to analyst)
      usePlatformStore.getState().startClipboardTimer(selectedOtp.clipboard_timeout_secs);

      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to copy OTP");
    }
  };

  const progresses = clipboardActive
    ? ((selectedOtp?.clipboard_timeout_secs ?? 30) - clipboardCountdown) / (selectedOtp?.clipboard_timeout_secs ?? 30) * 100
    : 0;

  return (
    <div className="flex flex-col gap-6" style={{ maxWidth: 600, margin: "0 auto" }}>
      <div className="flex items-center justify-between">
        <div>
          <button
            className="btn btn-ghost btn-sm"
            onClick={() => navigate("/platforms")}
            style={{ marginBottom: "var(--space-2)" }}
          >
            ← Back to Platforms
          </button>
          <h2>{platform?.name ?? "Loading..."}</h2>
          {platform && (
            <p className="text-secondary">
              {platform.platform_type} — {platform.environment}
            </p>
          )}
        </div>
        {platform && (
          <span className={`chip ${platform.criticality_level === "CRITICAL" ? "chip-error" : "chip-success"}`}>
            {platform.criticality_level}
          </span>
        )}
      </div>

      {error && (
        <div className="alert alert-error">
          <span>{error}</span>
        </div>
      )}

      {/* OTP Display */}
      <div className="card">
        <div className="card-body" style={{ textAlign: "center" }}>
          {loading ? (
            <p className="text-secondary" style={{ padding: "var(--space-8)" }}>
              Generating OTP...
            </p>
          ) : selectedOtp ? (
            <>
              <div className="otp-display">{selectedOtp.otp}</div>

              <div className="flex items-center justify-center gap-3" style={{ marginTop: "var(--space-6)" }}>
                <button className="btn btn-primary btn-lg" onClick={handleCopy}>
                  {copied ? "Copied!" : "Copy OTP"}
                </button>
                <button
                  className="btn btn-secondary btn-lg"
                  onClick={async () => {
                    if (!session || !platformId) return;
                    setLoading(true);
                    setError(null);
                    try {
                      await generateOtp(session.id, platformId);
                    } catch (err) {
                      setError(err instanceof Error ? err.message : "Failed to generate OTP");
                    }
                    setLoading(false);
                  }}
                >
                  Regenerate
                </button>
              </div>

              {/* Clipboard Countdown Timer (visible to analyst) */}
              {clipboardActive && (
                <div className="clipboard-timer" style={{ marginTop: "var(--space-6)" }}>
                  <span>Clipboard auto-clears in</span>
                  <strong>{clipboardCountdown}s</strong>
                  <div className="clipboard-timer-bar">
                    <div
                      className="clipboard-timer-fill"
                      style={{ width: `${progresses}%` }}
                    />
                  </div>
                </div>
              )}
            </>
          ) : (
            <p className="text-secondary" style={{ padding: "var(--space-8)" }}>
              Select a platform to generate an OTP
            </p>
          )}
        </div>
      </div>

      {/* Security Notice */}
      <div className="alert alert-warning">
        <span>
          OTP will remain in your clipboard for {selectedOtp?.clipboard_timeout_secs ?? 30} seconds.
          All OTP operations are audited and attributed to your session.
        </span>
      </div>
    </div>
  );
}
