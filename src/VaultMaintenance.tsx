import ChoiceSelect from "./ChoiceSelect";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Preview {
  review_id: string;
  operation: "restore" | "password";
  conversations: number;
  messages: number;
  memories: number;
  briefs: number;
  clients_to_remove: number;
  previous_vault: string | null;
}
interface Result {
  previous_vault: string | null;
  durability_warning: boolean;
}

export default function VaultMaintenance({
  locked = false,
  onChanged,
  onBusyChange,
}: {
  locked?: boolean;
  onBusyChange?: (busy: boolean) => void;
  onChanged: (message: string) => void;
}) {
  const [mode, setMode] = useState<"restore" | "password">("restore");
  const [path, setPath] = useState("");
  const [password, setPassword] = useState("");
  const [current, setCurrent] = useState("");
  const [confirm, setConfirm] = useState("");
  const [preview, setPreview] = useState<Preview | null>(null);
  const [accepted, setAccepted] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const passwordInput = useRef<HTMLInputElement>(null);
  const currentInput = useRef<HTMLInputElement>(null);
  const reviewCheckbox = useRef<HTMLInputElement>(null);
  const nextFocus = useRef<"password" | "review" | null>(null);
  useEffect(() => {
    if (busy || !nextFocus.current) return;
    const target =
      nextFocus.current === "review"
        ? reviewCheckbox.current
        : mode === "password"
          ? currentInput.current
          : passwordInput.current;
    if (target) {
      nextFocus.current = null;
      target.focus();
    }
  }, [busy, preview, mode, error]);
  useEffect(() => {
    onBusyChange?.(busy);
  }, [busy, onBusyChange]);
  const pending = useRef<string | null>(null);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      if (pending.current)
        void invoke("cancel_replacement", { reviewId: pending.current }).catch(
          () => {},
        );
    };
  }, []);
  async function cancel() {
    if (pending.current)
      await invoke("cancel_replacement", { reviewId: pending.current }).catch(
        () => {},
      );
    pending.current = null;
    nextFocus.current = "password";
    setPreview(null);
    setAccepted(false);
    setPassword("");
    setCurrent("");
    setConfirm("");
  }
  async function choose() {
    setError("");
    try {
      const value = await open({
        title: "Choose a standalone encrypted Pre-Drink backup",
        multiple: false,
        directory: false,
      });
      if (typeof value === "string") {
        setPath(value);
        passwordInput.current?.focus();
      }
    } catch (e) {
      setError(String(e));
    }
  }
  async function prepare(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (mode === "password" && password !== confirm) {
      setError("The new passwords do not match.");
      return;
    }
    setBusy(true);
    try {
      const result =
        mode === "restore"
          ? await invoke<Preview>("prepare_restore", { path, password })
          : await invoke<Preview>("prepare_password_change", {
              currentPassword: current,
              newPassword: password,
            });
      if (!mounted.current) {
        void invoke("cancel_replacement", { reviewId: result.review_id });
        return;
      }
      pending.current = result.review_id;
      nextFocus.current = "review";
      setPreview(result);
      setAccepted(false);
      setCurrent("");
      setConfirm("");
    } catch (e) {
      nextFocus.current = "password";
      setError(String(e));
      setPassword("");
      setCurrent("");
      setConfirm("");
    } finally {
      setBusy(false);
    }
  }
  async function apply() {
    if (!preview || !accepted) return;
    setBusy(true);
    setError("");
    try {
      const result = await invoke<Result>("apply_replacement", {
        reviewId: preview.review_id,
        password,
      });
      pending.current = null;
      setPassword("");
      setPreview(null);
      const message =
        mode === "restore"
          ? "Backup restored. Unlock with the backup password."
          : "Password changed. Unlock with your new password.";
      onChanged(
        `${message} Agent registrations were removed and sharing is off.${result.previous_vault ? ` Previous encrypted vault preserved at: ${result.previous_vault}. It still needs its previous password.` : ""}${result.durability_warning ? " The switch completed, but the folder could not be flushed to disk. Preserve all vault folders and verify after restarting." : ""}`,
      );
    } catch (e) {
      nextFocus.current = "password";
      setError(String(e));
      pending.current = null;
      setPreview(null);
      setAccepted(false);
      setPassword("");
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="settings-card maintenance-card">
      <h2>
        {locked ? "Restore an encrypted backup" : "Restore or change password"}
      </h2>
      {!preview ? (
        <>
          {!locked && (
            <label>
              Operation
              <ChoiceSelect
                label="Operation"
                value={mode}
                disabled={busy}
                onChange={(value) => {
                  setMode(value as "restore" | "password");
                  setPassword("");
                  setCurrent("");
                  setConfirm("");
                  setError("");
                }}
                options={[
                  { value: "restore", label: "Restore a backup" },
                  { value: "password", label: "Change vault password" },
                ]}
              />
            </label>
          )}
          <p>
            {mode === "restore"
              ? "Inspect a backup before switching. This replaces the active library; it does not merge it. Your current encrypted vault is preserved for recovery."
              : "Create and verify a separately encrypted replacement. The previous encrypted vault remains as a recovery copy under the old password."}
          </p>
          <form onSubmit={(e) => void prepare(e)}>
            {mode === "restore" ? (
              <>
                <button
                  type="button"
                  className="button secondary"
                  disabled={busy}
                  onClick={() => void choose()}
                >
                  Choose encrypted backup
                </button>
                {path && <code className="database-path">{path}</code>}
              </>
            ) : (
              <label>
                Current password
                <input
                  ref={currentInput}
                  type="password"
                  autoComplete="current-password"
                  value={current}
                  maxLength={1024}
                  required
                  disabled={busy}
                  onChange={(e) => setCurrent(e.target.value)}
                />
              </label>
            )}
            <label>
              {mode === "restore" ? "Backup password" : "New vault password"}
              <input
                ref={passwordInput}
                type="password"
                autoComplete={mode === "restore" ? "off" : "new-password"}
                value={password}
                minLength={mode === "password" ? 14 : undefined}
                maxLength={1024}
                required
                disabled={busy}
                onChange={(e) => setPassword(e.target.value)}
              />
            </label>
            {mode === "password" && (
              <label>
                Confirm new password
                <input
                  type="password"
                  autoComplete="new-password"
                  value={confirm}
                  maxLength={1024}
                  required
                  disabled={busy}
                  onChange={(e) => setConfirm(e.target.value)}
                />
              </label>
            )}
            <button
              className="button secondary"
              disabled={busy || (mode === "restore" && !path)}
            >
              {busy
                ? "Preparing and verifying…"
                : mode === "restore"
                  ? "Inspect backup"
                  : "Prepare password change"}
            </button>
          </form>
        </>
      ) : (
        <div aria-label="Review vault replacement">
          <h3>Verified replacement</h3>
          <p>
            {preview.conversations} conversations · {preview.messages} messages
            · {preview.memories} memories · {preview.briefs} briefs
          </p>
          <p>
            {preview.clients_to_remove} agent registrations will be removed.
            Reauthorize clients yourself after unlocking. Sharing stays off.
          </p>
          {preview.previous_vault && (
            <>
              <p>
                The current vault will remain encrypted at this location, with
                its existing password:
              </p>
              <code className="database-path">{preview.previous_vault}</code>
            </>
          )}
          <p className="fine-print">
            Existing backups and retained recovery copies keep their previous
            passwords. A password change does not erase those copies. After
            verifying the new vault and an independent backup, follow the
            recovery guide before removing old files.
          </p>
          <label className="checkbox-row">
            <input
              ref={reviewCheckbox}
              type="checkbox"
              checked={accepted}
              disabled={busy}
              onChange={(e) => setAccepted(e.target.checked)}
            />
            I reviewed this replacement and understand which password will
            unlock it.
          </label>
          <div className="form-actions">
            <button
              className="button secondary"
              disabled={busy}
              onClick={() => void cancel()}
            >
              Cancel
            </button>
            <button
              className="button primary"
              disabled={busy || !accepted}
              onClick={() => void apply()}
            >
              {busy
                ? "Switching safely…"
                : mode === "restore"
                  ? "Restore verified backup"
                  : "Change password & lock"}
            </button>
          </div>
        </div>
      )}
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      <p className="fine-print">
        Preparing stops agent sharing. Keep an independent encrypted backup on
        another drive. Nothing is uploaded and passwords are not saved.
      </p>
    </section>
  );
}
