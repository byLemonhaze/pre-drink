import { useRef, useState, type ReactNode } from "react";

// Bound mounted cards/Markdown trees. Filtering happens before pagination so
// matches beyond the current page remain discoverable. This is render paging;
// the caller still owns the complete local result set.
export default function PagedList<T>({
  items,
  scope,
  label,
  pageSize = 25,
  className,
  children,
}: {
  items: T[];
  scope: string;
  label: string;
  pageSize?: number;
  className?: string;
  children: (item: T) => ReactNode;
}) {
  const [position, setPosition] = useState({ scope, page: 0 });
  const top = useRef<HTMLDivElement>(null);
  const status = useRef<HTMLSpanElement>(null);
  const last = Math.max(0, Math.ceil(items.length / pageSize) - 1);
  const page = position.scope === scope ? Math.min(position.page, last) : 0;
  // Reset during render, so returning to an earlier filter cannot resurrect
  // its old page and briefly display the wrong slice.
  if (position.scope !== scope || position.page !== page) {
    setPosition({ scope, page });
  }
  const start = page * pageSize;
  function move(next: number) {
    setPosition({ scope, page: next });
    status.current?.focus({ preventScroll: true });
    top.current?.scrollIntoView({ block: "start" });
  }
  return (
    <div ref={top} className="paged-list">
      <nav className="page-controls" aria-label={`${label} pages`}>
        <span
          ref={status}
          tabIndex={-1}
          role="status"
          aria-live="polite"
          aria-atomic="true"
        >
          {items.length
            ? `${start + 1}–${Math.min(start + pageSize, items.length)}`
            : "0"}
          {` of ${items.length.toLocaleString()} ${label}`}
        </span>
        {last > 0 && (
          <div className="page-buttons">
            <button disabled={page === 0} onClick={() => move(0)}>
              First
            </button>
            <button disabled={page === 0} onClick={() => move(page - 1)}>
              Previous
            </button>
            <button disabled={page === last} onClick={() => move(page + 1)}>
              Next
            </button>
            <button disabled={page === last} onClick={() => move(last)}>
              Last
            </button>
          </div>
        )}
      </nav>
      <div className={className}>
        {items.slice(start, start + pageSize).map(children)}
      </div>
    </div>
  );
}
