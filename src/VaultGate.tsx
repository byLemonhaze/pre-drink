import ThemeSwitch from "./ThemeSwitch";
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { LockKeyhole, ShieldCheck } from "lucide-react";
import App from "./App";
import Modal from "./Modal";
import VaultMaintenance from "./VaultMaintenance";
import BuildInfo, { type VaultIdentity } from "./BuildInfo";
import { listen } from "@tauri-apps/api/event";

interface Status extends VaultIdentity {
  unlocked: boolean;
  exists: boolean;
  legacy_exists: boolean;
  sharing: boolean;
  database_path: string;
}
export default function VaultGate() {
  const [status, setStatus] = useState<Status | null>(null);
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [removeLegacy, setRemoveLegacy] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [panel, setPanel] = useState<"details" | "restore" | null>(null);
  const [maintenanceBusy, setMaintenanceBusy] = useState(false);
  const generation = useRef(0);
  const lastTouch = useRef(0);
  const reportedReady = useRef(false);
  useEffect(() => {
    if (status && !reportedReady.current) {
      reportedReady.current = true;
      void invoke("startup_stage", { stage: "interface-ready" }).catch(
        () => {},
      );
    }
  }, [status]);
  const refresh = useCallback(async () => {
    if (!isTauri()) return;
    const current = generation.current;
    try {
      const next = await invoke<Status>("vault_status");
      if (current === generation.current) setStatus(next);
    } catch (e) {
      if (current === generation.current) {
        setStatus(null);
        setError(String(e));
      }
    }
  }, []);
  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let stop: (() => void) | undefined;
    listen("vault-locked", () => {
      generation.current++;
      setStatus((previous) =>
        previous ? { ...previous, unlocked: false, sharing: false } : null,
      );
      setPassword("");
      setConfirm("");
    })
      .then((unlisten) => {
        if (disposed) unlisten();
        else stop = unlisten;
      })
      .catch(() => {});
    return () => {
      disposed = true;
      stop?.();
    };
  }, []);
  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => void refresh(), 2000);
    return () => clearInterval(timer);
  }, [refresh]);
  useEffect(() => {
    if (!status?.unlocked) return;
    const activity = () => {
      if (Date.now() - lastTouch.current > 30_000) {
        lastTouch.current = Date.now();
        void invoke("touch_vault").catch(() => refresh());
      }
    };
    window.addEventListener("pointerdown", activity);
    window.addEventListener("keydown", activity);
    return () => {
      window.removeEventListener("pointerdown", activity);
      window.removeEventListener("keydown", activity);
    };
  }, [status?.unlocked, refresh]);
  async function lock() {
    generation.current++;
    setStatus((previous) =>
      previous ? { ...previous, unlocked: false, sharing: false } : null,
    );
    await invoke("lock_vault").catch((e) => setError(String(e)));
    await refresh();
  }
  async function unlock(event: React.FormEvent) {
    event.preventDefault();
    if (!status) return;
    const create = !status.exists;
    if (create && password !== confirm) {
      setError("The passwords do not match.");
      return;
    }
    setBusy(true);
    setError("");
    try {
      await invoke("unlock_vault", {
        password,
        create,
        removeLegacy: create && status.legacy_exists && removeLegacy,
      });
      setPassword("");
      setConfirm("");
      generation.current++;
      await refresh();
    } catch (e) {
      setError(String(e));
      setPassword("");
      setConfirm("");
    } finally {
      setBusy(false);
    }
  }
  function changed(message: string) {
    setPanel(null);
    setMaintenanceBusy(false);
    generation.current++;
    setError("");
    setNotice(message);
    setPassword("");
    setConfirm("");
    void refresh();
  }
  if (status?.unlocked)
    return (
      <App
        onLock={() => void lock()}
        onVaultChanged={changed}
        sharing={status.sharing}
        identity={status}
      />
    );
  const create = status && !status.exists;
  return (
    <div className="vault-gate">
      <div className="unlock-card">
        <header className="lock-brandbar">
          <div className="brand">
            <img className="app-mark" src="/favicon.svg" alt="" />
            <span>pre-drink.</span>
          </div>
          <ThemeSwitch />
        </header>
        <div className="unlock-layout">
          <section className="unlock-story" aria-label="Pre-Drink">
            <h2>
              Your AI history,
              <br />
              <span>sober and ready.</span>
            </h2>
            <p>
              A private home for your conversations.
              <br />
              Better context for whatever comes next.
            </p>
            <div className="local-promise">
              <ShieldCheck size={15} /> Local. Encrypted. Under your control.
            </div>
          </section>
          <section className="unlock-form-panel">
            <div className="eyebrow">YOUR PRIVATE SPACE</div>
            <h1>{create ? "Create your vault." : "Welcome back."}</h1>
            <p className="unlock-intro">
              {create
                ? "One password. A place for your whole AI story."
                : "Your archive, memory and briefs are waiting."}
            </p>
            {notice && (
              <p className="maintenance-notice" role="status">
                {notice}
              </p>
            )}
            <form onSubmit={unlock}>
              <label>
                Vault password
                <input
                  autoFocus
                  type="password"
                  autoComplete={create ? "new-password" : "current-password"}
                  value={password}
                  minLength={create ? 14 : undefined}
                  maxLength={1024}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  disabled={busy}
                />
              </label>
              {create && (
                <label>
                  Confirm password
                  <input
                    type="password"
                    autoComplete="new-password"
                    value={confirm}
                    onChange={(e) => setConfirm(e.target.value)}
                    required
                    disabled={busy}
                  />
                </label>
              )}
              {create && status?.legacy_exists && (
                <div className="migration-note">
                  <strong>Your existing archive will be encrypted.</strong>
                  <p>
                    The app verifies the encrypted copy before switching.
                    Existing exports and Mac backups remain outside this
                    protection.
                  </p>
                  <label className="checkbox-row">
                    <input
                      type="checkbox"
                      checked={removeLegacy}
                      onChange={(e) => setRemoveLegacy(e.target.checked)}
                    />
                    Remove the old unencrypted database after verification
                  </label>
                </div>
              )}
              {error && (
                <p className="form-error" role="alert">
                  {error}
                </p>
              )}
              <button
                className="button primary"
                disabled={busy || !status || !isTauri()}
              >
                <LockKeyhole size={16} />
                {busy
                  ? "Opening securely…"
                  : create
                    ? status?.legacy_exists
                      ? "Encrypt my existing vault"
                      : "Create encrypted vault"
                    : "Unlock vault"}
              </button>
            </form>
            <div className="security-caption">
              <LockKeyhole size={14} />
              <span>
                {create
                  ? "Use a unique phrase of at least 14 characters."
                  : "Your password stays on this Mac."}
              </span>
            </div>
            <p className="fine-print lock-recovery-note">
              {create
                ? "There is no password reset. Keep your password and encrypted backups safe."
                : "Locking stops agent access. There is no password reset."}
            </p>
          </section>
        </div>
        {status && (
          <footer className="unlock-footer">
            <p className="fine-print">
              Pre-Drink {status.app_version} ·{" "}
              {status.isolated_storage
                ? "Separate test storage"
                : "Local vault"}
            </p>
            <div>
              <button
                type="button"
                className="text-button"
                disabled={busy}
                onClick={() => setPanel("details")}
              >
                Vault details
              </button>
              {isTauri() && (
                <button
                  type="button"
                  className="text-button"
                  disabled={busy}
                  onClick={() => setPanel("restore")}
                >
                  Restore backup
                </button>
              )}
            </div>
          </footer>
        )}
        {status && panel && (
          <Modal
            label={panel === "restore" ? "Restore backup" : "Vault details"}
            busy={maintenanceBusy}
            onClose={() => setPanel(null)}
          >
            <button
              className="button secondary"
              data-initial-focus
              disabled={maintenanceBusy}
              onClick={() => setPanel(null)}
            >
              Back to unlock
            </button>
            {panel === "restore" ? (
              <VaultMaintenance
                locked
                onChanged={changed}
                onBusyChange={setMaintenanceBusy}
              />
            ) : (
              <BuildInfo identity={status} expanded />
            )}
          </Modal>
        )}
        {!isTauri() && (
          <p className="fine-print">
            Interface preview. Open the Mac app to create or unlock a vault.
          </p>
        )}
      </div>
    </div>
  );
}
