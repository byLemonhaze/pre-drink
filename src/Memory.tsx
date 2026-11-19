import ReviewDialog from "./Review";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FileText, Plus, ShieldCheck, X } from "lucide-react";
import type { Message } from "./types";

export interface MemoryItem {
  id: string;
  kind: string;
  content: string;
  source_message_id: string | null;
  source_conversation_id: string | null;
  source_title: string | null;
  approved: boolean;
  private: boolean;
  sensitive: boolean;
  source_restricted: boolean;
  created_at: number;
  updated_at: number;
}
export function MemoryEditor({
  memory,
  source,
  onClose,
  onSaved,
}: {
  memory?: MemoryItem;
  source?: Message;
  onClose: () => void;
  onSaved: () => void;
}) {
  const [kind, setKind] = useState(memory?.kind || "fact");
  const [content, setContent] = useState(
    memory?.content || source?.content || "",
  );
  const [privateNote, setPrivate] = useState(memory?.private || false);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  async function save(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      await invoke("save_memory", {
        input: {
          id: memory?.id || null,
          kind,
          content,
          source_message_id: memory?.source_message_id || source?.id || null,
          private: privateNote,
        },
      });
      onSaved();
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <div className="modal-backdrop">
      <section
        className="editor-modal"
        role="dialog"
        aria-modal="true"
        aria-label="Edit memory"
      >
        <button
          className="modal-close icon-button"
          aria-label="Close memory editor"
          onClick={onClose}
        >
          <X size={17} />
        </button>
        <div className="eyebrow">LAYER 02 · MEMORY</div>
        <h2>{memory ? "Edit memory" : "New memory"}</h2>
        <p>
          Turn the conversation into one clear, lasting note. Your source stays
          linked; your original chat stays intact.
        </p>
        <form onSubmit={save}>
          <label>
            Type
            <select value={kind} onChange={(e) => setKind(e.target.value)}>
              <option value="fact">Fact</option>
              <option value="decision">Decision</option>
              <option value="project">Project</option>
            </select>
          </label>
          <label>
            Memory
            <textarea
              autoFocus
              rows={8}
              value={content}
              maxLength={16000}
              onChange={(e) => {
                setContent(e.target.value);
              }}
              required
            />
          </label>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={privateNote}
              onChange={(e) => setPrivate(e.target.checked)}
            />
            Keep this memory private
          </label>
          <p className="fine-print">
            Saving returns this item to draft. Use Review &amp; approve on its
            card to share the saved version.
          </p>
          <p className="fine-print">
            Approval never overrides a private or sensitive source. Sharing also
            needs an authorized client and an unlocked vault.
          </p>
          {error && (
            <p className="form-error" role="alert">
              {error}
            </p>
          )}
          <div className="form-actions">
            <button
              type="button"
              className="button secondary"
              onClick={onClose}
            >
              Cancel
            </button>
            <button className="button primary" disabled={busy}>
              {busy ? "Saving…" : "Save memory"}
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}

export function MemoryView({
  onSource,
}: {
  onSource: (id: string, title: string) => void;
}) {
  const [items, setItems] = useState<MemoryItem[]>([]);
  const [filter, setFilter] = useState("all");
  const [query, setQuery] = useState("");
  const [restricted, setRestricted] = useState(false);
  const [editing, setEditing] = useState<MemoryItem | null | undefined>();
  const [error, setError] = useState("");
  const [deleting, setDeleting] = useState<string | null>(null);
  const [reviewing, setReviewing] = useState<string | null>(null);
  const refresh = () =>
    invoke<MemoryItem[]>("list_memory")
      .then(setItems)
      .catch((e) => setError(String(e)));
  useEffect(() => {
    void refresh();
  }, []);
  async function remove(id: string) {
    try {
      await invoke("delete_memory", { id });
      setDeleting(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }
  const shown = items.filter(
    (m) =>
      (filter === "all" || m.kind === filter) &&
      (restricted || (!m.private && !m.sensitive && !m.source_restricted)) &&
      m.content.toLowerCase().includes(query.toLowerCase()),
  );
  return (
    <div className="secondary-page memory-page">
      <div className="eyebrow">02 / REVIEWED KNOWLEDGE</div>
      <div className="section-heading">
        <div>
          <h1>Memory</h1>
          <p className="page-intro">
            Facts, decisions, and projects. Written and approved by you.
          </p>
        </div>
        <button className="button primary" onClick={() => setEditing(null)}>
          <Plus size={15} />
          New memory
        </button>
      </div>
      <div className="memory-filters">
        <select
          aria-label="Memory type"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        >
          <option value="all">All memories</option>
          <option value="fact">Facts</option>
          <option value="decision">Decisions</option>
          <option value="project">Projects</option>
        </select>
        <input
          aria-label="Search memory"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Find a note…"
        />
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={restricted}
            onChange={(e) => setRestricted(e.target.checked)}
          />
          Review restricted notes
        </label>
      </div>
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      <div className="memory-grid">
        {shown.map((m) => (
          <article className="settings-card memory-card" key={m.id}>
            <div className="memory-meta">
              <span>{m.kind.toUpperCase()}</span>
              <span
                className={`approval ${m.approved && !m.private && !m.sensitive && !m.source_restricted ? "ready" : ""}`}
              >
                {m.private || m.sensitive || m.source_restricted
                  ? "Restricted"
                  : m.approved
                    ? "Approved"
                    : "Draft"}
              </span>
            </div>
            <p className="memory-content">{m.content}</p>
            {m.source_conversation_id && (
              <button
                className="source-link"
                onClick={() =>
                  onSource(m.source_conversation_id!, m.source_title || "")
                }
              >
                <FileText size={12} />
                {m.source_title || "View source"}
              </button>
            )}
            <div className="card-actions">
              <button onClick={() => setReviewing(m.id)}>
                Review &amp; approve
              </button>
              <button onClick={() => setEditing(m)}>Edit</button>
              {deleting === m.id ? (
                <>
                  <span>Remove this note?</span>
                  <button onClick={() => void remove(m.id)}>
                    Delete permanently
                  </button>
                  <button onClick={() => setDeleting(null)}>Cancel</button>
                </>
              ) : (
                <button onClick={() => setDeleting(m.id)}>Delete</button>
              )}
            </div>
          </article>
        ))}
      </div>
      {shown.length === 0 && (
        <div className="empty-layer">
          <ShieldCheck size={28} />
          <h3>A smaller, clearer version of your history.</h3>
          <p>
            Save a note from any message in your archive, or write one here. New
            notes start as drafts.
          </p>
        </div>
      )}
      {reviewing && (
        <ReviewDialog
          kind="memory"
          id={reviewing}
          onClose={() => setReviewing(null)}
          onApproved={() => void refresh()}
        />
      )}
      {editing !== undefined && (
        <MemoryEditor
          memory={editing || undefined}
          onClose={() => setEditing(undefined)}
          onSaved={() => void refresh()}
        />
      )}
    </div>
  );
}
