import { useEffect, useId, useRef, useState } from "react";

export interface Choice {
  value: string;
  label: string;
}
/** The options live inside the WebView, never in a detached macOS select menu. */
export default function ChoiceSelect({
  label,
  value,
  options,
  onChange,
  disabled = false,
}: {
  label: string;
  value: string;
  options: Choice[];
  onChange: (value: string) => void;
  disabled?: boolean;
}) {
  const id = useId();
  const root = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const typed = useRef({ text: "", time: 0 });
  const selected = options.findIndex((o) => o.value === value);
  const expanded = open && !disabled && options.length > 0;
  const index = Math.min(active, Math.max(0, options.length - 1));
  function close(focus = false) {
    setOpen(false);
    if (focus) trigger.current?.focus();
  }
  function show() {
    if (disabled || !options.length) return;
    typed.current = { text: "", time: 0 };
    setActive(Math.max(0, selected));
    setOpen(true);
  }
  function choose(next: number) {
    const option = options[next];
    if (!option) return;
    close(true);
    onChange(option.value);
  }
  useEffect(() => {
    if (!expanded) return;
    list.current?.focus();
    const dismiss = () => setOpen(false);
    const outside = (e: PointerEvent) => {
      if (e.target instanceof Node && !root.current?.contains(e.target))
        dismiss();
    };
    const visibility = () => {
      if (document.hidden) dismiss();
    };
    window.addEventListener("blur", dismiss);
    window.addEventListener("resize", dismiss);
    document.addEventListener("visibilitychange", visibility);
    document.addEventListener("pointerdown", outside, true);
    return () => {
      window.removeEventListener("blur", dismiss);
      window.removeEventListener("resize", dismiss);
      document.removeEventListener("visibilitychange", visibility);
      document.removeEventListener("pointerdown", outside, true);
    };
  }, [expanded]);
  useEffect(() => {
    if (expanded)
      list.current?.children[index]?.scrollIntoView({ block: "nearest" });
  }, [index, expanded]);
  return (
    <div
      className="choice-select"
      ref={root}
      onBlur={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget)) close();
      }}
    >
      <button
        ref={trigger}
        type="button"
        className="choice-trigger"
        aria-label={`${label}: ${options[selected]?.label ?? "Choose…"}`}
        aria-haspopup="listbox"
        aria-expanded={expanded}
        aria-controls={expanded ? id : undefined}
        disabled={disabled || !options.length}
        onClick={() => (expanded ? close() : show())}
        onKeyDown={(e) => {
          if (["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) {
            e.preventDefault();
            show();
            if (e.key === "Home") setActive(0);
            if (e.key === "End") setActive(options.length - 1);
          }
        }}
      >
        <span>{options[selected]?.label ?? "Choose…"}</span>
        <span aria-hidden="true">⌄</span>
      </button>
      {expanded && (
        <div
          ref={list}
          id={id}
          role="listbox"
          aria-label={label}
          aria-activedescendant={`${id}-${index}`}
          tabIndex={0}
          className="choice-options"
          onKeyDown={(e) => {
            if (e.key === "Escape") {
              e.preventDefault();
              e.stopPropagation();
              close(true);
            } else if (e.key === "Tab") {
              close(true);
            } else if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              choose(index);
            } else if (
              ["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)
            ) {
              e.preventDefault();
              setActive(
                e.key === "Home"
                  ? 0
                  : e.key === "End"
                    ? options.length - 1
                    : Math.max(
                        0,
                        Math.min(
                          options.length - 1,
                          index + (e.key === "ArrowDown" ? 1 : -1),
                        ),
                      ),
              );
            } else if (
              e.key.length === 1 &&
              !e.metaKey &&
              !e.ctrlKey &&
              !e.altKey
            ) {
              e.preventDefault();
              const now = Date.now();
              typed.current = {
                text:
                  (now - typed.current.time > 700 ? "" : typed.current.text) +
                  e.key.toLocaleLowerCase(),
                time: now,
              };
              const match = options.findIndex((o) =>
                o.label.toLocaleLowerCase().startsWith(typed.current.text),
              );
              if (match >= 0) setActive(match);
            }
          }}
        >
          {options.map((option, i) => (
            <div
              id={`${id}-${i}`}
              key={option.value}
              role="option"
              aria-selected={option.value === value}
              className={i === index ? "choice-active" : ""}
              onPointerMove={() => setActive(i)}
              onMouseDown={(e) => e.preventDefault()}
              onClick={(e) => {
                e.preventDefault();
                choose(i);
              }}
            >
              <span aria-hidden="true">
                {option.value === value ? "✓" : ""}
              </span>
              <span>{option.label}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
