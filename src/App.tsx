import PrivacyNotice from "./PrivacyNotice";
import ThemeSwitch from "./ThemeSwitch";
import ImportProgress, { type ImportState } from "./ImportProgress";
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import PagedConversation from "./PagedConversation";
import {
  ArrowDownToLine,
  Check,
  CheckCircle2,
  ChevronRight,
  CircleHelp,
  Database,
  FileArchive,
  FolderOpen,
  GitBranch,
  HardDrive,
  Import,
  Library,
  LoaderCircle,
  LockKeyhole,
  MessageSquare,
  Search,
  Settings2,
  ShieldCheck,
  Brain,
  FileText,
  Plug,
  Share2,
  X,
} from "lucide-react";
import type {
  AgentSessionStatus,
  PrivacyUpdate,
  Conversation,
  ImportReport,
  Message,
  Stats,
} from "./types";

import { MemoryEditor, MemoryView } from "./Memory";
import Briefs from "./Briefs";
import Agents from "./Agents";
import Backup from "./Backup";
import ArchiveRemoval from "./ArchiveRemoval";
import VaultMaintenance from "./VaultMaintenance";
import BuildInfo, { type VaultIdentity } from "./BuildInfo";
import PrivacyRecheck from "./PrivacyRecheck";
import RetainedVaults from "./RetainedVaults";
import Sharing from "./Sharing";
const native = isTauri();
const emptyStats: Stats = {
  conversations: 0,
  messages: 0,
  private: 0,
  sensitive: 0,
  database_path: "",
};
const date = (value: number | null) =>
  value == null
    ? "Undated"
    : new Date(value * 1000).toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
      });
const format = (value: number) => value.toLocaleString();

function App({
  onLock,
  onVaultChanged,
  sharing,
  agentSession,
  privacyUpdate,
  identity,
}: {
  onLock: () => void;
  onVaultChanged: (message: string) => void;
  sharing: boolean;
  agentSession: AgentSessionStatus | null;
  privacyUpdate: PrivacyUpdate;
  identity: VaultIdentity;
}) {
  const [privacyNotice, setPrivacyNotice] = useState("");
  const [page, setPage] = useState<
    | "library"
    | "imports"
    | "settings"
    | "memory"
    | "briefs"
    | "agents"
    | "sharing"
  >("library");
  const [memorySource, setMemorySource] = useState<Message | null>(null);
  const [sourceId, setSourceId] = useState<string | null>(null);
  const [stats, setStats] = useState<Stats>(emptyStats);
  const [query, setQuery] = useState("");
  const [restricted, setRestricted] = useState(false);
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [loading, setLoading] = useState(native);
  const [more, setMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [busy, setBusy] = useState(false);
  const [importState, setImportState] = useState<ImportState | null>(null);
  const importViewActive = useRef(true);
  const activeImportTicket = useRef<string | null>(null);
  const [importPhase, setImportPhase] = useState<ImportState["phase"] | null>(
    null,
  );
  useEffect(() => {
    importViewActive.current = true;
    if (!native) return;
    let disposed = false;
    let stop: (() => void) | undefined;
    void listen<{ ticket: string; phase: ImportState["phase"] }>(
      "import-phase",
      (event) => {
        if (!disposed && activeImportTicket.current === event.payload.ticket)
          setImportPhase(event.payload.phase);
      },
    )
      .then((unlisten) => {
        if (disposed) unlisten();
        else stop = unlisten;
      })
      .catch(() => {});
    return () => {
      disposed = true;
      importViewActive.current = false;
      stop?.();
      activeImportTicket.current = null;
    };
  }, []);
  const importRefreshGeneration = useRef(0);
  const refreshImport = useCallback(async () => {
    if (!native) return;
    const request = ++importRefreshGeneration.current;
    const next = await invoke<ImportState>("import_state");
    if (request === importRefreshGeneration.current) setImportState(next);
  }, []);
  useEffect(() => {
    if (!native) return;
    let disposed = false;
    let pending = false;
    const refresh = async () => {
      if (pending) return;
      pending = true;
      const request = ++importRefreshGeneration.current;
      try {
        const next = await invoke<ImportState>("import_state");
        if (!disposed && request === importRefreshGeneration.current)
          setImportState(next);
      } catch {
        /* Locking removes this view. */
      } finally {
        pending = false;
      }
    };
    void refresh();
    const timer = window.setInterval(() => void refresh(), 1000);
    return () => {
      disposed = true;
      window.clearInterval(timer);
    };
  }, []);
  const [dragging, setDragging] = useState(false);
  const [error, setError] = useState("");
  const [report, setReport] = useState<ImportReport | null>(null);
  const [revision, setRevision] = useState(0);
  const importLock = useRef(false);
  const workspace = useRef<HTMLElement>(null);
  const focusAfterDiscard = useRef(false);
  useEffect(() => {
    if (focusAfterDiscard.current && !importState?.checkpoint) {
      focusAfterDiscard.current = false;
      workspace.current?.focus();
    }
  }, [importState]);
  const listGeneration = useRef(0);
  const selectedConversation = conversations.find((c) => c.id === selected);

  const refreshStats = useCallback(async () => {
    if (native) setStats(await invoke<Stats>("get_stats"));
  }, []);

  useEffect(() => {
    refreshStats().catch((e) => setError(String(e)));
  }, [refreshStats, revision]);

  useEffect(() => {
    if (!native) return;
    const generation = ++listGeneration.current;
    let cancelled = false;
    setLoading(true);
    setLoadingMore(false);
    const timer = window.setTimeout(
      () => {
        invoke<Conversation[]>("list_conversations", {
          query,
          includeRestricted: restricted,
          offset: 0,
          sourceId,
        })
          .then((rows) => {
            if (cancelled || generation !== listGeneration.current) return;
            setConversations(rows);
            setMore(rows.length === 100);
            setSelected((previous) =>
              rows.some((c) => c.id === previous)
                ? previous
                : (rows[0]?.id ?? null),
            );
          })
          .catch((e) => {
            if (!cancelled) {
              setError(String(e));
              setConversations([]);
              setSelected(null);
            }
          })
          .finally(() => {
            if (!cancelled) setLoading(false);
          });
      },
      query ? 180 : 0,
    );
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [query, restricted, revision, sourceId]);

  const importFiles = useCallback(
    async (paths: string[]) => {
      if (importLock.current || !paths.length) return;
      importLock.current = true;
      setBusy(true);
      setDragging(false);
      setError("");
      setReport(null);
      const total: ImportReport = {
        conversations: 0,
        messages: 0,
        new_conversations: 0,
        updated_conversations: 0,
        skipped_messages: 0,
        warnings: [],
      };
      let completed = 0;
      try {
        for (const path of paths) {
          if (!importViewActive.current) break;
          const ticket = crypto.randomUUID();
          activeImportTicket.current = ticket;
          setImportPhase("reading");
          const next = await invoke<ImportReport>("import_zip", {
            path,
            ticket,
          });
          total.conversations += next.conversations;
          total.messages += next.messages;
          total.new_conversations += next.new_conversations;
          total.updated_conversations += next.updated_conversations;
          total.skipped_messages += next.skipped_messages;
          total.warnings.push(...next.warnings);
          completed++;
        }
        setReport(total);
        setPage("library");
        setQuery("");
      } catch (e) {
        if (completed) setReport(total);
        setError(
          String(e).startsWith("Import paused.")
            ? ""
            : `${String(e)}${completed ? ` ${completed} earlier ZIP(s) were imported successfully.` : ""}`,
        );
      } finally {
        setRevision((n) => n + 1);
        setBusy(false);
        importLock.current = false;
        activeImportTicket.current = null;
        setImportPhase(null);
        void refreshImport().catch(() => {});
      }
    },
    [refreshImport],
  );

  useEffect(() => {
    if (!native) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (disposed) return;
        if (event.payload.type === "drop") {
          setDragging(false);
          void importFiles(event.payload.paths);
        } else
          setDragging(
            event.payload.type === "enter" || event.payload.type === "over",
          );
      })
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      })
      .catch((e) => setError(String(e)));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [importFiles]);

  async function pickZip() {
    if (!native || busy) return;
    try {
      const paths = await open({
        multiple: true,
        directory: false,
        title: "Import official chat exports",
        filters: [{ name: "Official export ZIP", extensions: ["zip"] }],
      });
      if (paths) await importFiles(Array.isArray(paths) ? paths : [paths]);
    } catch (e) {
      setError(String(e));
    }
  }

  async function resumeImport(ready: boolean) {
    if (!ready) {
      await pickZip();
      return;
    }
    if (importLock.current) return;
    importLock.current = true;
    setBusy(true);
    setError("");
    setReport(null);
    try {
      const ticket = crypto.randomUUID();
      activeImportTicket.current = ticket;
      setImportPhase("preparing");
      setReport(await invoke<ImportReport>("finish_import", { ticket }));
      setRevision((n) => n + 1);
    } catch (e) {
      setError(String(e));
    } finally {
      importLock.current = false;
      activeImportTicket.current = null;
      setImportPhase(null);
      setBusy(false);
      await refreshImport().catch(() => {});
    }
  }

  function toggleRestricted() {
    // Clear already-rendered restricted content before requesting the safe view.
    listGeneration.current++;
    setConversations([]);
    setSelected(null);
    setRestricted((v) => !v);
  }

  async function togglePrivate() {
    if (!selectedConversation) return;
    try {
      await invoke("mark_private", {
        id: selectedConversation.id,
        private: !selectedConversation.private,
      });
      setPrivacyNotice(
        "Conversation privacy updated. Any linked memories and briefs need fresh approval before sharing.",
      );
      setConversations([]);
      setSelected(null);
      setRevision((n) => n + 1);
    } catch (e) {
      setError(String(e));
    }
  }

  async function loadMore() {
    const generation = listGeneration.current;
    setLoadingMore(true);
    try {
      const rows = await invoke<Conversation[]>("list_conversations", {
        query,
        includeRestricted: restricted,
        offset: conversations.length,
      });
      if (generation !== listGeneration.current) return;
      setConversations((previous) => [...previous, ...rows]);
      setMore(rows.length === 100);
    } catch (e) {
      if (generation === listGeneration.current) setError(String(e));
    } finally {
      if (generation === listGeneration.current) setLoadingMore(false);
    }
  }

  const importButton = (label = "Import export") => (
    <button
      className="button primary"
      onClick={pickZip}
      disabled={busy || !native}
    >
      {busy ? (
        <LoaderCircle className="spin" size={16} />
      ) : (
        <Import size={16} />
      )}
      {busy ? "Importing…" : label}
    </button>
  );

  return (
    <div
      className="app-shell"
      onClickCapture={(event) => {
        // macOS WebKit does not always focus pointer-clicked buttons. Establish
        // the opener before its handler mounts a dialog so Escape can return
        // keyboard users to the control they used, not to the document body.
        const button =
          event.target instanceof Element
            ? event.target.closest("button")
            : null;
        if (button instanceof HTMLButtonElement && !button.disabled)
          button.focus({ preventScroll: true });
      }}
    >
      <button
        className="skip-navigation"
        onClick={() => workspace.current?.focus()}
      >
        Skip to workspace
      </button>
      <aside className="sidebar">
        <div className="brand">
          <img className="app-mark" src="/favicon.svg" alt="" />
          <span>
            pre-drink<span className="brand-dot">.</span>
          </span>
        </div>
        <div className="workspace-label">YOUR PERSONAL VAULT</div>
        <nav aria-label="Main navigation">
          <button
            className={page === "library" ? "nav-item active" : "nav-item"}
            aria-current={page === "library" ? "page" : undefined}
            onClick={() => setPage("library")}
          >
            <Library size={18} />
            Archive
            <span className="nav-count">{format(stats.conversations)}</span>
          </button>
          <button
            className={page === "memory" ? "nav-item active" : "nav-item"}
            aria-current={page === "memory" ? "page" : undefined}
            onClick={() => setPage("memory")}
          >
            <Brain size={18} />
            Memory
          </button>
          <button
            className={page === "briefs" ? "nav-item active" : "nav-item"}
            aria-current={page === "briefs" ? "page" : undefined}
            onClick={() => setPage("briefs")}
          >
            <FileText size={18} />
            Briefs
          </button>
          <button
            className={page === "sharing" ? "nav-item active" : "nav-item"}
            aria-current={page === "sharing" ? "page" : undefined}
            onClick={() => setPage("sharing")}
          >
            <Share2 size={18} />
            Export context
          </button>
          <button
            className={page === "agents" ? "nav-item active" : "nav-item"}
            aria-current={page === "agents" ? "page" : undefined}
            onClick={() => setPage("agents")}
          >
            <Plug size={18} />
            Agent access
          </button>
          <button
            className={page === "imports" ? "nav-item active" : "nav-item"}
            aria-current={page === "imports" ? "page" : undefined}
            onClick={() => setPage("imports")}
          >
            <ArrowDownToLine size={18} />
            Imports
          </button>
          <button
            className={page === "settings" ? "nav-item active" : "nav-item"}
            aria-current={page === "settings" ? "page" : undefined}
            onClick={() => setPage("settings")}
          >
            <Settings2 size={18} />
            Vault settings
          </button>
        </nav>
        <div className="sidebar-bottom">
          <ThemeSwitch />
          <div className="local-label">
            <span className="status-dot" />
            Encrypted locally
          </div>
          <div className="version">
            PRE-DRINK{" "}
            <span>
              {identity.app_version} ·{" "}
              {identity.isolated_storage ? "TEST STORAGE" : "LOCAL VAULT"}
            </span>
          </div>
        </div>
      </aside>

      <main ref={workspace} tabIndex={-1} aria-label="Vault workspace">
        <header className="topbar">
          <div className="breadcrumb">
            Your vault
            <ChevronRight size={14} />
            <span>
              {
                {
                  library: "Archive",
                  imports: "Imports",
                  settings: "Settings",
                  memory: "Memory",
                  briefs: "Briefs",
                  agents: "Agent access",
                  sharing: "Export context",
                }[page]
              }
            </span>
          </div>
          <div className="offline-badge">
            <ShieldCheck size={14} />
            {agentSession
              ? "Agent work session active"
              : sharing
                ? "Agent sharing on"
                : "Agent sharing off"}
          </div>
          <button className="button secondary lock-button" onClick={onLock}>
            <LockKeyhole size={14} />
            Lock vault
          </button>
        </header>
        <PrivacyNotice update={privacyUpdate} />
        <ImportProgress
          state={
            importPhase
              ? {
                  checkpoint: importState?.checkpoint ?? null,
                  running: true,
                  phase: importPhase,
                }
              : importState
          }
          busy={busy}
          onResume={(ready) => void resumeImport(ready)}
          onPause={async () => {
            await invoke("pause_import");
            await refreshImport();
          }}
          onDiscard={async () => {
            await invoke("discard_import");
            focusAfterDiscard.current = true;
            await refreshImport();
            setError("");
          }}
        />
        {!native && (
          <div className="preview-banner">
            <CircleHelp size={16} />
            Browser preview — open the Mac app to import and store
            conversations.
          </div>
        )}
        {error && (
          <div className="notice error" role="alert">
            <span>{error}</span>
            <button aria-label="Dismiss error" onClick={() => setError("")}>
              <X size={16} />
            </button>
          </div>
        )}
        {privacyNotice && (
          <div className="notice success" role="status">
            <span>{privacyNotice}</span>
            <button
              aria-label="Dismiss privacy update"
              onClick={() => setPrivacyNotice("")}
            >
              <X size={16} />
            </button>
          </div>
        )}
        {report && (
          <div className="notice success" role="status">
            <CheckCircle2 size={18} />
            <div>
              <strong>
                {report.new_conversations} new · {report.updated_conversations}{" "}
                refreshed
              </strong>
              <span>
                {" "}
                {format(report.messages)} text messages processed. Saved on this
                Mac.
              </span>
              <div className="form-actions">
                <button
                  className="button secondary"
                  onClick={() => setPage("memory")}
                >
                  Next: create a memory
                </button>
                <button
                  className="button secondary"
                  onClick={() => setPage("agents")}
                >
                  Connect a curator
                </button>
              </div>
              {report.warnings.length > 0 && (
                <details>
                  <summary>Import notes</summary>
                  {[...new Set(report.warnings)].map((w) => (
                    <p key={w}>{w}</p>
                  ))}
                </details>
              )}
            </div>
            <button
              aria-label="Dismiss import report"
              onClick={() => setReport(null)}
            >
              <X size={16} />
            </button>
          </div>
        )}

        {page === "memory" && (
          <MemoryView
            onSource={(id) => {
              setSourceId(id);
              setQuery("");
              setPage("library");
              setRevision((n) => n + 1);
            }}
          />
        )}
        {page === "briefs" && <Briefs />}
        {page === "sharing" && <Sharing />}
        {page === "agents" && (
          <Agents sharing={sharing} agentSession={agentSession} />
        )}
        {page === "library" && (
          <>
            <section className="page-heading">
              <div>
                <div className="eyebrow">01 / SOURCE MATERIAL</div>
                <h1>
                  Archive<span className="heading-dot">.</span>
                </h1>
                <p>Your AI history, sober and ready.</p>
              </div>
              {importButton()}
            </section>
            <div className="library-toolbar">
              <label className="search">
                <Search size={18} />
                <input
                  aria-label="Search conversations"
                  autoCorrect="off"
                  autoCapitalize="off"
                  spellCheck={false}
                  placeholder="Search your conversations…"
                  value={query}
                  maxLength={1000}
                  onChange={(e) => {
                    setSourceId(null);
                    setQuery(e.target.value);
                  }}
                />
                {query && (
                  <button
                    aria-label="Clear search"
                    onClick={() => setQuery("")}
                  >
                    <X size={15} />
                  </button>
                )}
              </label>
              <label className="privacy-toggle">
                <input
                  type="checkbox"
                  checked={restricted}
                  onChange={toggleRestricted}
                />
                <ShieldCheck size={16} />
                <span>Show private & sensitive</span>
              </label>
            </div>
            {sourceId && (
              <div className="source-navigation">
                <span>
                  Viewing a linked source. Privacy filters still apply.
                </span>
                <button
                  className="text-button"
                  onClick={() => setSourceId(null)}
                >
                  Show all conversations
                </button>
              </div>
            )}
            {stats.conversations === 0 && !loading && !query ? (
              <section className="onboarding">
                <div className="onboarding-copy">
                  <div className="small-pill">
                    <span className="status-dot" />A FRESH START FOR YOUR
                    HISTORY
                  </div>
                  <h2>
                    Your history,
                    <br />
                    <span>under your control.</span>
                  </h2>
                  <p>
                    Bring your scattered chats home. One quiet place to find the
                    ideas, answers, and rabbit holes you want to come back to.
                  </p>
                  <div className="hero-action">
                    {importButton("Import chat export")}
                    <span>ChatGPT · Claude · Grok ZIPs</span>
                  </div>
                  <div className="privacy-note">
                    <LockKeyhole size={14} />
                    Your history stays on your machine.
                  </div>
                </div>
                <button
                  className="drop-card"
                  onClick={pickZip}
                  disabled={!native || busy}
                  aria-label="Choose an official chat export ZIP"
                >
                  <div className="archive-illustration">
                    <div className="back-sheet sheet-two" />
                    <div className="back-sheet sheet-one" />
                    <div className="front-sheet">
                      <div className="sheet-header">
                        <span>✳</span>
                        <span>YOUR CHATS</span>
                      </div>
                      <div className="sheet-line wide" />
                      <div className="sheet-line" />
                      <div className="sheet-line medium" />
                      <div className="sheet-divider" />
                      <div className="sheet-line wide" />
                      <div className="sheet-line medium" />
                      <div className="sheet-stamp">
                        <Check size={20} />
                      </div>
                    </div>
                    <div className="zip-label">.ZIP</div>
                  </div>
                  <strong>Drop your export here</strong>
                  <span>or choose a file from your Mac</span>
                  <div className="file-chip">
                    <FileArchive size={13} /> Official export · .zip
                  </div>
                </button>
                <div className="how-it-works">
                  <div>
                    <span className="step-number">01</span>
                    <div>
                      <strong>Export your history</strong>
                      <p>Use an official ChatGPT, Claude or Grok export.</p>
                    </div>
                  </div>
                  <div>
                    <span className="step-number">02</span>
                    <div>
                      <strong>Drop it in</strong>
                      <p>We organize the conversations locally.</p>
                    </div>
                  </div>
                  <div>
                    <span className="step-number">03</span>
                    <div>
                      <strong>Find your next thought</strong>
                      <p>Search across everything you kept.</p>
                    </div>
                  </div>
                </div>
              </section>
            ) : (
              <section className="vault-columns">
                <div className="conversation-pane">
                  <div className="pane-label">
                    <span>{query ? "SEARCH RESULTS" : "CONVERSATIONS"}</span>
                    <span>
                      {loading
                        ? "…"
                        : `${conversations.length}${more ? "+" : ""}`}
                    </span>
                  </div>
                  <div className="conversation-list">
                    {loading ? (
                      <div className="list-empty">
                        <LoaderCircle className="spin" size={20} />
                        Searching your vault…
                      </div>
                    ) : conversations.length === 0 ? (
                      <div className="list-empty">
                        <Search size={24} />
                        <strong>
                          {query
                            ? "No matching conversations"
                            : "No visible conversations"}
                        </strong>
                        <p>
                          {restricted
                            ? "Try a different search or provider filter."
                            : "Private and sensitive conversations are hidden. Reviewing them here does not make them available to agents."}
                        </p>
                        {!restricted && (
                          <button
                            className="button secondary"
                            onClick={() => setRestricted(true)}
                          >
                            Show restricted conversations
                          </button>
                        )}
                      </div>
                    ) : (
                      conversations.map((c) => (
                        <button
                          key={c.id}
                          className={`conversation-card ${selected === c.id ? "selected" : ""}`}
                          onClick={() => setSelected(c.id)}
                        >
                          <div className="conversation-meta">
                            <span>
                              {c.platform === "grok"
                                ? "Grok"
                                : c.platform === "claude"
                                  ? "Claude"
                                  : "ChatGPT"}
                            </span>
                            <time>{date(c.updated_at)}</time>
                          </div>
                          <h3>{c.title}</h3>
                          <p>{c.preview}</p>
                          <div className="card-footer">
                            <span>{c.message_count} messages</span>
                            {c.private && (
                              <LockKeyhole size={13} aria-label="Private" />
                            )}
                            {c.sensitive && (
                              <span
                                className="sensitive-dot"
                                title="Potentially sensitive"
                              />
                            )}
                          </div>
                        </button>
                      ))
                    )}
                    {more && !loading && (
                      <button
                        className="load-more"
                        onClick={loadMore}
                        disabled={loadingMore}
                      >
                        {loadingMore ? "Loading…" : "Load more conversations"}
                      </button>
                    )}
                  </div>
                </div>
                <div className="reader" key={selected ?? "empty"}>
                  {selectedConversation && !loading ? (
                    <>
                      <div className="reader-header">
                        <div>
                          <span className="eyebrow">
                            {selectedConversation.platform.toUpperCase()} ·{" "}
                            {date(selectedConversation.created_at)}
                          </span>
                          <h2>{selectedConversation.title}</h2>
                        </div>
                        <div className="reader-actions">
                          <button
                            className={`icon-button ${selectedConversation.private ? "is-private" : ""}`}
                            title={
                              selectedConversation.private
                                ? "Make conversation non-private"
                                : "Mark conversation private"
                            }
                            aria-label={
                              selectedConversation.private
                                ? "Make conversation non-private"
                                : "Mark conversation private"
                            }
                            onClick={togglePrivate}
                          >
                            <LockKeyhole size={18} />
                          </button>
                          <ArchiveRemoval
                            compact
                            key={selectedConversation.id}
                            conversationId={selectedConversation.id}
                            onChanged={() => {
                              setSelected(null);
                              setRevision((n) => n + 1);
                            }}
                          />
                        </div>
                      </div>
                      {(selectedConversation.private ||
                        selectedConversation.sensitive) && (
                        <div className="reader-warning">
                          <ShieldCheck size={15} />
                          {selectedConversation.private
                            ? "Private conversation"
                            : "Potentially sensitive content"}{" "}
                          · visible because you enabled review.
                        </div>
                      )}
                      <div className="messages">
                        <PagedConversation
                          key={JSON.stringify([selected, restricted, revision])}
                          id={selectedConversation.id}
                          provider={selectedConversation.platform}
                          includeRestricted={restricted}
                          onMemory={setMemorySource}
                        />
                      </div>
                    </>
                  ) : (
                    <div className="reader-placeholder">
                      <MessageSquare size={32} strokeWidth={1.2} />
                      <h3>A thought worth revisiting.</h3>
                      <p>Choose a conversation to settle back in.</p>
                    </div>
                  )}
                </div>
              </section>
            )}
            <footer className="library-footer">
              <span>
                <HardDrive size={13} />
                {format(stats.conversations)} conversations ·{" "}
                {format(stats.messages)} messages
              </span>
              <span>
                {stats.private + stats.sensitive > 0
                  ? `${stats.private} private · ${stats.sensitive} flagged sensitive`
                  : "A private space for a clearer head."}
              </span>
            </footer>
          </>
        )}

        {page === "imports" && (
          <div className="secondary-page">
            <div className="eyebrow">IMPORT / OFFICIAL EXPORTS</div>
            <h1>Import history</h1>
            <p className="page-intro">
              Import an official ChatGPT, Claude or Grok export. Everything is
              read and stored on this Mac.
            </p>
            <section className="settings-card import-guide">
              <div className="section-icon">
                <FileArchive size={25} />
              </div>
              <h2>Your first import, in three steps</h2>
              <ol>
                <li>
                  <strong>Request your account export</strong>
                  <p>
                    ChatGPT: Settings → Data Controls → Export data. Claude:
                    Settings → Privacy → Export data. Grok: Settings → Data
                    Controls → download your data.
                  </p>
                </li>
                <li>
                  <strong>Download the official ZIP</strong>
                  <p>
                    Keep the ZIP intact. Pre-Drink identifies the supported
                    conversation files inside it; no extraction is needed.
                  </p>
                </li>
                <li>
                  <strong>Bring it into your vault</strong>
                  <p>
                    Choose the ZIP below, or drop it anywhere in this window.
                  </p>
                </li>
              </ol>
              {importButton("Choose an export ZIP")}
            </section>
            <div className="info-grid">
              <section className="settings-card">
                <GitBranch size={21} />
                <h3>Every text branch has a place.</h3>
                <p>
                  Alternate replies are preserved. Repeat imports merge existing
                  conversations and keep your privacy choices.
                </p>
              </section>
              <section className="settings-card">
                <FolderOpen size={21} />
                <h3>Your original stays yours.</h3>
                <p>
                  We never modify the ZIP. Text and original conversation JSON
                  are saved locally; attachment files are not copied.
                </p>
              </section>
            </div>
            <p className="fine-print">
              Supports ChatGPT/Claude conversations JSON and Grok account
              exports containing prod-grok-backend.json, up to 256 MB of
              conversation JSON. Media and reasoning are not separately
              imported. Unknown formats fail without a partial import.
            </p>
          </div>
        )}

        {page === "settings" && (
          <div className="secondary-page">
            <div className="eyebrow">SECURITY / LOCAL STORAGE</div>
            <h1>Vault settings</h1>
            <p className="page-intro">
              No account to create. No server to trust. Your conversations live
              here.
            </p>
            <section className="settings-card">
              <div className="card-heading">
                <ShieldCheck size={22} />
                <h2>Privacy & visibility</h2>
              </div>
              <div className="setting-row">
                <div>
                  <strong>Show private & sensitive conversations</strong>
                  <p>
                    Review restricted content in this session. Off by default
                    each time you open the app.
                  </p>
                </div>
                <button
                  className={`switch ${restricted ? "on" : ""}`}
                  role="switch"
                  aria-checked={restricted}
                  aria-label="Show private and sensitive conversations"
                  onClick={toggleRestricted}
                >
                  <span />
                </button>
              </div>
              <p className="fine-print">
                Basic rules flag email addresses, phone-like numbers, street
                addresses, and some medical, financial, or credential terms.
                They can miss sensitive details or flag harmless text. You can
                also mark any conversation private.
              </p>
            </section>
            <section className="settings-card">
              <div className="card-heading">
                <Database size={22} />
                <h2>Local storage</h2>
              </div>
              <div className="storage-stats">
                <div>
                  <strong>{format(stats.conversations)}</strong>
                  <span>conversations</span>
                </div>
                <div>
                  <strong>{format(stats.messages)}</strong>
                  <span>messages</span>
                </div>
                <div>
                  <strong>{format(stats.private)}</strong>
                  <span>marked private</span>
                </div>
              </div>
              <label className="path-label">VAULT LOCATION</label>
              <code className="database-path">
                {stats.database_path || "Available in the Mac app"}
              </code>
              <p className="fine-print">
                Archive, memory, briefs, and search index are encrypted with
                SQLCipher AES-256 and authenticated pages. Argon2id derives the
                key from your password. No password or database key is saved.
                Normally locks after 15 minutes without activity in this app. An
                explicit agent work session keeps it unlocked for your chosen
                period. Mac lock, system/display sleep, and user-session
                switching still lock the vault. Original export ZIPs and old Mac
                backups are outside this protection.
              </p>
            </section>
            <PrivacyRecheck onApplied={() => setRevision((r) => r + 1)} />
            <section className="settings-card">
              <h2>App &amp; vault identity</h2>
              <BuildInfo identity={identity} />
            </section>
            <Backup />
            <RetainedVaults />
            <section className="settings-card">
              <h2>Archive snapshot retention</h2>
              <p>
                Keep the newest imported raw snapshot for each conversation.
                Review and remove older raw snapshots explicitly. Retained
                message branches stay searchable.
              </p>
              <ArchiveRemoval onChanged={() => setRevision((n) => n + 1)} />
            </section>
            <VaultMaintenance onChanged={onVaultChanged} />
            <section className="settings-card">
              <h2>Security boundaries</h2>
              <p>
                Encryption protects stored files, not a compromised Mac or
                content already received by an agent. Privacy detection is a
                basic safety filter, not a guarantee. Use a long unique
                password, FileVault, and a locked Mac account. Original exports
                remain unencrypted unless you protect them separately.
              </p>
            </section>
          </div>
        )}
      </main>
      {memorySource && (
        <MemoryEditor
          source={memorySource}
          onClose={() => setMemorySource(null)}
          onSaved={() => setPage("memory")}
        />
      )}
      {dragging && (
        <div className="drop-overlay">
          <FileArchive size={48} />
          <h2>Drop official export</h2>
          <p>Drop your official ChatGPT, Claude or Grok export ZIP.</p>
        </div>
      )}
    </div>
  );
}

export default App;
