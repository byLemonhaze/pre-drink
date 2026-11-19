import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ShieldCheck } from "lucide-react";
interface Client {
  id: string;
  name: string;
  allow_chats: boolean;
}
export default function Agents({ sharing }: { sharing: boolean }) {
  const [clients, setClients] = useState<Client[]>([]);
  const [name, setName] = useState("");
  const [allowChats, setAllowChats] = useState(false);
  const [consent, setConsent] = useState(false);
  const [config, setConfig] = useState("");
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
    setBusy(true);
    setError("");
    try {
      const result = await invoke("create_client", { name, allowChats });
      setConfig(JSON.stringify(result, null, 2));
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
      setConfig("");
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
            Client name
            <input
              value={name}
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
              onChange={(e) => setAllowChats(e.target.checked)}
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
            <strong>
              Save this configuration in your agent’s MCP settings.
            </strong>
            <p>
              The token grants access to this scope while sharing is on. Treat
              it as a secret; it is shown only now.
            </p>
            <textarea
              rows={13}
              readOnly
              value={config}
              aria-label="MCP client configuration"
              spellCheck={false}
            />
            <button className="button secondary" onClick={() => setConfig("")}>
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
