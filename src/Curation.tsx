import ChoiceSelect from "./ChoiceSelect";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Modal from "./Modal";

type Provider = "chatgpt" | "grok" | "claude";
interface Note {
  id: string;
  provider: Provider;
  kind: string;
  content: string;
  origin: string;
  session_label: string;
  revision: number;
  conflict: boolean;
  owner_hold: boolean;
  sensitive: boolean;
  withdrawn: boolean;
  blocked: boolean;
  published: boolean;
  needs_review: boolean;
  publication_basis: string | null;
  sources: { message_id: string; version: number }[];
}
interface Policy {
  provider: Provider;
  publish_archive: boolean;
  publish_session: boolean;
}
interface Review {
  notes: Note[];
  review_hash: string;
}

export default function Curation() {
  const opener = useRef<HTMLElement | null>(null);
  const [provider, setProvider] = useState<Provider>("chatgpt");
  const [after, setAfter] = useState("");
  const [notes, setNotes] = useState<Note[]>([]);
  const [jobs, setJobs] = useState<
    {
      id: string;
      label: string;
      revision: number;
      checkpoint: string;
      status: string;
    }[]
  >([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [policy, setPolicy] = useState<Policy>({
    provider: "chatgpt",
    publish_archive: false,
    publish_session: false,
  });
  const [consent, setConsent] = useState(false);
  const [review, setReview] = useState<Review | null>(null);
  const [editing, setEditing] = useState<Note | null>(null);
  const [history, setHistory] = useState<Note[]>([]);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  useEffect(() => {
    if (!busy && !editing && !review && opener.current) {
      const target = opener.current;
      opener.current = null;
      if (target.isConnected) target.focus({ preventScroll: true });
    }
  }, [busy, editing, review]);
  const reload = async () => {
    setNotes(await invoke<Note[]>("list_curation", { provider, after }));
    setJobs(await invoke("list_curation_jobs", { provider }));
    setSelected([]);
    setReview(null);
  };
  useEffect(() => {
    let current = true;
    setLoading(true);
    setNotes([]);
    setSelected([]);
    setReview(null);
    setEditing(null);
    setHistory([]);
    setConsent(false);
    setNotice("");
    Promise.all([
      invoke<Note[]>("list_curation", { provider, after }),
      invoke<Policy[]>("curation_policies"),
      invoke<typeof jobs>("list_curation_jobs", { provider }),
    ])
      .then(([items, policies, savedJobs]) => {
        if (current) {
          setLoading(false);
          setNotes(items);
          setJobs(savedJobs);
          setPolicy(policies.find((p) => p.provider === provider)!);
        }
      })
      .catch((e) => {
        if (current) {
          setLoading(false);
          setError(String(e));
        }
      });
    return () => {
      current = false;
    };
  }, [provider, after]);
  async function act(work: () => Promise<void>) {
    if (busy || loading) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await work();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section
      className="settings-card curation-panel"
      onClickCapture={(event) => {
        const target = (event.target as Element).closest(
          "button[data-curation-opener]",
        );
        if (target instanceof HTMLElement) opener.current = target;
      }}
    >
      <div className="eyebrow">PRIVATE DRAFTS → SHARED MEMORY</div>
      <h2>Agent curation</h2>
      <p>
        Each curator works with its own provider’s archive and drafts. Other
        agents receive only eligible shared summaries. You can review a batch
        together or enable limited automatic sharing.
      </p>
      <label>
        Private workspace
        <ChoiceSelect
          label="Private workspace"
          value={provider}
          disabled={loading || busy}
          onChange={(value) => {
            setProvider(value as Provider);
            setAfter("");
          }}
          options={[
            { value: "chatgpt", label: "ChatGPT / Codex" },
            { value: "grok", label: "Grok" },
            { value: "claude", label: "Claude" },
          ]}
        />
      </label>
      {jobs.length > 0 && (
        <details>
          <summary>Saved curation progress ({jobs.length})</summary>
          <p>
            The connected agent performs the work. Pausing or closing that agent
            stops processing; these checkpoints stay in the encrypted vault.
          </p>
          {jobs.map((job) => (
            <p key={job.id}>
              <strong>{job.label}</strong> — {job.status},{" "}
              {job.checkpoint || "0"} source messages acknowledged · revision{" "}
              {job.revision}
            </p>
          ))}
        </details>
      )}
      <details>
        <summary>Automatic sharing rules</summary>
        <p>
          Off by default. When enabled, future projects and decisions can enter
          shared memory automatically. Facts, links, and instruction-like text
          still need your review. Sensitive content, stale evidence, and
          unresolved conflicts are withheld. These checks cannot guarantee
          factual accuracy or detect every harmful instruction.
        </p>
        <label className="checkbox-row">
          <input
            type="checkbox"
            disabled={loading || busy}
            checked={policy.publish_archive}
            onChange={(e) => {
              setPolicy({ ...policy, publish_archive: e.target.checked });
              setConsent(false);
            }}
          />
          Automatically share eligible archive summaries
        </label>
        <label className="checkbox-row">
          <input
            type="checkbox"
            disabled={loading || busy}
            checked={policy.publish_session}
            onChange={(e) => {
              setPolicy({ ...policy, publish_session: e.target.checked });
              setConsent(false);
            }}
          />
          Automatically share eligible session summaries
        </label>
        {(policy.publish_archive || policy.publish_session) && (
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={consent}
              disabled={loading || busy}
              onChange={(e) => setConsent(e.target.checked)}
            />
            I allow these future summaries to be shared without reviewing each
            one.
          </label>
        )}
        <button
          className="button secondary"
          disabled={
            busy ||
            ((policy.publish_archive || policy.publish_session) && !consent)
          }
          onClick={() =>
            void act(async () => {
              await invoke("set_curation_policy", {
                provider,
                archive: policy.publish_archive,
                session: policy.publish_session,
              });
              setConsent(false);
              setNotice("Rules saved. Existing drafts were not published.");
            })
          }
        >
          Save sharing rules
        </button>
      </details>
      <div className="curation-actions">
        <button
          className="button secondary"
          disabled={loading || busy}
          onClick={() => void act(reload)}
        >
          Refresh drafts
        </button>
        <button
          className="button secondary"
          disabled={loading || busy || !notes.some((n) => !n.blocked)}
          onClick={() => {
            setSelected(notes.filter((n) => !n.blocked).map((n) => n.id));
            setReview(null);
          }}
        >
          Select eligible on this page
        </button>
        <button
          data-curation-opener
          className="button primary"
          disabled={loading || busy || !selected.length}
          onClick={() =>
            void act(async () => {
              setReview(
                await invoke<Review>("review_curation", { ids: selected }),
              );
            })
          }
        >
          Review selected ({selected.length})
        </button>
      </div>
      {notes.length === 0 && (
        <p>
          No drafts on this page. Set up a curator connection above, then ask it
          to save useful session memory or curate its permitted archive.
        </p>
      )}
      {notes.map((n) => (
        <article className="curation-note" key={n.id}>
          <label className="checkbox-row">
            <input
              type="checkbox"
              disabled={loading || busy || n.blocked}
              checked={selected.includes(n.id)}
              onChange={(e) => {
                setSelected(
                  e.target.checked
                    ? [...selected, n.id]
                    : selected.filter((id) => id !== n.id),
                );
                setReview(null);
              }}
            />
            <strong>
              {n.kind} · revision {n.revision}
            </strong>{" "}
            —{" "}
            {n.published
              ? n.publication_basis === "automatic_policy"
                ? "Shared by your automatic rule"
                : "Shared after your review"
              : n.withdrawn
                ? "Withdrawn"
                : n.blocked
                  ? "Withheld"
                  : "Private draft"}
          </label>
          <p className="curation-content">{n.content}</p>
          <p className="fine-print">
            {n.origin === "archive"
              ? `${n.sources.length} imported source(s)`
              : `Agent-authored session summary: ${n.session_label}`}
            {n.needs_review && !n.published ? " · Requires review" : ""}
            {n.owner_hold
              ? " · Held by you"
              : n.conflict
                ? " · Conflict flagged"
                : ""}
            {n.sensitive ? " · Sensitive content" : ""}
            {n.blocked &&
            !n.sensitive &&
            !n.conflict &&
            !n.owner_hold &&
            !n.withdrawn
              ? " · Source unavailable, restricted, or changed"
              : ""}
          </p>
          <div className="curation-actions">
            <button
              data-curation-opener
              className="button secondary"
              disabled={loading || busy || n.withdrawn}
              onClick={() => {
                setEditing({ ...n, conflict: n.conflict || n.owner_hold });
                setHistory([]);
              }}
            >
              Edit / resolve
            </button>
            <button
              data-curation-opener
              className="button secondary"
              disabled={loading || busy}
              onClick={() =>
                void act(async () => {
                  setHistory(
                    await invoke<Note[]>("curation_history", { id: n.id }),
                  );
                  setEditing({ ...n, conflict: n.conflict || n.owner_hold });
                })
              }
            >
              History / undo
            </button>
            <button
              className="button secondary"
              disabled={loading || busy || n.withdrawn}
              onClick={() =>
                void act(async () => {
                  await invoke("withdraw_curation", {
                    id: n.id,
                    revision: n.revision,
                  });
                  await reload();
                  setNotice(
                    "Contribution withdrawn from sharing. Its history is retained.",
                  );
                })
              }
            >
              Withdraw
            </button>
          </div>
        </article>
      ))}
      <div className="curation-actions">
        {after && (
          <button
            className="button secondary"
            disabled={loading || busy}
            onClick={() => setAfter("")}
          >
            First page
          </button>
        )}
        {notes.length === 30 && (
          <button
            className="button secondary"
            disabled={loading || busy}
            onClick={() => setAfter(notes[notes.length - 1].id)}
          >
            Next page
          </button>
        )}
      </div>
      {editing && (
        <Modal
          label="Edit contribution"
          wide
          busy={busy}
          onClose={() => {
            setEditing(null);
            setHistory([]);
          }}
        >
          <h3>Revise contribution · revision {editing.revision}</h3>
          <p>
            Saving creates a private revision. Existing evidence stays attached.
            Review it again before sharing.
          </p>
          <label className="curation-edit-content">
            Content
            <textarea
              rows={6}
              value={editing.content}
              maxLength={16000}
              disabled={loading || busy || editing.withdrawn}
              onChange={(e) =>
                setEditing({ ...editing, content: e.target.value })
              }
            />
          </label>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={editing.conflict}
              disabled={loading || busy || editing.withdrawn}
              onChange={(e) =>
                setEditing({ ...editing, conflict: e.target.checked })
              }
            />
            Owner hold — keep withheld until I clear it
          </label>
          {history.map((h) => (
            <div className="curation-note" key={h.revision}>
              <strong>Revision {h.revision}</strong>
              <p className="curation-content">{h.content}</p>
              <button
                className="button secondary"
                disabled={loading || busy || editing.withdrawn}
                onClick={() =>
                  setEditing({ ...editing, content: h.content, conflict: true })
                }
              >
                Use this text for a new revision
              </button>
            </div>
          ))}
          <div className="curation-actions">
            <button
              className="button primary"
              disabled={loading || busy || editing.withdrawn}
              onClick={() =>
                void act(async () => {
                  await invoke("edit_curation", {
                    id: editing.id,
                    revision: editing.revision,
                    content: editing.content,
                    conflict: editing.conflict,
                  });
                  setEditing(null);
                  setHistory([]);
                  await reload();
                  setNotice("New private revision saved.");
                })
              }
            >
              Save private revision
            </button>
            <button
              className="button secondary"
              disabled={loading || busy}
              onClick={() => {
                setEditing(null);
                setHistory([]);
              }}
            >
              Close editor
            </button>
          </div>
          {error && (
            <p className="form-error" role="alert">
              {error}
            </p>
          )}
        </Modal>
      )}
      {review && (
        <Modal
          label="Review sharing batch"
          wide
          busy={busy}
          onClose={() => setReview(null)}
        >
          <h3 tabIndex={-1} data-initial-focus>
            Exactly what this batch will share
          </h3>
          <p>
            Only these summaries and their provider, origin, and revision labels
            become available to authorized agents. Source transcripts and
            private workspace notes stay separate.
          </p>
          {review.notes.map((n) => (
            <article className="curation-note" key={n.id}>
              <strong>
                {n.provider} · {n.kind} · revision {n.revision} · {n.origin}
              </strong>
              <p className="curation-content">{n.content}</p>
              {n.blocked && (
                <p className="form-error">
                  Withheld — exclude or resolve this item.
                </p>
              )}
            </article>
          ))}
          <div className="curation-actions">
            <button
              className="button primary"
              disabled={loading || busy || review.notes.some((n) => n.blocked)}
              onClick={() =>
                void act(async () => {
                  await invoke("publish_curation", {
                    ids: review.notes.map((n) => n.id),
                    reviewHash: review.review_hash,
                  });
                  await reload();
                  setNotice(
                    "Reviewed batch shared. Future edits require a fresh release or an eligible automatic sharing rule.",
                  );
                })
              }
            >
              Share this reviewed batch
            </button>
            <button
              className="button secondary"
              disabled={loading || busy}
              onClick={() => setReview(null)}
            >
              Cancel review
            </button>
          </div>
          {error && (
            <p className="form-error" role="alert">
              {error}
            </p>
          )}
        </Modal>
      )}
      {notice && <p role="status">{notice}</p>}
      {error && !editing && !review && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
