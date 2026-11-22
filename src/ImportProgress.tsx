import { useEffect, useRef, useState } from "react";
import Modal from "./Modal";

export interface ImportState {
  checkpoint: { label: string; saved: number; total: number } | null;
  running: boolean;
  phase: "reading" | "preparing" | "saving" | "paused";
}

export default function ImportProgress({
  state,
  busy,
  onResume,
  onPause,
  onDiscard,
}: {
  state: ImportState | null;
  busy: boolean;
  onResume: (ready: boolean) => void;
  onPause: () => Promise<void>;
  onDiscard: () => Promise<void>;
}) {
  const [confirm, setConfirm] = useState(false);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState("");
  const discardButton = useRef<HTMLButtonElement>(null);
  const resumeButton = useRef<HTMLButtonElement>(null);
  const focusAfterPause = useRef(false);
  const checkpoint = state?.checkpoint;
  const running = busy || !!state?.running;
  useEffect(() => {
    if (!running && checkpoint && focusAfterPause.current) {
      focusAfterPause.current = false;
      resumeButton.current?.focus();
    }
  }, [running, checkpoint]);
  if (!checkpoint && !running) return null;
  const ready = !!checkpoint && checkpoint.saved === checkpoint.total;
  async function pause() {
    focusAfterPause.current = true;
    setWorking(true);
    setError("");
    try {
      await onPause();
    } catch (e) {
      setError(String(e));
    } finally {
      setWorking(false);
    }
  }
  async function discard() {
    setWorking(true);
    setError("");
    try {
      await onDiscard();
      setConfirm(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setWorking(false);
    }
  }
  return (
    <section className="import-checkpoint" aria-label="Import preparation">
      <div>
        <strong role="status">
          {running
            ? state?.phase === "saving"
              ? "Saving your archive…"
              : state?.phase === "preparing"
                ? "Preparing your import…"
                : "Checking your ZIP…"
            : ready
              ? "Import ready to finish"
              : "Import paused"}
        </strong>
        {checkpoint && (
          <>
            <p className="import-filename">{checkpoint.label}</p>
            <progress
              aria-label="Conversations safely prepared"
              max={Math.max(1, checkpoint.total)}
              value={checkpoint.saved}
            />
            <p>
              {checkpoint.saved.toLocaleString()} of{" "}
              {checkpoint.total.toLocaleString()} conversations safely prepared.
            </p>
          </>
        )}
        <p>
          {running
            ? "You can pause at a checkpoint. The final archive save must finish once started."
            : "Preparation is encrypted. Nothing from this import is searchable or shared until it finishes."}
        </p>
      </div>
      <div className="import-checkpoint-actions">
        {running ? (
          <button
            className="button secondary"
            disabled={working || state?.phase === "saving"}
            onClick={() => void pause()}
          >
            {working ? "Pausing…" : "Pause import"}
          </button>
        ) : (
          <>
            <button
              ref={resumeButton}
              className="button primary"
              onClick={() => onResume(ready)}
            >
              {ready ? "Finish import" : "Choose ZIP to resume"}
            </button>
            <button
              ref={discardButton}
              className="button secondary"
              onClick={() => {
                setError("");
                setConfirm(true);
              }}
            >
              Discard preparation…
            </button>
          </>
        )}
      </div>
      {error && !confirm && <p role="alert">{error}</p>}
      {confirm && (
        <Modal
          label="Discard import preparation"
          returnFocus={discardButton}
          busy={working}
          onClose={() => setConfirm(false)}
        >
          <h2>Discard saved preparation?</h2>
          <p>
            Your existing archive and original ZIP stay unchanged. To import
            this ZIP later, preparation starts again from the beginning.
          </p>
          {error && <p role="alert">{error}</p>}
          <div className="editor-actions">
            <button
              data-initial-focus
              className="button secondary"
              disabled={working}
              onClick={() => setConfirm(false)}
            >
              Keep preparation
            </button>
            <button
              className="button secondary"
              disabled={working}
              onClick={() => void discard()}
            >
              Discard preparation
            </button>
          </div>
        </Modal>
      )}
    </section>
  );
}
