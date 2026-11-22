import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import test from "node:test";
import ts from "typescript";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

// Render the actual components with React's server renderer. TypeScript is an
// existing build dependency; no DOM substitute or extra test runtime is needed.
const cache = new Map();
function component(name) {
  if (cache.has(name)) return cache.get(name);
  const path = fileURLToPath(new URL(`../src/${name}.tsx`, import.meta.url));
  const source = ts.transpileModule(readFileSync(path, "utf8"), {
    compilerOptions: {
      module: ts.ModuleKind.CommonJS,
      jsx: ts.JsxEmit.ReactJSX,
      esModuleInterop: true,
    },
  }).outputText;
  const module = { exports: {} };
  const require = createRequire(path);
  const imports = (id) => {
    if (id === "@tauri-apps/api/core")
      return {
        invoke: () => {
          throw new Error("Rendering must not issue native commands");
        },
      };
    if (id === "react-markdown") return ReactMarkdown;
    if (id === "remark-gfm") return remarkGfm;
    return id.startsWith("./") ? component(id.slice(2)) : require(id);
  };
  vm.runInThisContext(`(function(module, exports, require) {${source}\n})`, {
    filename: path,
  })(module, module.exports, imports);
  cache.set(name, module.exports);
  return module.exports;
}
const Reader = component("ConversationMessages").default;
const PagedList = component("PagedList").default;
const message = (id, content, is_active = true) => ({
  id,
  content,
  is_active,
  role: "user",
  sensitive: false,
});
const render = (messages, allBranches = false) =>
  renderToStaticMarkup(
    React.createElement(Reader, { messages, allBranches, onMemory() {} }),
  );

test("long conversations mount one bounded page without discarding the result count", () => {
  const html = render(
    Array.from({ length: 2000 }, (_, i) =>
      message(String(i), `garden-entry-${i}`),
    ),
  );
  assert.equal((html.match(/<article /g) || []).length, 20);
  assert.match(html, /2,000 messages/);
  assert.doesNotMatch(html, /garden-entry-1999/);
});

test("retained branches require the explicit local viewer toggle", () => {
  const messages = [
    message("a", "active-garden"),
    message("b", "alternate-garden", false),
  ];
  assert.doesNotMatch(render(messages), /alternate-garden/);
  assert.match(render(messages, true), /alternate-garden/);
});

test("the extracted Markdown reader keeps remote media, links and HTML inert", () => {
  const html = render([
    message(
      "a",
      '![orchid](https://example.invalid/pixel.png) [visit](https://example.invalid/) <img src="https://example.invalid/html.png">',
    ),
  ]);
  assert.doesNotMatch(html, /<img\b|<a\b|src=|href=/);
  assert.match(html, /\[Image: orchid\]/);
});

test("very long messages show an explicit preview before expensive full rendering", () => {
  const html = render([message("a", "garden ".repeat(3000) + "tail-orchid")]);
  assert.match(html, /Read full message/);
  assert.doesNotMatch(html, /tail-orchid/);
  assert.ok(html.length < 6000);
});

test("large curation libraries bound mounted cards and describe empty results", () => {
  const props = {
    scope: "all",
    label: "memories",
    children: (id) => React.createElement("article", { key: id }, id),
  };
  const html = renderToStaticMarkup(
    React.createElement(PagedList, {
      ...props,
      items: Array.from({ length: 3000 }, (_, i) => i),
    }),
  );
  assert.equal((html.match(/<article>/g) || []).length, 25);
  assert.match(html, /3,000 memories/);
  const empty = renderToStaticMarkup(
    React.createElement(PagedList, { ...props, items: [] }),
  );
  assert.match(empty, /0 of 0 memories/);
  assert.doesNotMatch(empty, /<article>/);
});

test("assistant labels follow the imported provider", () => {
  for (const [provider, label] of [
    ["chatgpt", "ChatGPT"],
    ["grok", "Grok"],
    ["claude", "Claude"],
  ]) {
    const html = renderToStaticMarkup(
      React.createElement(Reader, {
        messages: [
          {
            ...message("answer", "Fictional garden answer."),
            role: "assistant",
          },
        ],
        provider,
        allBranches: false,
        onMemory() {},
      }),
    );
    assert.ok(html.includes(`<strong>${label}</strong>`));
    if (provider !== "chatgpt")
      assert.ok(!html.includes("<strong>ChatGPT</strong>"));
  }
});

test("huge expanded messages render bounded inert text, with lossless Unicode parts", () => {
  const {
    default: LargeReader,
    textPart,
    TEXT_PART_SIZE,
  } = component("LargeMessageReader");
  const text =
    "x".repeat(TEXT_PART_SIZE - 1) +
    "🌿" +
    "<script>fictional()</script>\n".repeat(20000) +
    "tail-orchid";
  const count = Math.ceil(text.length / TEXT_PART_SIZE);
  const parts = Array.from({ length: count }, (_, i) => textPart(text, i));
  assert.equal(parts.join(""), text);
  assert.ok(parts.every((p) => p.length <= TEXT_PART_SIZE + 1));
  assert.ok(
    parts.every(
      (p) => !/[\uD800-\uDBFF]$/.test(p) && !/^[\uDC00-\uDFFF]/.test(p),
    ),
  );
  const html = renderToStaticMarkup(
    React.createElement(LargeReader, { content: text }),
  );
  assert.match(html, /shown in parts as plain text/);
  assert.match(html, /Last part/);
  assert.doesNotMatch(html, /<script>|tail-orchid/);
  assert.ok(html.length < 22000);
  assert.ok(parts.at(-1).endsWith("tail-orchid"));
});

test("agent setup initially grants nothing and defaults to shared-only with acknowledgement required", () => {
  const Setup = component("AgentSetup").default;
  const html = renderToStaticMarkup(
    React.createElement(Setup, {
      sharing: false,
      onChanged: async () => {},
      onClose() {},
    }),
  );
  assert.match(html, /Read shared memory only/);
  assert.match(html, /No raw history or private agent drafts/);
  assert.match(html, /disabled="">Create private settings/);
  assert.doesNotMatch(html, /checked=""|PRE_DRINK_CLIENT_TOKEN/);
});

test("all choice surfaces avoid detached native select popups", () => {
  const Choice = component("ChoiceSelect").default;
  const html = renderToStaticMarkup(
    React.createElement(Choice, {
      label: "Access",
      value: "shared",
      options: [{ value: "shared", label: "Shared only" }],
      onChange() {},
    }),
  );
  assert.match(html, /aria-haspopup="listbox"/);
  assert.match(html, /aria-expanded="false"/);
  assert.match(html, /Shared only/);
  assert.doesNotMatch(html, /<select|role="listbox"/);
  for (const name of [
    "AgentSetup",
    "Memory",
    "Briefs",
    "Curation",
    "VaultMaintenance",
  ]) {
    assert.doesNotMatch(
      readFileSync(new URL(`../src/${name}.tsx`, import.meta.url), "utf8"),
      /<select\b/,
      `${name} must not recreate an OS select popup`,
    );
  }
});

test("agent work sessions are opt-in and display the current bounded grant", () => {
  const Agents = component("Agents").default;
  const inactive = renderToStaticMarkup(
    React.createElement(Agents, { sharing: false, agentSession: null }),
  );
  assert.match(inactive, /Start agent work session/);
  assert.match(inactive, /1 hour/);
  assert.doesNotMatch(inactive, /End session/);
  const active = renderToStaticMarkup(
    React.createElement(Agents, {
      sharing: true,
      agentSession: { remaining_seconds: 120 },
    }),
  );
  assert.match(active, /2 minutes left/);
  assert.match(active, /End session/);
  assert.doesNotMatch(active, /Start agent work session/);
  const untilLock = renderToStaticMarkup(
    React.createElement(Agents, {
      sharing: true,
      agentSession: { remaining_seconds: null },
    }),
  );
  assert.match(untilLock, /Active until your Mac locks/);
});

test("backend library pages expose complete counts and safe form navigation", () => {
  const Page = component("LibraryPage").default;
  const state = {
    items: [{ id: "late-match" }],
    page: { items: [{ id: "late-match" }], total: 101, offset: 100 },
    error: "",
    loading: false,
    refresh() {},
    move() {},
  };
  const html = renderToStaticMarkup(
    React.createElement(Page, {
      state,
      label: "sources",
      children: (item) =>
        React.createElement("article", { key: item.id }, item.id),
    }),
  );
  assert.match(html, /101–101 of 101 sources/);
  assert.match(html, /late-match/);
  assert.match(html, /type="button" disabled="">Next/);
  assert.match(html, /type="button" disabled="">Last/);
  assert.doesNotMatch(html, /<button(?![^>]*type="button")/);
  const error = renderToStaticMarkup(
    React.createElement(Page, {
      state: { ...state, items: [], page: null, error: "Unavailable" },
      label: "sources",
      children() {},
    }),
  );
  assert.match(error, /role="alert"/);
  assert.match(error, /Retry/);
  assert.doesNotMatch(error, /late-match/);
});

test("resumable import states distinguish encrypted preparation from published history", () => {
  const Progress = component("ImportProgress").default;
  const view = (state, busy = false) =>
    renderToStaticMarkup(
      React.createElement(Progress, {
        state,
        busy,
        onResume() {},
        async onPause() {},
        async onDiscard() {},
      }),
    );
  assert.equal(view(null), "");
  const paused = view({
    checkpoint: { label: "fictional <script>.zip", saved: 20, total: 45 },
    running: false,
    phase: "paused",
  });
  assert.match(paused, /Choose ZIP to resume/);
  assert.match(paused, /20 of 45 conversations safely prepared/);
  assert.match(paused, /Nothing from this import is searchable or shared/);
  assert.match(paused, /aria-label="Conversations safely prepared"/);
  assert.doesNotMatch(paused, /<script>/);
  const ready = view({
    checkpoint: { label: "fictional.zip", saved: 45, total: 45 },
    running: false,
    phase: "paused",
  });
  assert.match(ready, /Finish import/);
  assert.doesNotMatch(ready, /Choose ZIP to resume/);
});

test("import status offers pause while preparing and explains the final save boundary", () => {
  const Progress = component("ImportProgress").default;
  const view = (phase) =>
    renderToStaticMarkup(
      React.createElement(Progress, {
        state: {
          checkpoint: { label: "fictional.zip", saved: 20, total: 45 },
          running: true,
          phase,
        },
        busy: true,
        onResume() {},
        async onPause() {},
        async onDiscard() {},
      }),
    );
  assert.match(view("preparing"), /Preparing your import/);
  assert.match(view("preparing"), /Pause import/);
  assert.match(view("saving"), /Saving your archive/);
  assert.match(view("saving"), /disabled=""/);
  assert.match(view("saving"), /final archive save must finish/);
});
