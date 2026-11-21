import ChoiceSelect from "./ChoiceSelect";
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Modal from "./Modal";

export interface Client {
  id: string;
  name: string;
  allow_chats: boolean;
  provider: "chatgpt" | "grok" | "claude" | null;
  can_curate: boolean;
}
type Format = "codex" | "grok" | "claude" | "generic";
type Access = "shared" | "history" | "curate";
interface Setup {
  client: Client;
  configuration: string;
}
interface Connection {
  request_seen: boolean;
  sharing: boolean;
}
export const destinations: Record<
  Format,
  { label: string; provider: string; instructions: string }
> = {
  codex: {
    label: "Codex",
    provider: "ChatGPT",
    instructions:
      "Merge these Pre-Drink sections into your private ~/.codex/config.toml (or CODEX_HOME/config.toml). Keep unrelated settings. Replace older Pre-Drink sections, then restart the client.",
  },
  grok: {
    label: "Grok Build",
    provider: "Grok",
    instructions:
      "Merge these Pre-Drink sections into your private ~/.grok/config.toml (or GROK_HOME/config.toml). Keep unrelated settings. Replace older Pre-Drink sections, then restart Grok Build. This connects the local coding client, not Grok web or Grok Bot.",
  },
  claude: {
    label: "Claude (local MCP)",
    provider: "Claude",
    instructions:
      "Merge this pre-drink entry into your local Claude client's private mcpServers settings. Keep unrelated servers and reload the client. Follow that client's setup instructions; this does not connect Claude web automatically.",
  },
  generic: {
    label: "Other local MCP client",
    provider: "",
    instructions:
      "Merge this pre-drink entry into your client's private mcpServers settings. Keep unrelated servers and reload the connection. The client must support local stdio MCP.",
  },
};
export function scopeDescription(format: Format, access: Access) {
  if (access === "shared" || format === "generic")
    return "Reads eligible shared memories and briefs. No raw history or private agent drafts.";
  return `${destinations[format].provider} history only, excluding private or sensitive material, plus eligible shared memory.${access === "curate" ? " Can revise this provider’s contributions; publication follows your existing sharing rules." : " Cannot write memories."}`;
}
const probe =
  "Call only Pre-Drink's connection_status tool. Report the returned permissions. Do not read archives or memories, write anything, or call another tool.";

export default function AgentSetup({
  sharing,
  onChanged,
  onClose,
}: {
  sharing: boolean;
  onChanged: () => Promise<unknown>;
  onClose: () => void;
}) {
  const [format, setFormat] = useState<Format>("codex");
  const [access, setAccess] = useState<Access>("shared");
  const [name, setName] = useState("");
  const [consent, setConsent] = useState(false);
  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [setup, setSetup] = useState<Setup | null>(null);
  const [connection, setConnection] = useState<Connection | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    heading.current?.focus();
  }, [step]);
  // Any sharing change invalidates the displayed observation until checked again.
  useEffect(() => {
    setConnection(null);
  }, [sharing]);
  async function create(e: React.FormEvent) {
    e.preventDefault();
    if (busy || !consent) return;
    setBusy(true);
    setError("");
    try {
      const result = await invoke<Setup>("create_client", {
        name,
        format,
        allowChats: access !== "shared",
        canCurate: access === "curate",
      });
      setSetup(result);
      setConsent(false);
      setStep(2);
      await onChanged();
    } catch (e) {
      setError(`${String(e)} Check the registered clients before retrying.`);
      await onChanged();
    } finally {
      setBusy(false);
    }
  }
  async function check() {
    if (!setup || busy) return;
    setBusy(true);
    setError("");
    setConnection(null);
    try {
      setConnection(
        await invoke<Connection>("client_connection", { id: setup.client.id }),
      );
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function enable() {
    if (busy) return;
    setBusy(true);
    setError("");
    try {
      await invoke("set_sharing", { enabled: true });
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <Modal label="Connect an agent" busy={busy} onClose={onClose}>
      <div className="setup-progress" aria-label={`Step ${step} of 3`}>
        {[
          [1, "Choose access"],
          [2, "Install settings"],
          [3, "Verify"],
        ].map(([n, label]) => (
          <span key={n} aria-current={step === n ? "step" : undefined}>
            {n}. {label}
          </span>
        ))}
      </div>
      <h2 ref={heading} tabIndex={-1} data-initial-focus>
        {step === 1
          ? "Connect an agent"
          : step === 2
            ? "Install your private settings"
            : "Verify the connection"}
      </h2>
      {step === 1 && (
        <form onSubmit={create}>
          <label>
            Agent application
            <ChoiceSelect
              label="Agent application"
              disabled={busy}
              value={format}
              onChange={(value) => {
                setFormat(value as Format);
                setAccess("shared");
                setConsent(false);
              }}
              options={Object.entries(destinations).map(([value, d]) => ({
                value,
                label: d.label,
              }))}
            />
          </label>
          <label>
            Connection name
            <input
              required
              maxLength={80}
              disabled={busy}
              value={name}
              placeholder="For example, my coding agent"
              onChange={(e) => {
                setName(e.target.value);
                setConsent(false);
              }}
            />
          </label>
          <label>
            Access
            <ChoiceSelect
              label="Access"
              disabled={busy}
              value={access}
              onChange={(value) => {
                setAccess(value as Access);
                setConsent(false);
              }}
              options={[
                { value: "shared", label: "Read shared memory only" },
                ...(format === "generic"
                  ? []
                  : [
                      {
                        value: "history",
                        label: `Also read ${destinations[format].provider} history`,
                      },
                      {
                        value: "curate",
                        label: `Curate ${destinations[format].provider} history and memory`,
                      },
                    ]),
              ]}
            />
          </label>
          <p className="setup-scope">{scopeDescription(format, access)}</p>
          <p className="fine-print">
            Cloud curation is optional. A cloud-powered client may send allowed
            context to its provider and use its subscription or API allowance.
            Never reuse this credential with a different provider.
          </p>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={consent}
              disabled={busy}
              onChange={(e) => setConsent(e.target.checked)}
              required
            />
            I authorize this scope and understand where the client may send
            context.
          </label>
          <div className="form-actions">
            <button
              type="button"
              className="button secondary"
              disabled={busy}
              onClick={onClose}
            >
              Cancel
            </button>
            <button className="button primary" disabled={busy || !consent}>
              {busy ? "Creating…" : "Create private settings"}
            </button>
          </div>
        </form>
      )}
      {step === 2 && setup && (
        <>
          <p>
            <strong>{setup.client.name}</strong> · {destinations[format].label}
          </p>
          <p>{destinations[format].instructions}</p>
          <textarea
            className="setup-config"
            rows={7}
            readOnly
            spellCheck={false}
            aria-label="MCP client configuration"
            value={setup.configuration}
          />
          <p className="fine-print">
            These settings contain a secret token, shown only during setup. Keep
            them outside repositories and chats. Closing hides the token; it
            does not revoke access. Revoke this client if you lose the settings
            or need a different scope.
          </p>
          <div className="form-actions">
            <button
              className="button secondary"
              disabled={busy}
              onClick={onClose}
            >
              Close setup
            </button>
            <button
              className="button primary"
              disabled={busy}
              onClick={() => setStep(3)}
            >
              Continue to verification
            </button>
          </div>
        </>
      )}
      {step === 3 && setup && (
        <>
          <p>
            Keep Pre-Drink open and unlocked. In your configured client, send
            this instruction:
          </p>
          <blockquote className="setup-probe">{probe}</blockquote>
          <p className="fine-print">
            This check returns permissions only—no archive or memory content. It
            does not start curation. Enabling sharing also allows your other
            authorized clients to use their existing grants.
          </p>
          <div className="form-actions">
            {!sharing && (
              <button
                className="button secondary"
                disabled={busy}
                onClick={() => void enable()}
              >
                Enable agent sharing
              </button>
            )}
            <button
              className="button primary"
              disabled={busy || !sharing}
              onClick={() => void check()}
            >
              {busy ? "Checking…" : "Check for client request"}
            </button>
          </div>
          <div role="status" className="setup-status">
            {connection?.request_seen
              ? `A request from this credential succeeded during this unlock. ${sharing && connection.sharing ? "Sharing is on." : "Sharing is off now."} This does not prove a live connection or which model used the credential.`
              : connection
                ? "No successful request seen yet. Check the settings, restart the client, then send the instruction above and check again."
                : "Waiting for verification. Creating settings alone does not establish a connection."}
          </div>
          <div className="form-actions">
            <button
              className="button secondary"
              disabled={busy}
              onClick={() => setStep(2)}
            >
              Back to settings
            </button>
            <button
              className="button secondary"
              disabled={busy}
              onClick={onClose}
            >
              Close setup
            </button>
          </div>
        </>
      )}
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </Modal>
  );
}
