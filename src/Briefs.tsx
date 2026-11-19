import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Plus, X } from "lucide-react";
import type { MemoryItem } from "./Memory";
interface Brief {
  id: string;
  title: string;
  content: string;
  approved: boolean;
  private: boolean;
  sensitive: boolean;
  blocked: boolean;
  source_ids: string[];
  updated_at: number;
}
export default function Briefs() {
  const [items, setItems] = useState<Brief[]>([]);
  const [notes, setNotes] = useState<MemoryItem[]>([]);
  const [editing, setEditing] = useState<Brief | null | undefined>();
  const [error, setError] = useState("");
  const [deleting, setDeleting] = useState<string | null>(null);
  const [restricted, setRestricted] = useState(false);
  const refresh = () =>
    Promise.all([
      invoke<Brief[]>("list_briefs").then(setItems),
      invoke<MemoryItem[]>("list_memory").then(setNotes),
    ]).catch((e) => setError(String(e)));
  useEffect(() => {
    void refresh();
  }, []);
  async function remove(id: string) {
    try {
      await invoke("delete_brief", { id });
      setDeleting(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }
  return (
    <div className="secondary-page">
      <div className="eyebrow">LAYER 03 · THE STORY YOU CHOOSE TO SHARE</div>
      <div className="section-heading">
        <div>
          <h1>Briefs with boundaries.</h1>
          <p className="page-intro">
            Turn approved memories into a project spec, handoff, or reusable
            context pack.
          </p>
        </div>
        <button className="button primary" onClick={() => setEditing(null)}>
          <Plus size={15} />
          New brief
        </button>
      </div>
      <label className="checkbox-row">
        <input
          type="checkbox"
          checked={restricted}
          onChange={(e) => setRestricted(e.target.checked)}
        />
        Review private, sensitive, or blocked briefs
      </label>
      {error && <p className="form-error">{error}</p>}
      {items
        .filter((b) => restricted || (!b.private && !b.sensitive && !b.blocked))
        .map((b) => (
          <article className="settings-card" key={b.id}>
            <div className="memory-meta">
              <h2>{b.title}</h2>
              <span className="approval">
                {b.private || b.sensitive
                  ? "Restricted"
                  : b.blocked
                    ? "Source review needed"
                    : b.approved
                      ? "Approved for sharing"
                      : "Draft"}
              </span>
            </div>
            <p className="brief-excerpt">{b.content}</p>
            <p className="fine-print">
              {b.source_ids.length} linked memories · Changes to supporting
              notes require a new review.
            </p>
            <div className="card-actions">
              <button onClick={() => setEditing(b)}>Edit & review</button>
              {deleting === b.id ? (
                <>
                  <button onClick={() => void remove(b.id)}>
                    Delete permanently
                  </button>
                  <button onClick={() => setDeleting(null)}>Cancel</button>
                </>
              ) : (
                <button onClick={() => setDeleting(b.id)}>Delete</button>
              )}
            </div>
          </article>
        ))}
      {items.length === 0 && (
        <div className="empty-layer">
          <h3>The whole history isn’t the handoff.</h3>
          <p>
            Choose what a new agent needs to know. Keep the source material in
            your vault.
          </p>
        </div>
      )}
      {editing !== undefined && (
        <BriefEditor
          brief={editing || undefined}
          notes={notes}
          onClose={() => setEditing(undefined)}
          onSaved={() => void refresh()}
        />
      )}
    </div>
  );
}
function BriefEditor({
  brief,
  notes,
  onClose,
  onSaved,
}: {
  brief?: Brief;
  notes: MemoryItem[];
  onClose: () => void;
  onSaved: () => void;
}) {
  const [title, setTitle] = useState(brief?.title || "");
  const [content, setContent] = useState(brief?.content || "");
  const [sources, setSources] = useState<string[]>(brief?.source_ids || []);
  const [approved, setApproved] = useState(false);
  const [privateBrief, setPrivate] = useState(brief?.private || false);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const available = notes.filter(
    (m) => m.approved && !m.private && !m.sensitive && !m.source_restricted,
  );
  function addSource(m: MemoryItem) {
    if (sources.includes(m.id)) return;
    setSources((previous) => [...previous, m.id]);
    setContent(
      (previous) =>
        previous + `${previous ? "\n\n" : ""}## ${m.kind}\n${m.content}`,
    );
    setApproved(false);
  }
  async function save(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await invoke("save_brief", {
        input: {
          id: brief?.id || null,
          title,
          content,
          approved,
          private: privateBrief,
          source_ids: sources,
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
        className="editor-modal wide"
        role="dialog"
        aria-modal="true"
        aria-label="Edit brief"
      >
        <button
          className="modal-close icon-button"
          aria-label="Close brief editor"
          onClick={onClose}
        >
          <X size={17} />
        </button>
        <div className="eyebrow">CURATED, NOT AUTOMATIC</div>
        <h2>{brief ? "Review your brief." : "Write the handoff."}</h2>
        <form onSubmit={save}>
          <label>
            Title
            <input
              autoFocus
              value={title}
              maxLength={200}
              onChange={(e) => {
                setTitle(e.target.value);
                setApproved(false);
              }}
              required
            />
          </label>
          <label>
            Add an approved memory
            <select
              value=""
              onChange={(e) => {
                const m = available.find((m) => m.id === e.target.value);
                if (m) addSource(m);
              }}
            >
              <option value="">Choose a source to include…</option>
              {available
                .filter((m) => !sources.includes(m.id))
                .map((m) => (
                  <option value={m.id} key={m.id}>
                    {m.kind}: {m.content.slice(0, 100)}
                  </option>
                ))}
            </select>
          </label>
          <p className="fine-print">
            {sources.length} linked memories. Source links are retained so
            privacy decisions continue to apply.
          </p>
          <label>
            Brief (Markdown)
            <textarea
              rows={11}
              value={content}
              maxLength={32000}
              onChange={(e) => {
                setContent(e.target.value);
                setApproved(false);
              }}
              required
            />
          </label>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={privateBrief}
              onChange={(e) => setPrivate(e.target.checked)}
            />
            Keep this brief private
          </label>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={approved}
              onChange={(e) => setApproved(e.target.checked)}
            />
            I reviewed the brief and its current sources; approve it for sharing
          </label>
          <p className="fine-print">
            Private, sensitive, deleted, or unapproved sources always block
            sharing. If a source was deleted, create a fresh brief with valid
            sources.
          </p>
          {error && <p className="form-error">{error}</p>}
          <div className="form-actions">
            <button
              className="button secondary"
              type="button"
              onClick={onClose}
            >
              Cancel
            </button>
            <button className="button primary" disabled={busy}>
              {busy ? "Saving…" : "Save brief"}
            </button>
          </div>
        </form>
      </section>
    </div>
  );
}
