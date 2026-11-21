import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Curation from "./Curation";
import { ShieldCheck } from "lucide-react";
import AgentSetup, { type Client } from "./AgentSetup";
export default function Agents({ sharing }: { sharing: boolean }) {
  const [clients, setClients] = useState<Client[]>([]);
  const [settingUp, setSettingUp] = useState(false);
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
  async function toggleSharing() {
    try {
      await invoke("set_sharing", { enabled: !sharing });
    } catch (e) {
      setError(String(e));
    }
  }
  async function revoke(id: string) {
    if (busy) return;
    setBusy(true);
    try {
      await invoke("revoke_client", { id });
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
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
        <h2>Connect an agent</h2>
        <p>
          Choose its access, install private settings, then verify a request. No
          model is required for local manual use.
        </p>
        <button className="button primary" onClick={() => setSettingUp(true)}>
          Set up a connection
        </button>
      </section>
      {settingUp && (
        <AgentSetup
          sharing={sharing}
          onChanged={refresh}
          onClose={() => {
            setSettingUp(false);
            void refresh();
          }}
        />
      )}
      <Curation />
      {clients.map((client) => (
        <section className="client-row" key={client.id}>
          <div>
            <strong>{client.name}</strong>
            <p>
              {client.allow_chats
                ? `${client.provider === "grok" ? "Grok" : client.provider === "claude" ? "Claude" : "ChatGPT"} history only${client.can_curate ? " + private curation" : ""}; eligible shared memory`
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
