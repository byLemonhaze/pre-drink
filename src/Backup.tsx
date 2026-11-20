import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
export default function Backup() {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  async function backup(e: React.FormEvent) {
    e.preventDefault();
    if (password !== confirm) {
      setMessage("The backup passwords do not match.");
      return;
    }
    setBusy(true);
    setMessage("");
    try {
      const path = await save({
        title: "Save encrypted vault backup",
        defaultPath: `Pre-Drink-${new Date().toISOString().slice(0, 10)}.sqlite3`,
        filters: [{ name: "Encrypted vault", extensions: ["sqlite3"] }],
      });
      if (path) {
        await invoke("backup_vault", { path, password });
        setMessage(
          "Encrypted backup saved and verified. Keep its password safe.",
        );
        setPassword("");
        setConfirm("");
      }
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="settings-card">
      <h2>Encrypted backup</h2>
      <p>
        Your backup includes archive snapshots, memory, and briefs. Choose a
        password for the backup; it can differ from your vault password. A
        forgotten password cannot be recovered.
      </p>
      <form onSubmit={backup}>
        <div className="field-pair">
          <label>
            Backup password
            <input
              type="password"
              autoComplete="new-password"
              minLength={14}
              maxLength={1024}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
            />
          </label>
          <label>
            Confirm backup password
            <input
              type="password"
              autoComplete="new-password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              required
            />
          </label>
        </div>
        <button className="button secondary" disabled={busy}>
          {busy ? "Encrypting and verifying…" : "Save encrypted backup"}
        </button>
      </form>
      {message && <p role="status">{message}</p>}
      <p className="fine-print">
        Backups retain client registrations. In-app restore removes those
        registrations and leaves sharing off. Keep backups on a separate drive.
        Use the restore review below or follow the recovery guide.
      </p>
    </section>
  );
}
