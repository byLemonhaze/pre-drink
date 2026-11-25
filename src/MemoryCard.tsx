import type { ReactNode } from "react";
import { FileText, X } from "lucide-react";
import Modal from "./Modal";
import type { MemoryItem } from "./Memory";

export function memoryLabels(memory: MemoryItem) {
  const withheld =
    memory.private || memory.sensitive || memory.source_restricted;
  return {
    review: !memory.approved
      ? "Draft"
      : memory.publication_basis === "automatic_policy"
        ? "Auto-published"
        : "Reviewed",
    access: withheld
      ? "Withheld"
      : memory.approved
        ? "Eligible for sharing"
        : "Not approved for sharing",
    eligible: memory.approved && !withheld,
  };
}

export default function MemoryCard({
  memory,
  onRead,
  children,
}: {
  memory: MemoryItem;
  onRead: () => void;
  children?: ReactNode;
}) {
  const status = memoryLabels(memory);
  return (
    <article className="settings-card memory-card">
      <div className="memory-meta">
        <span>{memory.kind}</span>
        <span className={`approval${memory.approved ? " ready" : ""}`}>
          {status.review}
        </span>
      </div>
      <p className="memory-content memory-preview">{memory.content}</p>
      <div className="memory-provenance">
        <FileText size={13} aria-hidden="true" />
        <span>
          {memory.source_title ||
            (memory.curator_provider
              ? `${memory.curator_provider} contribution`
              : memory.source_message_id
                ? "Linked source"
                : "Standalone note")}
        </span>
      </div>
      <div className="memory-access-row">
        <span className={`access-label${status.eligible ? " eligible" : ""}`}>
          {status.access}
        </span>
        <button className="read-memory" onClick={onRead}>
          Read note <span aria-hidden="true">↗</span>
        </button>
      </div>
      {children}
    </article>
  );
}

export function MemoryInspector({
  memory,
  onClose,
  onSource,
}: {
  memory: MemoryItem;
  onClose: () => void;
  onSource: (id: string, title: string) => void;
}) {
  const status = memoryLabels(memory);
  return (
    <Modal label="Memory details" placement="side" onClose={onClose}>
      <button
        className="modal-close icon-button"
        aria-label="Close memory details"
        data-initial-focus
        onClick={onClose}
      >
        <X size={17} />
      </button>
      <div className="eyebrow">
        MEMORY CABINET / {memory.kind.toUpperCase()}
      </div>
      <h2>A note to carry forward.</h2>
      <p className="inspector-content">{memory.content}</p>
      <dl className="memory-details">
        <div>
          <dt>Review</dt>
          <dd>{status.review}</dd>
        </div>
        <div>
          <dt>Sharing</dt>
          <dd>{status.access}</dd>
        </div>
        {memory.curator_provider && (
          <div>
            <dt>Curator</dt>
            <dd>
              {memory.curator_provider} · revision {memory.curator_revision}
            </dd>
          </div>
        )}
      </dl>
      <p className="fine-print">
        Eligibility is not an access grant. An agent also needs an authorized
        connection, an unlocked vault, and an active sharing session.
      </p>
      {memory.curator_provider && (
        <p className="fine-print">
          Manage this contribution and its revision history in Agent access.
        </p>
      )}
      {memory.source_conversation_id && (
        <button
          className="button secondary inspector-source"
          onClick={() => {
            onClose();
            onSource(memory.source_conversation_id!, memory.source_title || "");
          }}
        >
          <FileText size={15} />
          {memory.source_title || "View source conversation"}
        </button>
      )}
    </Modal>
  );
}
