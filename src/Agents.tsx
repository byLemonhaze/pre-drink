import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ShieldCheck } from "lucide-react";
interface Client {
  id: string;
  name: string;
  allow_chats: boolean;
}
type ClientFormat = "codex" | "grok" | "generic";
const destinations: Record<
  ClientFormat,
  { label: string; instructions: string }
> = {
  codex: {
    label: "Codex",
    instructions:
      "Add these Pre-Drink sections to your private ~/.codex/config.toml (or your custom CODEX_HOME/config.toml). Keep your other settings and replace any existing Pre-Drink sections. Restart the client to load the change.",
  },
  grok: {
    label: "Grok Build",
    instructions:
      "Add these Pre-Drink sections to your private ~/.grok/config.toml (or your custom GROK_HOME/config.toml). Keep your other settings and replace any existing Pre-Drink sections. Restart Grok Build to load the change. This is for the local coding client, not Grok web or Grok Bot.",
  },
  generic: {
    label: "Other client (JSON)",
    instructions:
      "Merge the pre-drink entry into your client's private mcpServers settings, keeping any other servers. Your client must support local stdio connections. Follow its setup instructions and reload the connection.",
  },
};
export default function Agents({ sharing }: { sharing: boolean }) {
  const [clients, setClients] = useState<Client[]>([]);
  const [name, setName] = useState("");
  const [allowChats, setAllowChats] = useState(false);
  const [consent, setConsent] = useState(false);
  const [format, setFormat] = useState<ClientFormat>("codex");
  const [config, setConfig] = useState<{
    format: ClientFormat;
    text: string;
  } | null>(null);
  const [error, setError] = useState("");
  const [topic, setTopic] = useState("");
  const [pack, setPack] = useState("");
  const [busy, setBusy] = useState(false);
  const refresh = () =>
    invoke<Client[]>("list_clients")
      .then(setClients)
      .catch((e) => setError(String(e)));
  useEffect(() => {
    void refresh();
  }, []);
  async function register(e: React.FormEvent) {
    e.preventDefault();
    if (busy || !consent) return;
    setConfig(null);
    setBusy(true);
    setError("");
    try {
      const text = await invoke<string>("create_client", {
        name,
        allowChats,
        format,
      });
      setConfig({ format, text });
      setName("");
      setConsent(false);
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function toggleSharing() {
    try {
      await invoke("set_sharing", { enabled: !sharing });
    } catch (e) {
      setError(String(e));
    }
  }
  async function revoke(id: string) {
    try {
      await invoke("revoke_client", { id });
      setConfig(null);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }
  async function preview(e: React.FormEvent) {
    e.preventDefault();
    setPack("");
    try {
      setPack(await invoke<string>("preview_context", { topic }));
    } catch (e) {
      setError(String(e));
    }
  }
  return (
    <div className="secondary-page">
      <div className="eyebrow">MCP / CLIENT PERMISSIONS</div>
      <h1>Agent access</h1>
      <p className="page-intro">
        An agent can ask for approved facts, decisions, projects, and curated
        briefs. You control who can connect.
      </p>
      <section className="settings-card">
        <div className="setting-row">
          <div>
            <h2>Agent sharing is {sharing ? "on" : "off"}</h2>
            <p>
              Off after every lock or restart. The Mac app must stay open and
              unlocked.
            </p>
          </div>
          <button
            className={`switch ${sharing ? "on" : ""}`}
            role="switch"
            aria-checked={sharing}
            aria-label="Enable agent sharing"
            onClick={() => void toggleSharing()}
          >
            <span />
          </button>
        </div>
        <p className="sharing-warning">
          <ShieldCheck size={18} />
          The connection to Pre-Drink is local. A cloud-powered agent may send
          the context it receives to its AI provider. Only authorize clients you
          trust. Revocation cannot recall context already received.
        </p>
      </section>
      <section className="settings-card">
        <h2>Authorize a client</h2>
        <form onSubmit={register}>
          <label>
            Agent application
            <select
              value={format}
              disabled={busy}
              onChange={(e) => {
                setFormat(e.target.value as ClientFormat);
                setConsent(false);
              }}
            >
              {Object.entries(destinations).map(([value, destination]) => (
                <option key={value} value={value}>
                  {destination.label}
                </option>
              ))}
            </select>
          </label>
          <label>
            Client name
            <input
              value={name}
              disabled={busy}
              maxLength={80}
              onChange={(e) => setName(e.target.value)}
              placeholder="For example, my coding agent"
              required
            />
          </label>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={allowChats}
              disabled={busy}
              onChange={(e) => {
                setAllowChats(e.target.checked);
                setConsent(false);
              }}
            />
            Also allow searching safe raw conversations and marking them private
          </label>
          <p className="fine-print">
            By default, access is limited to approved memory and briefs. Private
            or sensitive material is always excluded. Agents cannot edit notes,
            approve content, unlock the vault, or remove privacy flags.
          </p>
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={consent}
              disabled={busy}
              onChange={(e) => setConsent(e.target.checked)}
              required
            />
            I authorize this client to receive this scope of context and
            understand its provider may process it.
          </label>
          <button className="button primary" disabled={!consent || busy}>
            {busy ? "Creating…" : "Create client configuration"}
          </button>
        </form>
        {config && (
          <div className="config-box">
            <strong>{destinations[config.format].label} configuration</strong>
            <p>{destinations[config.format].instructions}</p>
            <p>
              The token grants access to this scope while sharing is on. Treat
              it as a secret; it is shown only now. Keep the settings file
              private to your Mac account and outside project repositories.
              Hiding this configuration does not revoke access.
            </p>
            <textarea
              rows={13}
              readOnly
              value={config.text}
              aria-label="MCP client configuration"
              spellCheck={false}
            />
            <button
              className="button secondary"
              onClick={() => setConfig(null)}
            >
              Hide configuration
            </button>
          </div>
        )}
      </section>
      {clients.map((client) => (
        <section className="client-row" key={client.id}>
          <div>
            <strong>{client.name}</strong>
            <p>
              {client.allow_chats
                ? "Approved memory, briefs, and safe chat search"
                : "Approved memory and briefs only"}
            </p>
          </div>
          <button
            className="button secondary"
            disabled={busy}
            onClick={() => void revoke(client.id)}
          >
            Revoke access
          </button>
        </section>
      ))}
      <section className="settings-card">
        <h2>Preview the context</h2>
        <p>
          See what a memory-only client can receive for a topic. Only approved
          and currently eligible content appears.
        </p>
        <form onSubmit={preview}>
          <label>
            Topic
            <input
              value={topic}
              maxLength={200}
              onChange={(e) => setTopic(e.target.value)}
              placeholder="Pre-Drink"
              required
            />
          </label>
          <button className="button secondary">Build context preview</button>
        </form>
        {pack && <pre className="context-preview">{pack}</pre>}
      </section>
      {error && (
        <p className="form-error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
