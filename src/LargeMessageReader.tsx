import { useRef, useState } from "react";

export const TEXT_PART_SIZE = 16000;
export const MARKDOWN_LIMIT = 48000;

// Bound the rendered text without splitting UTF-16 surrogate pairs. The next
// part uses the same adjusted boundary, so concatenating parts is lossless.
export function textPart(content: string, page: number) {
  function boundary(index: number) {
    if (
      index > 0 &&
      index < content.length &&
      content.charCodeAt(index) >= 0xdc00 &&
      content.charCodeAt(index) <= 0xdfff &&
      content.charCodeAt(index - 1) >= 0xd800 &&
      content.charCodeAt(index - 1) <= 0xdbff
    )
      return index - 1;
    return index;
  }
  return content.slice(
    boundary(page * TEXT_PART_SIZE),
    boundary((page + 1) * TEXT_PART_SIZE),
  );
}

export default function LargeMessageReader({ content }: { content: string }) {
  const [page, setPage] = useState(0);
  const status = useRef<HTMLSpanElement>(null);
  const top = useRef<HTMLDivElement>(null);
  const last = Math.max(0, Math.ceil(content.length / TEXT_PART_SIZE) - 1);
  function move(next: number) {
    setPage(next);
    status.current?.focus({ preventScroll: true });
    top.current?.scrollIntoView({ block: "start" });
  }
  return (
    <div className="large-message-reader" ref={top}>
      <p className="large-message-explanation">
        Very long message — shown in parts as plain text.
      </p>
      <nav className="page-controls" aria-label="Long message parts">
        <span
          ref={status}
          tabIndex={-1}
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          Part {page + 1} of {last + 1}
        </span>
        <div className="page-buttons">
          <button disabled={page === 0} onClick={() => move(0)}>
            First part
          </button>
          <button disabled={page === 0} onClick={() => move(page - 1)}>
            Previous part
          </button>
          <button disabled={page === last} onClick={() => move(page + 1)}>
            Next part
          </button>
          <button disabled={page === last} onClick={() => move(last)}>
            Last part
          </button>
        </div>
      </nav>
      <div className="large-message-text">{textPart(content, page)}</div>
    </div>
  );
}
