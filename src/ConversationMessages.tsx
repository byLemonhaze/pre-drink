import { useMemo, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Brain, ShieldCheck } from "lucide-react";
import PagedList from "./PagedList";
import LargeMessageReader, { MARKDOWN_LIMIT } from "./LargeMessageReader";
import type { Message } from "./types";

function MessageBody({ content }: { content: string }) {
  const [expanded, setExpanded] = useState(false);
  const long = content.length > 12000;
  return (
    <>
      <div className="markdown">
        {long && !expanded ? (
          <p className="long-message-preview">{content.slice(0, 2000)}…</p>
        ) : content.length > MARKDOWN_LIMIT ? (
          <LargeMessageReader content={content} />
        ) : (
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            skipHtml
            components={{
              img: ({ alt }) => (
                <span className="blocked-image">
                  [Image: {alt || "not loaded"}]
                </span>
              ),
              a: ({ children }) => (
                <span
                  className="inert-link"
                  title="External links stay inactive to keep this vault offline"
                >
                  {children}
                </span>
              ),
            }}
          >
            {content}
          </ReactMarkdown>
        )}
      </div>
      {long && (
        <button
          className="text-button"
          aria-expanded={expanded}
          onClick={() => setExpanded(!expanded)}
        >
          {expanded
            ? "Collapse long message"
            : `Read full message (${content.length.toLocaleString()} characters)`}
        </button>
      )}
    </>
  );
}

export default function ConversationMessages({
  messages,
  allBranches,
  provider = "chatgpt",
  onMemory,
}: {
  messages: Message[];
  allBranches: boolean;
  provider?: string;
  onMemory: (message: Message) => void;
}) {
  const [query, setQuery] = useState("");
  const matching = useMemo(() => {
    const term = query.trim().toLocaleLowerCase();
    return messages.filter(
      (m) =>
        (allBranches || m.is_active) &&
        m.content.toLocaleLowerCase().includes(term),
    );
  }, [messages, allBranches, query]);
  return (
    <>
      <label className="conversation-find">
        Find in this conversation
        <input
          type="search"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search all messages in the selected branches…"
        />
      </label>
      <PagedList
        items={matching}
        scope={JSON.stringify([allBranches, query])}
        label="messages"
        pageSize={20}
      >
        {(m) => (
          <MessageCard
            key={m.id}
            message={m}
            provider={provider}
            onMemory={onMemory}
          />
        )}
      </PagedList>
      {!matching.length && (
        <p className="list-empty">
          No messages match. Try another phrase or include all branches.
        </p>
      )}
    </>
  );
}

export function MessageCard({
  message,
  provider = "chatgpt",
  onMemory,
}: {
  message: Message;
  provider?: string;
  onMemory: (message: Message) => void;
}) {
  return (
    <article className={`message ${message.role}`} key={message.id}>
      <div className="message-heading">
        <span className={`avatar ${message.role}`}>
          {message.role === "user" ? "Y" : "✳"}
        </span>
        <strong>
          {message.role === "user"
            ? "You"
            : provider === "grok"
              ? "Grok"
              : provider === "claude"
                ? "Claude"
                : "ChatGPT"}
        </strong>
        {!message.is_active && (
          <span className="branch-label">Alternate / retained branch</span>
        )}
        {message.sensitive && (
          <ShieldCheck size={14} aria-label="Potentially sensitive message" />
        )}
      </div>
      <MessageBody key={message.content} content={message.content} />
      <button className="save-memory-button" onClick={() => onMemory(message)}>
        <Brain size={13} />
        Keep as memory
      </button>
    </article>
  );
}
