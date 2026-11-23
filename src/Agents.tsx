import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ChoiceSelect from "./ChoiceSelect";
import type { AgentSessionStatus } from "./types";
import Curation from "./Curation";
import { ShieldCheck } from "lucide-react";
import AgentSetup, { type Client } from "./AgentSetup";
import AgentActivity, { ClientCard, type ActivityPage } from "./AgentActivity";
import Modal from "./Modal";
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
  const [activity, setActivity] = useState<ActivityPage | null>(null);
  const [confirm, setConfirm] = useState<"clear" | "revoke" | "single" | null>(
    null,
  );
  const [acknowledged, setAcknowledged] = useState(false);
  const [reviewedIds, setReviewedIds] = useState<string[]>([]);
  // Ignore reads started before a newer read, control change, or unmount.
  // IPC replies can arrive out of order even when backend calls are serialized.
  const readGeneration = useRef(0);
  const refresh = async () => {
    const request = ++readGeneration.current;
    try {
      const [currentClients, currentActivity] = await Promise.all([
        invoke<Client[]>("list_clients"),
        invoke<ActivityPage>("list_agent_activity", { before: null }),
      ]);
      if (request !== readGeneration.current) return;
      setClients(currentClients);
      setActivity(currentActivity);
    } catch (e) {
      if (request === readGeneration.current) setError(String(e));
    }
  };
  async function pause(client: Client) {
    if (busy) return;
    ++readGeneration.current;
    setBusy(true);
    setError("");
    try {
      await invoke("set_client_paused", {
        id: client.id,
        paused: !client.paused,
      });
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function more() {
    if (busy || !activity?.next_before) return;
    const request = ++readGeneration.current;
    setBusy(true);
    setError("");
    try {
      const next = await invoke<ActivityPage>("list_agent_activity", {
        before: activity.next_before,
      });
      if (request !== readGeneration.current) return;
      if (next.session_generation !== activity.session_generation)
        throw new Error("The vault session changed. Refresh activity.");
      setActivity({ ...next, items: [...activity.items, ...next.items] });
    } catch (e) {
      if (request === readGeneration.current) setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  async function applyConfirmed() {
    if (busy || !acknowledged || !confirm || !activity) return;
    ++readGeneration.current;
    setBusy(true);
    setError("");
    try {
      if (confirm === "clear")
        await invoke("clear_agent_activity", {
          sessionGeneration: activity.session_generation,
        });
      else if (confirm === "single")
        await invoke("revoke_client", { id: reviewedIds[0] });
      else await invoke("revoke_all_clients", { expectedIds: reviewedIds });
      setConfirm(null);
      setAcknowledged(false);
      await refresh();
    } catch (e) {
      setConfirm(null);
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  useEffect(() => {
    void refresh();
    return () => {
      ++readGeneration.current;
    };
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
          Mac ends every session. Agents cannot extend it. Ending sharing
          restores the normal 15-minute idle lock; use Lock vault to lock now. A
          final import save already in progress must finish or roll back before
          locking completes.
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
      <section aria-label="Registered connections">
        <h2>Your connections</h2>
        <p className="fine-print">
          Pause keeps the same credential and permissions. Resume restores that
          access only when global agent sharing is on. Revoke permanently
          invalidates the credential. Already returned context cannot be
          recalled.
        </p>
        {!clients.length && (
          <p>
            No connections yet. Use “Set up a connection” above to choose an
            agent and its permissions.
          </p>
        )}
        {clients.map((client) => (
          <ClientCard
            key={client.id}
            client={client}
            busy={busy}
            onPause={() => void pause(client)}
            onRevoke={() => {
              setReviewedIds([client.id]);
              setAcknowledged(false);
              setConfirm("single");
            }}
          />
        ))}
        {!!clients.length && (
          <button
            className="button secondary"
            disabled={busy}
            onClick={() => {
              setReviewedIds(clients.map((c) => c.id));
              setAcknowledged(false);
              setConfirm("revoke");
            }}
          >
            Revoke all connections…
          </button>
        )}
      </section>
      <AgentActivity
        page={activity}
        busy={busy}
        onRefresh={() => void refresh()}
        onMore={() => void more()}
        onClear={() => {
          setAcknowledged(false);
          setConfirm("clear");
        }}
      />
      {confirm && (
        <Modal
          label={
            confirm === "clear"
              ? "Clear access history"
              : confirm === "single"
                ? "Revoke this connection"
                : "Revoke all connections"
          }
          busy={busy}
          onClose={() => setConfirm(null)}
        >
          <h2 data-initial-focus tabIndex={-1}>
            {confirm === "clear"
              ? "Clear access history?"
              : confirm === "single"
                ? "Revoke this connection?"
                : `Revoke ${reviewedIds.length} connections?`}
          </h2>
          <p>
            {confirm === "clear"
              ? "This permanently clears the current activity record, counts and last-request times in this vault. Connections and their paused states stay unchanged. New requests can create new events. Older encrypted backups retain their history."
              : confirm === "single"
                ? "This credential will stop working permanently. Reconnecting requires new settings. Other connections and agent sharing stay unchanged. Your conversations and memories are kept; returned context cannot be recalled."
                : "All listed credentials will stop working and agent sharing will turn off. Reconnecting requires new settings. Your conversations and memories are kept; returned context cannot be recalled."}
          </p>
          {confirm !== "clear" && (
            <ul>
              {clients
                .filter((c) => reviewedIds.includes(c.id))
                .map((c) => (
                  <li key={c.id}>{c.name}</li>
                ))}
            </ul>
          )}
          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={acknowledged}
              onChange={(e) => setAcknowledged(e.target.checked)}
              disabled={busy}
            />
            I understand and want to continue.
          </label>
          <div className="form-actions">
            <button
              className="button secondary"
              disabled={busy}
              onClick={() => setConfirm(null)}
            >
              Cancel
            </button>
            <button
              className="button primary"
              disabled={busy || !acknowledged}
              onClick={() => void applyConfirmed()}
            >
              {confirm === "clear"
                ? "Clear activity"
                : confirm === "single"
                  ? "Revoke this connection"
                  : "Revoke all & stop sharing"}
            </button>
          </div>
        </Modal>
      )}
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
