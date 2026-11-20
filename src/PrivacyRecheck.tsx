import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
interface Preview {
  flagged: number;
  cleared: number;
  approvals_to_reset: number;
  review_hash: string;
}
export default function PrivacyRecheck({
  onApplied,
}: {
  onApplied: () => void;
}) {
  const [preview, setPreview] = useState<Preview | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  async function inspect() {
    setBusy(true);
    setMessage("");
    setPreview(null);
    try {
      setPreview(await invoke<Preview>("preview_privacy_recheck"));
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function apply() {
    if (!preview) return;
    setBusy(true);
    setMessage("");
    try {
      await invoke("apply_privacy_recheck", {
        reviewHash: preview.review_hash,
      });
      setPreview(null);
      setMessage(
        "Flags rechecked. Agent sharing is off. Review any notes and briefs returned to draft before sharing again.",
      );
      onApplied();
    } catch (e) {
      setMessage(String(e));
      setPreview(null);
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="settings-card">
      <h2>Recheck sensitive flags</h2>
      <p>
        Older rules could mistake timestamps and technical discussions for
        sensitive data. Review how the current rules would classify your saved
        content. Detected credentials remain blocked.
      </p>
      <button
        className="button secondary"
        disabled={busy}
        onClick={() => void inspect()}
      >
        {busy ? "Checking…" : "Preview flag changes"}
      </button>
      {preview && (
        <div role="region" aria-label="Flag recheck preview">
          <p>
            {preview.cleared} flags would clear · {preview.flagged} new flags ·{" "}
            {preview.approvals_to_reset} approvals would return to draft.
          </p>
          <p className="fine-print">
            Counts include conversations, messages, notes and briefs. Applying
            stops agent sharing. If classifications change, all approved notes
            and briefs return to draft. Manual private flags stay private.
            Cleared conversations may become available to chat-enabled clients
            if you later turn sharing back on.
          </p>
          <div className="form-actions">
            <button
              className="button secondary"
              disabled={busy}
              onClick={() => setPreview(null)}
            >
              Cancel
            </button>
            <button
              className="button primary"
              disabled={busy}
              onClick={() => void apply()}
            >
              Apply recheck &amp; stop sharing
            </button>
          </div>
        </div>
      )}
      {message && <p role="status">{message}</p>}
    </section>
  );
}
