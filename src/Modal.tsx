import { useLayoutEffect, useRef, type ReactNode, type RefObject } from "react";

/** Native WebView modality keeps the background inert and contains keyboard focus. */
export default function Modal({
  label,
  wide = false,
  busy = false,
  onClose,
  returnFocus,
  placement = "center",
  children,
}: {
  label: string;
  wide?: boolean;
  busy?: boolean;
  onClose: () => void;
  returnFocus?: RefObject<HTMLElement | null>;
  placement?: "center" | "side";
  children: ReactNode;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  useLayoutEffect(() => {
    const dialog = ref.current!;
    // Pointer clicks do not focus buttons in every macOS WebView. Callers can
    // explicitly name their opener instead of accidentally capturing <body>.
    const previous = returnFocus?.current ?? document.activeElement;
    dialog.showModal();
    dialog.querySelector<HTMLElement>("[data-initial-focus]")?.focus();
    return () => {
      dialog.close();
      if (previous instanceof HTMLElement && previous.isConnected) {
        previous.focus({ preventScroll: true });
      }
    };
  }, [returnFocus]);
  return (
    <dialog
      ref={ref}
      className={`editor-modal${wide ? " wide" : ""}${placement === "side" ? " side-panel" : ""}`}
      aria-label={label}
      aria-modal="true"
      aria-busy={busy}
      onKeyDown={(event) => {
        if (event.key !== "Tab") return;
        // WebKit can briefly move focus to the document after the last control.
        // Wrap explicitly so keyboard users stay on a visible dialog control.
        const controls = Array.from(
          event.currentTarget.querySelectorAll<HTMLElement>(
            "button, input, select, textarea, a[href], [tabindex]",
          ),
        ).filter(
          (element) =>
            !element.matches(":disabled") &&
            element.tabIndex >= 0 &&
            element.getClientRects().length > 0,
        );
        const first = controls[0];
        const last = controls.at(-1);
        if (!first || !last) {
          event.preventDefault();
          return;
        }
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }}
      onCancel={(event) => {
        event.preventDefault();
        if (!busy) onClose();
      }}
    >
      {children}
    </dialog>
  );
}
