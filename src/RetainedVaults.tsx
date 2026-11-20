import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
interface Copy {
  id: string;
  bytes: number;
  modified_at: number;
  files: string[];
  review_hash: string;
}
interface Review {
  copies: Copy[];
  backup_ready: boolean;
}
const phrase = "REMOVE PREVIOUS VAULT";
export default function RetainedVaults() {
  const [review, setReview] = useState<Review | null>(null);
  const [selected, setSelected] = useState<Copy | null>(null);
  const [confirmation, setConfirmation] = useState("");
  const [understood, setUnderstood] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  async function inspect() {
    setBusy(true);
    setReview(null);
    setSelected(null);
    setConfirmation("");
    setUnderstood(false);
    setMessage("");
    try {
      setReview(await invoke<Review>("preview_retained_vaults"));
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function remove() {
    if (!selected || !understood || confirmation !== phrase) return;
    setBusy(true);
    setMessage("");
    try {
      await invoke("remove_retained_vault", {
        id: selected.id,
        reviewHash: selected.review_hash,
        confirmation,
      });
      setMessage(
        "Selected previous copy removed. Agent sharing is off. Backups, exports and Mac snapshots may still retain older data.",
      );
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
      setReview(null);
      setSelected(null);
      setConfirmation("");
      setUnderstood(false);
    }
  }
  return (
    <section className="settings-card">
      <h2>Previous encrypted vault copies</h2>
      <p>
        Password changes and restores retain the previous vault for recovery.
        Each copy can still open with its old password. After checking the
        current vault, save a fresh encrypted backup outside the vault folder
        using the backup form above. Then review previous copies here.
      </p>
      <button
        className="button secondary"
        disabled={busy}
        onClick={() => void inspect()}
      >
        {busy ? "Checking…" : "Review previous copies"}
      </button>
      {review && (
        <div role="region" aria-label="Previous vault copies">
          {!review.backup_ready && (
            <p role="status">
              Removal needs a fresh verified backup outside the vault folder,
              made in this unlocked session. Save one above, then refresh this
              review. Changing vault data or locking requires a new backup.
            </p>
          )}
          {review.copies.length === 0 && <p>No previous copies found.</p>}
          {review.copies.map((copy) => (
            <div className="settings-card" key={copy.id}>
              <strong>
                {copy.id === "original"
                  ? "Original vault before the first replacement"
                  : copy.id}
              </strong>
              <p>
                {(copy.bytes / 1048576).toFixed(2)} MB ·{" "}
                {copy.modified_at
                  ? `Last modified ${new Date(copy.modified_at * 1000).toLocaleString()}`
                  : "Empty retained folder"}
              </p>
              <p>{copy.files.join(", ") || "No remaining files"}</p>
              <button
                className="button secondary"
                disabled={busy || !review.backup_ready}
                onClick={() => {
                  setSelected(copy);
                  setConfirmation("");
                  setUnderstood(false);
                }}
              >
                Review removal
              </button>
            </div>
          ))}
          {selected && (
            <div className="settings-card retained-confirmation">
              <h3>
                Remove{" "}
                {selected.id === "original"
                  ? "the original vault copy"
                  : selected.id}
                ?
              </h3>
              <p>
                This permanently removes the selected copy and its listed
                journals. It cannot be undone. It does not securely erase SSD
                blocks, exports, backups or system snapshots. A backup of the
                current vault may not include information that existed only in
                this older copy.
              </p>
              <label className="checkbox-row">
                <input
                  type="checkbox"
                  checked={understood}
                  disabled={busy}
                  onChange={(e) => setUnderstood(e.target.checked)}
                />{" "}
                I checked the current vault and do not need this older copy for
                recovery.
              </label>
              <label className="confirmation-field">
                Type {phrase}
                <input
                  value={confirmation}
                  disabled={busy}
                  autoComplete="off"
                  onChange={(e) => setConfirmation(e.target.value)}
                />
              </label>
              <div className="form-actions">
                <button
                  className="button secondary"
                  disabled={busy}
                  onClick={() => setSelected(null)}
                >
                  Cancel
                </button>{" "}
                <button
                  className="button primary"
                  disabled={busy || !understood || confirmation !== phrase}
                  onClick={() => void remove()}
                >
                  Remove selected copy &amp; stop sharing
                </button>
              </div>
            </div>
          )}
        </div>
      )}
      {message && <p role="status">{message}</p>}
    </section>
  );
}
