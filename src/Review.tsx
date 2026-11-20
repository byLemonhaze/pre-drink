import Modal from "./Modal";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MemoryItem } from "./Memory";

interface Review {
  kind: string;
  id: string;
  title: string;
  content: string;
  sources: MemoryItem[];
  restricted: boolean;
  review_hash: string;
}
export default function ReviewDialog({
  kind,
  id,
  onClose,
  onApproved,
}: {
  kind: "memory" | "brief";
  id: string;
  onClose: () => void;
  onApproved: () => void;
}) {
  const [review, setReview] = useState<Review | null>(null);
  const [checked, setChecked] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  useEffect(() => {
    let active = true;
    invoke<Review>("approval_review", { kind, id })
      .then((r) => {
        if (active) setReview(r);
      })
      .catch((e) => {
        if (active) setError(String(e));
      });
    return () => {
      active = false;
    };
  }, [kind, id]);
  async function approve() {
    if (!review || !checked || busy) return;
    setBusy(true);
    try {
      await invoke("approve_review", {
        kind,
        id,
        reviewHash: review.review_hash,
      });
      onApproved();
      onClose();
    } catch (e) {
      setError(String(e));
      setChecked(false);
    } finally {
      setBusy(false);
    }
  }
  return (
    <Modal label="Review for sharing" wide busy={busy} onClose={onClose}>
      <div className="eyebrow">REVIEW BEFORE SHARING</div>
      <h2>{review?.title || "Loading review…"}</h2>
      {review && (
        <>
          <p className="memory-content">{review.content}</p>
          {review.sources.length > 0 && (
            <>
              <h3>Current supporting memories</h3>
              {review.sources.map((source) => (
                <article className="settings-card" key={source.id}>
                  <strong>{source.kind}</strong>
                  <p className="memory-content">{source.content}</p>
                  <p className="fine-print">
                    {source.approved &&
                    !source.private &&
                    !source.sensitive &&
                    !source.source_restricted
                      ? "Approved"
                      : "Restricted or awaiting approval"}
                  </p>
                </article>
              ))}
            </>
          )}
          {review.restricted ? (
            <p className="form-error">
              This item has private, sensitive, missing, or unapproved content
              or sources. It cannot be approved for sharing.
            </p>
          ) : (
            <label className="checkbox-row">
              <input
                type="checkbox"
                checked={checked}
                onChange={(e) => setChecked(e.target.checked)}
              />
              I reviewed this saved version and its sources and approve it for
              agent sharing.
            </label>
          )}
          <p className="fine-print">
            Edits return it to draft. If anything changes during review,
            approval is rejected.
          </p>
        </>
      )}
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
      <div className="form-actions">
        <button className="button secondary" disabled={busy} onClick={onClose}>
          Close
        </button>
        <button
          className="button primary"
          disabled={!review || review.restricted || !checked || busy}
          onClick={() => void approve()}
        >
          {busy ? "Approving…" : "Approve saved version"}
        </button>
      </div>
    </Modal>
  );
}
