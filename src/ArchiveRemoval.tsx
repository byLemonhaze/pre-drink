import Modal from "./Modal";
import { Trash2 } from "lucide-react";
import { useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
interface Preview {
  title: string;
  messages: number;
  snapshots: number;
  memories: number;
  briefs: number;
  pending_import: { label: string; saved: number; total: number } | null;
  review_hash: string;
  session_generation: number;
}
export function PendingImportRemovalNotice({
  pending,
}: {
  pending: Preview["pending_import"];
}) {
  if (!pending) return null;
  return (
    <p className="notice warning">
      This also discards the entire unfinished import of {pending.label}:{" "}
      {pending.saved.toLocaleString()} of {pending.total.toLocaleString()}{" "}
      conversations prepared. This prevents its saved copy or remaining ZIP
      contents from bringing the deleted conversation back when resumed. Other
      conversations already in your archive stay unchanged. Selecting that ZIP
      again starts a new import and can restore deleted conversations.
    </p>
  );
}
export default function ArchiveRemoval({
  conversationId,
  compact = false,
  onChanged,
}: {
  conversationId?: string;
  compact?: boolean;
  onChanged: () => void;
}) {
  const opener = useRef<HTMLButtonElement>(null);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [confirmation, setConfirmation] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const input = {
    kind: conversationId ? "conversation" : "snapshots",
    conversation_id: conversationId || null,
  };
  async function inspect() {
    setBusy(true);
    setMessage("");
    setConfirmation("");
    try {
      setPreview(await invoke<Preview>("preview_removal", { input }));
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function remove() {
    if (!preview || confirmation !== "DELETE") return;
    setBusy(true);
    setMessage("");
    try {
      await invoke("apply_removal", {
        input,
        reviewHash: preview.review_hash,
        sessionGeneration: preview.session_generation,
      });
      setPreview(null);
      setConfirmation("");
      setMessage(
        conversationId
          ? "Conversation deleted. Derived notes remain blocked. Agent sharing is off."
          : "Older snapshots removed. Messages and current raw snapshots remain. Agent sharing is off.",
      );
      onChanged();
    } catch (e) {
      setMessage(String(e));
      setPreview(null);
    } finally {
      setBusy(false);
    }
  }
  return (
    <div className="archive-removal">
      <button
        ref={opener}
        className={compact ? "icon-button" : "button secondary"}
        aria-label={compact ? "Delete conversation…" : undefined}
        title={compact ? "Delete conversation…" : undefined}
        disabled={busy}
        onClick={() => void inspect()}
      >
        {compact ? (
          <Trash2 size={16} />
        ) : conversationId ? (
          "Delete conversation…"
        ) : (
          "Review older snapshots…"
        )}
      </button>
      {message && <p role="status">{message}</p>}
      {preview && (
        <Modal
          label="Review archive deletion"
          returnFocus={opener}
          busy={busy}
          onClose={() => setPreview(null)}
        >
          <div className="eyebrow">REVIEW PERMANENT DELETION</div>
          <h2>{preview.title}</h2>
          <p>
            {conversationId
              ? `${preview.messages} messages and ${preview.snapshots} raw snapshots will be deleted from this vault and its search index. ${preview.memories} linked memories will remain as blocked drafts; ${preview.briefs} dependent briefs will stay blocked by their missing evidence.`
              : `${preview.snapshots} older raw JSON snapshots will be removed. The most recently imported raw version of each conversation, all message branches, notes and briefs remain.`}
          </p>
          <PendingImportRemovalNotice pending={preview.pending_import} />
          <p>
            There is no undo here. Original export ZIPs, encrypted backups,
            retained vault generations and Mac snapshots can still contain this
            data. Deleting does not securely erase those copies.
          </p>
          <label>
            Type DELETE to confirm
            <input
              data-initial-focus
              value={confirmation}
              disabled={busy}
              onChange={(e) => setConfirmation(e.target.value)}
              autoComplete="off"
            />
          </label>
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
              disabled={
                busy ||
                confirmation !== "DELETE" ||
                (!conversationId && !preview.snapshots)
              }
              onClick={() => void remove()}
            >
              {busy ? "Deleting…" : "Delete reviewed data"}
            </button>
          </div>
        </Modal>
      )}
    </div>
  );
}
