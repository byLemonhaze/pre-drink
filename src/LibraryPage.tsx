import { useEffect, useRef, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MemoryItem } from "./Memory";
import type { Brief } from "./Briefs";

export type LibraryItem =
  (MemoryItem & { collection: "memory" }) | (Brief & { collection: "brief" });
interface Query {
  collection: "memory" | "brief" | "all";
  query?: string;
  kind?: string | null;
  include_restricted?: boolean;
  eligible_only?: boolean;
  exclude_ids?: string[];
}
interface Page<T> {
  items: T[];
  total: number;
  offset: number;
}
export function useLibraryPage<T>(request: Query) {
  const filter = JSON.stringify(request);
  const [position, setPosition] = useState({ filter, offset: 0 });
  const [revision, setRevision] = useState(0);
  const [result, setResult] = useState<{ scope: string; page: Page<T> } | null>(
    null,
  );
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const offset = position.filter === filter ? position.offset : 0;
  if (position.filter !== filter) setPosition({ filter, offset: 0 });
  const scope = JSON.stringify([filter, offset, revision]);
  const current = result?.scope === scope;
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError("");
    const timer = window.setTimeout(
      () => {
        invoke<Page<T>>("library_page", {
          request: { ...JSON.parse(filter), offset },
        })
          .then((page) => {
            if (!cancelled) setResult({ scope, page });
          })
          .catch((e) => {
            if (!cancelled) {
              setResult(null);
              setError(String(e));
            }
          })
          .finally(() => {
            if (!cancelled) setLoading(false);
          });
      },
      request.query ? 180 : 0,
    );
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [filter, offset, scope, request.query]);
  const page = current && !loading ? result.page : null;
  return {
    items: page?.items || [],
    page,
    error,
    loading: loading || (!current && !error),
    refresh: () => setRevision((n) => n + 1),
    move: (next: number) => setPosition({ filter, offset: next }),
  };
}
export default function LibraryPage<T>({
  state,
  label,
  className,
  disabled = false,
  children,
}: {
  state: ReturnType<typeof useLibraryPage<T>>;
  label: string;
  className?: string;
  disabled?: boolean;
  children: (item: T) => ReactNode;
}) {
  const status = useRef<HTMLSpanElement>(null);
  const pendingFocus = useRef(false);
  useEffect(() => {
    if (!state.loading && pendingFocus.current) {
      pendingFocus.current = false;
      status.current?.focus({ preventScroll: true });
      status.current?.scrollIntoView({ block: "nearest" });
    }
  }, [state.loading]);
  const page = state.page;
  const last = page ? Math.max(0, Math.floor((page.total - 1) / 25) * 25) : 0;
  function move(offset: number) {
    pendingFocus.current = true;
    state.move(offset);
  }
  return (
    <div className="paged-list">
      <nav
        className="page-controls"
        aria-label={`${label} pages`}
        aria-busy={state.loading}
      >
        <span
          ref={status}
          tabIndex={-1}
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          {state.loading
            ? `Loading ${label}…`
            : state.error
              ? `${label} unavailable`
              : `${page?.total ? `${page.offset + 1}–${Math.min(page.offset + 25, page.total)}` : "0"} of ${(page?.total || 0).toLocaleString()} ${label}`}
        </span>
        {page && page.total > 25 && (
          <div className="page-buttons">
            <button
              type="button"
              disabled={disabled || page.offset === 0}
              onClick={() => move(0)}
            >
              First
            </button>
            <button
              type="button"
              disabled={disabled || page.offset === 0}
              onClick={() => move(page.offset - 25)}
            >
              Previous
            </button>
            <button
              type="button"
              disabled={disabled || page.offset === last}
              onClick={() => move(page.offset + 25)}
            >
              Next
            </button>
            <button
              type="button"
              disabled={disabled || page.offset === last}
              onClick={() => move(last)}
            >
              Last
            </button>
          </div>
        )}
      </nav>
      {state.error && (
        <p className="form-error" role="alert">
          {state.error}{" "}
          <button
            type="button"
            className="button secondary"
            onClick={state.refresh}
          >
            Retry
          </button>
        </p>
      )}
      <div className={className}>{state.items.map(children)}</div>
    </div>
  );
}
