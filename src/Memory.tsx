import ChoiceSelect from "./ChoiceSelect";
import Modal from "./Modal";
import MemoryCard, { MemoryInspector } from "./MemoryCard";
import LibraryPage, { useLibraryPage } from "./LibraryPage";
import ReviewDialog from "./Review";
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LayoutGrid, List, Plus, ShieldCheck, X } from "lucide-react";
import type { Message } from "./types";

export interface MemoryItem {
  id: string;
  kind: string;
  content: string;
  source_message_id: string | null;
  source_conversation_id: string | null;
  source_title: string | null;
  approved: boolean;
  curator_provider?: string;
  curator_revision?: number;
  publication_basis?: string;
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
    <Modal label="Edit memory" busy={busy} onClose={onClose}>
      <button
        className="modal-close icon-button"
        aria-label="Close memory editor"
        disabled={busy}
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
          <ChoiceSelect
            label="Type"
            value={kind}
            onChange={setKind}
            options={[
              { value: "fact", label: "Fact" },
              { value: "decision", label: "Decision" },
              { value: "project", label: "Project" },
            ]}
          />
        </label>
        <label>
          Memory
          <textarea
            data-initial-focus
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
            disabled={busy}
            onClick={onClose}
          >
            Cancel
          </button>
          <button className="button primary" disabled={busy}>
            {busy ? "Saving…" : "Save memory"}
          </button>
        </div>
      </form>
    </Modal>
  );
}

export function MemoryView({
  onSource,
}: {
  onSource: (id: string, title: string) => void;
}) {
  const [filter, setFilter] = useState("all");
  const [layout, setLayout] = useState<"cards" | "list">("cards");
  const [reading, setReading] = useState<MemoryItem | null>(null);
  const [query, setQuery] = useState("");
  const [restricted, setRestricted] = useState(false);
  const [editing, setEditing] = useState<MemoryItem | null | undefined>();
  const [error, setError] = useState("");
  const [deleting, setDeleting] = useState<string | null>(null);
  const [reviewing, setReviewing] = useState<string | null>(null);
  const library = useLibraryPage<MemoryItem>({
    collection: "memory",
    query,
    kind: filter === "all" ? null : filter,
    include_restricted: restricted,
  });
  const refresh = library.refresh;
  async function remove(id: string) {
    try {
      await invoke("delete_memory", { id });
      setDeleting(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }
  return (
    <div className="secondary-page memory-page">
      <div className="eyebrow">02 / YOUR KNOWLEDGE, CURATED</div>
      <div className="section-heading">
        <div>
          <h1>Memory cabinet</h1>
          <p className="page-intro">
            Facts, decisions, and projects. Your notes and released agent
            contributions.
          </p>
        </div>
        <button className="button primary" onClick={() => setEditing(null)}>
          <Plus size={15} />
          New memory
        </button>
      </div>
      <div className="cabinet-toolbar">
        <div className="folder-tabs" role="group" aria-label="Memory type">
          {[
            ["all", "All memories"],
            ["fact", "Facts"],
            ["decision", "Decisions"],
            ["project", "Projects"],
          ].map(([value, label]) => (
            <button
              key={value}
              aria-pressed={filter === value}
              onClick={() => setFilter(value)}
            >
              {label}
            </button>
          ))}
        </div>
        <div className="view-switch" role="group" aria-label="Memory layout">
          <button
            aria-label="Card view"
            aria-pressed={layout === "cards"}
            onClick={() => setLayout("cards")}
          >
            <LayoutGrid size={16} />
          </button>
          <button
            aria-label="List view"
            aria-pressed={layout === "list"}
            onClick={() => setLayout("list")}
          >
            <List size={16} />
          </button>
        </div>
      </div>
      <div className="memory-filters">
        <input
          aria-label="Search memory"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
          maxLength={200}
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
      <LibraryPage
        state={library}
        label="memories"
        className={`memory-grid${layout === "list" ? " memory-list" : ""}`}
      >
        {(m) => (
          <MemoryCard memory={m} key={m.id} onRead={() => setReading(m)}>
            <div className="card-actions">
              <button
                disabled={!!m.curator_provider}
                onClick={() => setReviewing(m.id)}
              >
                Review &amp; approve
              </button>
              <button
                disabled={!!m.curator_provider}
                onClick={() => setEditing(m)}
              >
                Edit
              </button>
              {deleting === m.id ? (
                <>
                  <span>Remove this note?</span>
                  <button onClick={() => void remove(m.id)}>
                    Delete permanently
                  </button>
                  <button onClick={() => setDeleting(null)}>Cancel</button>
                </>
              ) : (
                <button
                  disabled={!!m.curator_provider}
                  onClick={() => setDeleting(m.id)}
                >
                  Delete
                </button>
              )}
            </div>
          </MemoryCard>
        )}
      </LibraryPage>
      {library.page?.total === 0 && (
        <div className="empty-layer">
          <ShieldCheck size={28} />
          <h3>
            {query || filter !== "all" || restricted
              ? "No matching memories."
              : "A smaller, clearer version of your history."}
          </h3>
          <p>
            {query || filter !== "all" || restricted
              ? "Try a different search, type, or privacy filter."
              : "Save a note from any message in your archive, or write one here. New notes start as drafts."}
          </p>
        </div>
      )}
      {reading && (
        <MemoryInspector
          memory={reading}
          onClose={() => setReading(null)}
          onSource={onSource}
        />
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
