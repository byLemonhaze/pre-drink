import type { Client } from "./AgentSetup";

export interface Activity {
  id: number;
  client_name: string;
  occurred_at: number;
  operation: string;
  outcome: string;
}
export interface ActivityPage {
  items: Activity[];
  next_before: number | null;
  session_generation: number;
}
export function activityTime(seconds: number | null) {
  return seconds == null
    ? "No recorded request"
    : new Date(seconds * 1000).toLocaleString();
}
const operations: Record<string, string> = {
  connection_status: "Connection check",
  get_facts: "Read facts",
  get_decisions: "Read decisions",
  get_projects: "Read projects",
  get_context_pack: "Build context",
  search_conversations: "Search history",
  mark_private: "Mark a source private",
  curate_memory: "Curation request",
  unknown_tool: "Unsupported tool",
  client_created: "Connection created",
  client_paused: "Connection paused",
  client_resumed: "Connection resumed",
  client_revoked: "Connection revoked",
  revoke_all: "All connections revoked",
};
const outcomes: Record<string, string> = {
  started: "Started — final result not recorded",
  completed: "Processed",
  failed: "Rejected or failed",
  paused: "Blocked — connection paused",
};
export function ClientCard({
  client,
  busy,
  onPause,
  onRevoke,
}: {
  client: Client;
  busy: boolean;
  onPause: () => void;
  onRevoke: () => void;
}) {
  return (
    <section className="client-row">
      <div>
        <strong>{client.name}</strong>
        <p>{client.paused ? "Paused" : "Allowed when agent sharing is on"}</p>
        <p>
          {client.allow_chats
            ? `${client.provider === "grok" ? "Grok" : client.provider === "claude" ? "Claude" : "ChatGPT"} history only${client.can_curate ? " + private curation" : ""}; eligible shared memory`
            : "Approved memory and briefs only"}
        </p>
        <p className="fine-print">
          Last authenticated request: {activityTime(client.last_request_at)}
        </p>
      </div>
      <div className="form-actions">
        <button
          className="button secondary"
          disabled={busy}
          aria-label={`${client.paused ? "Resume" : "Pause"} access for ${client.name}`}
          onClick={onPause}
        >
          {client.paused ? "Resume access" : "Pause access"}
        </button>
        <button
          className="button secondary"
          disabled={busy}
          aria-label={`Revoke access for ${client.name}`}
          onClick={onRevoke}
        >
          Revoke access
        </button>
      </div>
    </section>
  );
}
export default function AgentActivity({
  page,
  busy,
  onRefresh,
  onMore,
  onClear,
}: {
  page: ActivityPage | null;
  busy: boolean;
  onRefresh: () => void;
  onMore: () => void;
  onClear: () => void;
}) {
  return (
    <section className="settings-card" aria-label="Agent access history">
      <h2>Access history</h2>
      <p>
        Latest 1,000 events, encrypted in this vault. Client names, times, tool
        categories and results only. No prompts, replies, source IDs or
        credentials.
      </p>
      <p className="fine-print">
        Processed means the vault handled the request, not that a client
        received or used the response. Paused requests appear after
        authentication. Invalid credentials and attempts while locked or sharing
        is off are not recorded. This is a local activity record, not a
        tamper-proof audit.
      </p>
      <div className="form-actions">
        <button
          className="button secondary"
          disabled={busy}
          onClick={onRefresh}
        >
          Refresh activity
        </button>
        <button
          className="button secondary"
          disabled={busy || !page?.items.length}
          onClick={onClear}
        >
          Clear activity…
        </button>
      </div>
      {page?.items.length ? (
        <ol className="agent-activity-list">
          {page.items.map((event) => (
            <li key={event.id}>
              <strong>{event.client_name}</strong>
              <span>
                {operations[event.operation] ?? "Activity"} ·{" "}
                {outcomes[event.outcome] ?? "Unknown result"}
              </span>
              <time dateTime={new Date(event.occurred_at * 1000).toISOString()}>
                {activityTime(event.occurred_at)}
              </time>
            </li>
          ))}
        </ol>
      ) : (
        <p>{page ? "No activity recorded yet." : "Loading activity…"}</p>
      )}
      {page?.next_before && (
        <button className="button secondary" disabled={busy} onClick={onMore}>
          Older activity
        </button>
      )}
    </section>
  );
}
