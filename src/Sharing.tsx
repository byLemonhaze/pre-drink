import { useState } from "react";
import LibraryPage, { useLibraryPage, type LibraryItem } from "./LibraryPage";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { Copy, Download, Eye } from "lucide-react";

interface Item {
  kind: "memory" | "brief";
  id: string;
}
interface Choice extends Item {
  title: string;
  content: string;
  status: string;
}
interface Preview {
  markdown: string;
  bytes: number;
  included: Item[];
  excluded: { item: Item; reason: string }[];
  review_hash: string;
  session_generation: number;
}
const key = (item: Item) => `${item.kind}:${item.id}`;

export default function Sharing() {
  const [selected, setSelected] = useState<(Item & { title: string })[]>([]);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [reviewed, setReviewed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [query, setQuery] = useState("");
  const library = useLibraryPage<LibraryItem>({
    collection: "all",
    query,
    include_restricted: true,
  });
  function choice(item: LibraryItem): Choice {
    const restricted =
      item.private ||
      item.sensitive ||
      (item.collection === "memory" ? item.source_restricted : item.blocked);
    return {
      kind: item.collection,
      id: item.id,
      title: item.collection === "brief" ? item.title : item.kind,
      content: item.content,
      status: restricted ? "Restricted" : item.approved ? "Approved" : "Draft",
    };
  }
  function toggle(item: Choice) {
    setSelected((items) =>
      items.some((i) => key(i) === key(item))
        ? items.filter((i) => key(i) !== key(item))
        : items.length < 50
          ? [...items, { kind: item.kind, id: item.id, title: item.title }]
          : items,
    );
    setPreview(null);
    setReviewed(false);
    setMessage("");
  }
  async function prepare() {
    setBusy(true);
    setPreview(null);
    setReviewed(false);
    setMessage("");
    try {
      setPreview(
        await invoke<Preview>("prepare_share", {
          selection: { items: selected.map(({ kind, id }) => ({ kind, id })) },
        }),
      );
    } catch (e) {
      setMessage(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function deliver(destination: "clipboard" | "file") {
    if (!preview || !reviewed || !preview.included.length) return;
    setBusy(true);
    setMessage("");
    const request = {
      selection: { items: selected.map(({ kind, id }) => ({ kind, id })) },
      review_hash: preview.review_hash,
      session_generation: preview.session_generation,
    };
    try {
      if (destination === "clipboard") {
        await invoke("copy_share", { request });
        setMessage(
          "Copied the reviewed Markdown. Clipboard history and other apps may retain it.",
        );
      } else {
        const path = await save({
          title: "Save reviewed context as plaintext",
          defaultPath: "Pre-Drink-context.md",
          filters: [{ name: "Markdown", extensions: ["md"] }],
        });
        if (!path) return;
        await invoke("save_share", { request, path });
        setMessage(
          "Saved the reviewed Markdown. This file is outside your encrypted vault.",
        );
      }
    } catch (e) {
      setMessage(String(e));
      setPreview(null);
      setReviewed(false);
    } finally {
      setBusy(false);
    }
  }
  return (
    <div className="secondary-page sharing-page">
      <div className="eyebrow">04 / DELIBERATE SHARING</div>
      <h1>See what leaves your vault.</h1>
      <p className="page-intro">
        Choose memories and briefs. Review the complete text, then decide where
        to put it.
      </p>
      <p className="fine-print">
        This exports only your selection. Agent access is separate: existing MCP
        clients can query all content allowed by their grants. This screen does
        not change those grants or send anything to a provider.
      </p>
      <div className="sharing-layout">
        <section
          className="settings-card share-picker"
          aria-label="Select context"
        >
          <h2>1. Choose context</h2>
          <label>
            Find a memory or brief
            <input
              type="search"
              autoCorrect="off"
              autoCapitalize="off"
              spellCheck={false}
              maxLength={200}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search your saved context"
            />
          </label>
          <p>
            {selected.length} / 50 selected · Selection stays across pages and
            searches.
          </p>
          <div className="share-choices">
            {library.page?.total === 0 && (
              <p>
                {query
                  ? "No matching memories or briefs. Try a different search."
                  : "Create and approve a memory or brief first."}
              </p>
            )}
            <LibraryPage state={library} label="items" disabled={busy}>
              {(item) => {
                const c = choice(item);
                const checked = selected.some((s) => key(s) === key(c));
                return (
                  <label className="share-choice" key={key(c)}>
                    <input
                      type="checkbox"
                      checked={checked}
                      disabled={busy || (!checked && selected.length >= 50)}
                      onChange={() => toggle(c)}
                    />
                    <span>
                      <strong>{c.title}</strong>
                      <small>
                        {c.kind === "brief" ? "Brief" : "Memory"} · {c.status}
                      </small>
                      <span className="share-excerpt">{c.content}</span>
                    </span>
                  </label>
                );
              }}
            </LibraryPage>
          </div>
          <button
            className="button primary"
            disabled={busy || !selected.length}
            onClick={() => void prepare()}
          >
            <Eye size={15} />
            {busy ? "Working…" : "Build exact preview"}
          </button>
        </section>
        <section
          className="settings-card share-output"
          aria-label="Exact outgoing Markdown"
        >
          <h2>2. Review what leaves</h2>
          {!preview ? (
            <p className="share-empty">
              Your complete outgoing Markdown will appear here. Restricted
              content and unapproved drafts are excluded.
            </p>
          ) : (
            <>
              <p>
                {preview.included.length} included · {preview.excluded.length}{" "}
                excluded · {preview.bytes.toLocaleString()} bytes
              </p>
              {preview.excluded.length > 0 && (
                <details className="share-exclusions">
                  <summary>Why some selected items are excluded</summary>
                  <ul>
                    {preview.excluded.map((e) => (
                      <li key={key(e.item)}>
                        {selected.find((c) => key(c) === key(e.item))?.title ||
                          "Removed item"}
                        : {e.reason}
                      </li>
                    ))}
                  </ul>
                  <p>
                    These explanations are visible only here. They are not in
                    your export.
                  </p>
                </details>
              )}
              <textarea
                className="share-markdown"
                aria-label="Full outgoing Markdown"
                readOnly
                spellCheck={false}
                value={preview.markdown}
                placeholder="No eligible content. Approve your notes or resolve their source restrictions first."
              />
              <label className="checkbox-row">
                <input
                  type="checkbox"
                  checked={reviewed}
                  disabled={busy || !preview.included.length}
                  onChange={(e) => setReviewed(e.target.checked)}
                />
                I reviewed this exact text and want to share it.
              </label>
              <p className="fine-print">
                Copy stays on this Mac and is marked confidential for
                cooperating clipboard managers. Copying or saving creates
                plaintext outside vault encryption. A destination app, clipboard
                manager or synced folder may retain it. Review before sending it
                to any AI product. Locking the vault cannot retract a copy.
              </p>
              <div className="form-actions">
                <button
                  className="button secondary"
                  disabled={busy || !reviewed || !preview.included.length}
                  onClick={() => void deliver("clipboard")}
                >
                  <Copy size={15} />
                  Copy Markdown
                </button>
                <button
                  className="button primary"
                  disabled={busy || !reviewed || !preview.included.length}
                  onClick={() => void deliver("file")}
                >
                  <Download size={15} />
                  Save Markdown
                </button>
              </div>
            </>
          )}
        </section>
      </div>
      {message && (
        <p className="share-status" role="status">
          {message}
        </p>
      )}
    </div>
  );
}
