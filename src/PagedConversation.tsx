import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MessageCard } from "./ConversationMessages";
import type { Message } from "./types";
interface Page {
  messages: Message[];
  total: number;
  offset: number;
  has_alternates: boolean;
}
export default function PagedConversation({
  id,
  provider,
  includeRestricted,
  onMemory,
}: {
  id: string;
  provider: string;
  includeRestricted: boolean;
  onMemory: (message: Message) => void;
}) {
  const [query, setQuery] = useState("");
  const [branches, setBranches] = useState(false);
  const [offset, setOffset] = useState(0);
  const [page, setPage] = useState<Page | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [retry, setRetry] = useState(0);
  const status = useRef<HTMLSpanElement>(null);
  const focusPage = useRef(false);
  const scope = JSON.stringify([
    id,
    includeRestricted,
    branches,
    query,
    offset,
    retry,
  ]);
  const [loadedScope, setLoadedScope] = useState("");
  const current = loadedScope === scope;
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError("");
    const timer = window.setTimeout(
      () => {
        invoke<Page>("get_message_page", {
          id,
          includeRestricted,
          allBranches: branches,
          query,
          offset,
        })
          .then((result) => {
            if (cancelled) return;
            setPage(result);
            setLoadedScope(scope);
            if (focusPage.current) {
              focusPage.current = false;
              requestAnimationFrame(() => {
                if (!cancelled) {
                  status.current?.focus({ preventScroll: true });
                  status.current?.scrollIntoView({ block: "start" });
                }
              });
            }
          })
          .catch((e) => {
            if (!cancelled) {
              setPage(null);
              setError(String(e));
            }
          })
          .finally(() => {
            if (!cancelled) setLoading(false);
          });
      },
      query ? 180 : 0,
    );
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [id, includeRestricted, branches, query, offset, retry, scope]);
  function move(next: number) {
    focusPage.current = true;
    setOffset(next);
  }
  const last = page ? Math.max(0, Math.floor((page.total - 1) / 20) * 20) : 0;
  return (
    <>
      {page?.has_alternates && (
        <label className="branch-toggle">
          <input
            type="checkbox"
            checked={branches}
            onChange={(e) => {
              setBranches(e.target.checked);
              setOffset(0);
              focusPage.current = false;
            }}
          />
          Show all branches &amp; retained history
        </label>
      )}
      <label className="conversation-find">
        Find in this conversation
        <input
          type="search"
          maxLength={200}
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
          value={query}
          placeholder="Search all messages in the selected branches…"
          onChange={(e) => {
            setQuery(e.target.value);
            setOffset(0);
            focusPage.current = false;
          }}
        />
      </label>
      <nav
        className="page-controls"
        aria-label="messages pages"
        aria-busy={!error && (loading || !current)}
      >
        <span
          ref={status}
          tabIndex={-1}
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          {error
            ? "Messages unavailable"
            : loading || !current
              ? "Loading messages…"
              : page
                ? `${page.total ? `${page.offset + 1}–${Math.min(page.offset + 20, page.total)}` : "0"} of ${page.total.toLocaleString()} messages`
                : "Messages unavailable"}
        </span>
        {page && page.total > 20 && (
          <div className="page-buttons">
            <button
              disabled={!current || loading || page.offset === 0}
              onClick={() => move(0)}
            >
              First
            </button>
            <button
              disabled={!current || loading || page.offset === 0}
              onClick={() => move(Math.max(0, page.offset - 20))}
            >
              Previous
            </button>
            <button
              disabled={!current || loading || page.offset >= last}
              onClick={() => move(page.offset + 20)}
            >
              Next
            </button>
            <button
              disabled={!current || loading || page.offset >= last}
              onClick={() => move(last)}
            >
              Last
            </button>
          </div>
        )}
      </nav>
      {error ? (
        <div role="alert" className="list-empty">
          <p>{error}</p>
          <button
            className="button secondary"
            onClick={() => setRetry((n) => n + 1)}
          >
            Retry loading messages
          </button>
        </div>
      ) : (
        current &&
        !loading && (
          <>
            {page?.messages.map((message) => (
              <MessageCard
                key={message.id}
                message={message}
                provider={provider}
                onMemory={onMemory}
              />
            ))}
            {!page?.total && (
              <p className="list-empty">
                No messages match. Try another phrase or include all branches.
              </p>
            )}
          </>
        )
      )}
    </>
  );
}
