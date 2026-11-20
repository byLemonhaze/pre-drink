import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
interface Preview {
  flagged: number;
  cleared: number;
  approvals_to_reset: number;
  review_hash: string;
  clear_candidates: { key: string; kind: string; content: string }[];
}
export default function PrivacyRecheck({
  onApplied,
}: {
  onApplied: () => void;
}) {
  const [preview, setPreview] = useState<Preview | null>(null);
  const [selected, setSelected] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  async function inspect() {
    setBusy(true);
    setMessage("");
    setPreview(null);
    setSelected([]);
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
        clearIds: selected,
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
            {preview.cleared} possible false flags · {selected.length} selected
            to clear · {preview.flagged} new flags ·{" "}
            {preview.approvals_to_reset} approvals would return to draft.
          </p>
          <p className="fine-print">
            Counts include conversations, messages, notes and briefs. Applying
            stops agent sharing. If classifications change, all approved notes
            and briefs return to draft. Manual private flags stay private.
            Cleared conversations may become available to chat-enabled clients
            if you later turn sharing back on.
          </p>
          <p>
            Nothing is selected automatically. Read every item before clearing
            it. Clear flagged messages before (or together with) their
            conversation. Up to 100 complete items are shown per review; items
            over 16 KB stay restricted.
          </p>
          {preview.clear_candidates.map((item) => (
            <div key={item.key} className="settings-card">
              <label>
                <input
                  type="checkbox"
                  disabled={busy}
                  checked={selected.includes(item.key)}
                  onChange={(event) =>
                    setSelected((ids) =>
                      event.target.checked
                        ? [...ids, item.key]
                        : ids.filter((id) => id !== item.key),
                    )
                  }
                />
                Clear this {item.kind} flag
              </label>
              <pre
                style={{
                  whiteSpace: "pre-wrap",
                  overflowWrap: "anywhere",
                  maxHeight: 280,
                  overflow: "auto",
                }}
              >
                {item.content}
              </pre>
            </div>
          ))}
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
