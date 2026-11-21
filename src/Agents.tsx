import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ChoiceSelect from "./ChoiceSelect";
import type { AgentSessionStatus } from "./types";
import Curation from "./Curation";
import { ShieldCheck } from "lucide-react";
import AgentSetup, { type Client } from "./AgentSetup";
export default function Agents({
  sharing,
  agentSession,
}: {
  sharing: boolean;
  agentSession: AgentSessionStatus | null;
}) {
  const [duration, setDuration] = useState("one_hour");
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
    if (busy) return;
    setBusy(true);
    try {
      await invoke("set_sharing", { enabled: !sharing });
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function startWorkSession() {
    if (busy) return;
    setBusy(true);
    setError("");
    try {
      await invoke("start_agent_session", { mode: duration });
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
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
            disabled={busy}
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
        <h2>Let your agents work</h2>
        <p>
          Start a work session to enable sharing and keep this vault unlocked,
          even while you are away from the app. Only your registered clients get
          access, with their existing permissions. This does not run a model or
          purchase any usage.
        </p>
        {agentSession ? (
          <>
            <p role="status">
              <strong>
                {agentSession.remaining_seconds == null
                  ? "Active until your Mac locks"
                  : `Active — about ${Math.ceil(agentSession.remaining_seconds / 60)} minutes left`}
              </strong>
            </p>
            <button
              className="button secondary"
              disabled={busy}
              onClick={() => void toggleSharing()}
            >
              End session &amp; stop sharing
            </button>
          </>
        ) : (
          <>
            <ChoiceSelect
              label="Agent work session duration"
              value={duration}
              onChange={setDuration}
              disabled={busy}
              options={[
                { value: "one_hour", label: "1 hour" },
                { value: "four_hours", label: "4 hours" },
                { value: "until_lock", label: "Until my Mac locks" },
              ]}
            />
            <button
              className="button primary"
              disabled={busy}
              onClick={() => void startWorkSession()}
            >
              Start agent work session
            </button>
          </>
        )}
        <p className="fine-print">
          The vault locks when the chosen time ends. Locking or sleeping your
          Mac ends every session immediately. Agents cannot extend it. Ending
          sharing restores the normal 15-minute idle lock; use Lock vault to
          lock immediately.
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
