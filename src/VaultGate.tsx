import { useCallback, useEffect, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { GlassWater, LockKeyhole, ShieldCheck } from "lucide-react";
import App from "./App";
import { listen } from "@tauri-apps/api/event";

interface Status {
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
  const generation = useRef(0);
  const lastTouch = useRef(0);
  const refresh = useCallback(async () => {
    if (!isTauri()) return;
    const current = generation.current;
    try {
      const next = await invoke<Status>("vault_status");
      if (current === generation.current) setStatus(next);
    } catch {
      if (current === generation.current) {
        setStatus(null);
        setError("Could not reach your vault. Reopen Pre-Drink.");
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
  if (status?.unlocked)
    return <App onLock={() => void lock()} sharing={status.sharing} />;
  const create = status && !status.exists;
  return (
    <div className="vault-gate">
      <div className="unlock-card">
        <div className="brand">
          <div className="brand-symbol">
            <GlassWater size={25} />
          </div>
          <span>pre-drink.</span>
        </div>
        <div className="eyebrow">LOCAL VAULT / ENCRYPTED</div>
        <h1>{create ? "Create vault" : "Unlock vault"}</h1>
        <p>
          {create
            ? "Create your encrypted vault with a long, unique password. Only you can unlock it."
            : "Unlock your archive, memory, and curated briefs."}
        </p>
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
                The app verifies the encrypted copy before switching. Existing
                exports and Mac backups remain outside this protection.
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
          <ShieldCheck size={17} />
          <span>
            Argon2id · AES-256 · authenticated database pages
            <br />
            Locks after 15 minutes without activity in Pre-Drink.
          </span>
        </div>
        <p className="fine-print">
          {create
            ? "Use at least 14 characters; a unique phrase of several random words is better. There is no password reset or recovery key. Keep your password and encrypted backups safe."
            : "Your password stays on this Mac. Locking also stops agent access. There is no password reset."}
        </p>
        {!isTauri() && (
          <p className="fine-print">
            Interface preview. Open the Mac app to create or unlock a vault.
          </p>
        )}
      </div>
    </div>
  );
}
