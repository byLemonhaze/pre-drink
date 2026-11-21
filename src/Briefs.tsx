import ChoiceSelect from "./ChoiceSelect";
import Modal from "./Modal";
import PagedList from "./PagedList";
import ReviewDialog from "./Review";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Plus, X } from "lucide-react";
import type { MemoryItem } from "./Memory";
export interface Brief {
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
  const [reviewing, setReviewing] = useState<string | null>(null);
  const [restricted, setRestricted] = useState(false);
  const [query, setQuery] = useState("");
  const shown = items.filter(
    (b) =>
      (restricted || (!b.private && !b.sensitive && !b.blocked)) &&
      `${b.title} ${b.content}`
        .toLocaleLowerCase()
        .includes(query.trim().toLocaleLowerCase()),
  );
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
      <div className="eyebrow">03 / CURATED CONTEXT</div>
      <div className="section-heading">
        <div>
          <h1>Briefs</h1>
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
      <div className="memory-filters">
        <input
          type="search"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
          aria-label="Search briefs"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Find a brief…"
        />
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={restricted}
            onChange={(e) => setRestricted(e.target.checked)}
          />
          Review private, sensitive, or blocked briefs
        </label>
      </div>
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      <PagedList
        items={shown}
        scope={JSON.stringify([query, restricted])}
        label="briefs"
      >
        {(b) => (
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
              <button onClick={() => setReviewing(b.id)}>
                Review &amp; approve
              </button>
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
        )}
      </PagedList>
      {shown.length === 0 && (
        <div className="empty-layer">
          <h3>
            {items.length
              ? "No matching briefs."
              : "The whole history isn’t the handoff."}
          </h3>
          <p>
            {items.length
              ? "Try a different search or privacy filter."
              : "Choose what a new agent needs to know. Keep the source material in your vault."}
          </p>
        </div>
      )}
      {reviewing && (
        <ReviewDialog
          kind="brief"
          id={reviewing}
          onClose={() => setReviewing(null)}
          onApproved={() => void refresh()}
        />
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
  const [privateBrief, setPrivate] = useState(brief?.private || false);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [sourceQuery, setSourceQuery] = useState("");
  const available = notes.filter(
    (m) => m.approved && !m.private && !m.sensitive && !m.source_restricted,
  );
  const sourceMatches = available.filter(
    (m) =>
      !sources.includes(m.id) &&
      m.content
        .toLocaleLowerCase()
        .includes(sourceQuery.trim().toLocaleLowerCase()),
  );
  function addSource(m: MemoryItem) {
    if (sources.includes(m.id)) return;
    setSources((previous) => [...previous, m.id]);
    setContent(
      (previous) =>
        previous + `${previous ? "\n\n" : ""}## ${m.kind}\n${m.content}`,
    );
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
    <Modal label="Edit brief" wide busy={busy} onClose={onClose}>
      <button
        className="modal-close icon-button"
        aria-label="Close brief editor"
        disabled={busy}
        onClick={onClose}
      >
        <X size={17} />
      </button>
      <div className="eyebrow">CURATED, NOT AUTOMATIC</div>
      <h2>{brief ? "Edit brief" : "New brief"}</h2>
      <form onSubmit={save}>
        <label>
          Title
          <input
            data-initial-focus
            value={title}
            maxLength={200}
            onChange={(e) => {
              setTitle(e.target.value);
            }}
            required
          />
        </label>
        <label>
          Find an approved memory
          <input
            type="search"
            autoCorrect="off"
            autoCapitalize="off"
            spellCheck={false}
            value={sourceQuery}
            onChange={(e) => setSourceQuery(e.target.value)}
            placeholder="Search all approved source memories…"
          />
        </label>
        <label>
          Add an approved memory
          <ChoiceSelect
            label="Add an approved memory"
            value=""
            onChange={(value) => {
              const m = available.find((m) => m.id === value);
              if (m) addSource(m);
            }}
            options={[
              { value: "", label: "Choose a source to include…" },
              ...sourceMatches.slice(0, 50).map((m) => ({
                value: m.id,
                label: `${m.kind}: ${m.content.slice(0, 100)}`,
              })),
            ]}
          />
        </label>
        <p className="fine-print">
          {sourceMatches.length > 50 &&
            "Showing the first 50 matching sources. Refine your search to find another. "}
          {sources.length} linked memories. Source links are retained so privacy
          decisions continue to apply.
        </p>
        <label>
          Brief (Markdown)
          <textarea
            rows={11}
            value={content}
            maxLength={32000}
            onChange={(e) => {
              setContent(e.target.value);
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
        <p className="fine-print">
          Saving returns this item to draft. Use Review &amp; approve on its
          card to share the saved version.
        </p>
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
            disabled={busy}
            onClick={onClose}
          >
            Cancel
          </button>
          <button className="button primary" disabled={busy}>
            {busy ? "Saving…" : "Save brief"}
          </button>
        </div>
      </form>
    </Modal>
  );
}
