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

test("light and dark controls retain readable text, boundaries and focus", () => {
  const css = readFileSync(
    new URL("../public/theme.css", import.meta.url),
    "utf8",
  );
  const themes = [...css.matchAll(/\{([^}]+)\}/g)];
  const colors = {};
  const luminance = (hex) => {
    const channels = [1, 3, 5].map((offset) => {
      const value = parseInt(hex.slice(offset, offset + 2), 16) / 255;
      return value <= 0.04045
        ? value / 12.92
        : ((value + 0.055) / 1.055) ** 2.4;
    });
    return channels.reduce(
      (sum, value, i) => sum + value * [0.2126, 0.7152, 0.0722][i],
      0,
    );
  };
  function contrast(foreground, background, minimum, theme) {
    assert.ok(
      colors[foreground] && colors[background],
      `Missing theme token ${foreground}/${background}`,
    );
    const [low, high] = [
      luminance(colors[foreground]),
      luminance(colors[background]),
    ].sort((a, b) => a - b);
    const ratio = (high + 0.05) / (low + 0.05);
    assert.ok(
      ratio >= minimum,
      `${theme}: ${foreground}/${background} contrast ${ratio.toFixed(2)} < ${minimum}`,
    );
  }
  for (const [i, block] of themes.entries()) {
    Object.assign(
      colors,
      Object.fromEntries(
        [...block[1].matchAll(/--([\w-]+):\s*(#[\da-f]{6});/g)].map((m) => [
          m[1],
          m[2],
        ]),
      ),
    );
    const theme = i ? "dark" : "light";
    for (const background of ["canvas", "surface", "field", "raised"]) {
      contrast("text", background, 4.5, theme);
      contrast("muted", background, 4.5, theme);
      contrast("control-line", background, 3, theme);
      contrast("accent-text", background, 3, theme);
    }
    contrast("primary-text", "primary-bg", 4.5, theme);
    contrast("control-line", "hover", 3, theme);
    contrast("accent-ink", "accent", 4.5, theme);
  }
});

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

test("curator setup explains the shared provider workspace without granting foreign history", () => {
  const { scopeDescription } = component("AgentSetup");
  for (const [format, provider] of [
    ["codex", "ChatGPT"],
    ["grok", "Grok"],
    ["claude", "Claude"],
  ]) {
    const curate = scopeDescription(format, "curate");
    assert.ok(curate.startsWith(`${provider} history only`));
    assert.match(
      curate,
      /share one workspace and can revise each other’s contributions/,
    );
    assert.match(scopeDescription(format, "history"), /Cannot write memories/);
  }
  assert.match(
    scopeDescription("generic", "curate"),
    /No raw history or private agent drafts/,
  );
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

test("deletion review explains the entire pending import loss and explicit reimport boundary", () => {
  const Notice = component("ArchiveRemoval").PendingImportRemovalNotice;
  const view = (pending) =>
    renderToStaticMarkup(React.createElement(Notice, { pending }));
  assert.equal(view(null), "");
  const html = view({ label: "fictional <script>.zip", saved: 20, total: 45 });
  assert.match(html, /entire unfinished import/);
  assert.match(html, /20 of 45 conversations prepared/);
  assert.match(html, /remaining ZIP contents/);
  assert.match(
    html,
    /Other conversations already in your archive stay unchanged/,
  );
  assert.match(
    html,
    /Selecting that ZIP again starts a new import and can restore deleted conversations/,
  );
  assert.doesNotMatch(html, /<script>/);
});

test("upgrade notice reports legacy approval resets even without new sensitive flags", () => {
  const Notice = component("PrivacyNotice").default;
  const renderNotice = (update) =>
    renderToStaticMarkup(React.createElement(Notice, { update }));
  const legacy = renderNotice({
    flagged: 0,
    approvals_reset: 5,
    legacy_review_upgrade: true,
  });
  assert.match(legacy, /role="status"/);
  assert.match(legacy, /5 earlier approvals returned to draft/);
  assert.match(legacy, /fresh review before sharing/);
  assert.doesNotMatch(legacy, /Unrelated approvals were preserved/);
  const targeted = renderNotice({
    flagged: 2,
    approvals_reset: 2,
    legacy_review_upgrade: false,
  });
  assert.match(targeted, /2 restrictions/);
  assert.match(targeted, /2 linked approvals/);
  assert.match(targeted, /Unrelated approvals were preserved/);
  assert.equal(
    renderNotice({
      flagged: 0,
      approvals_reset: 0,
      legacy_review_upgrade: false,
    }),
    "",
  );
});

test("individual client controls distinguish pause, global sharing and last request", () => {
  const { ClientCard } = component("AgentActivity");
  const client = {
    id: "fictional",
    name: "Fictional reader",
    provider: null,
    allow_chats: false,
    can_curate: false,
    paused: true,
    last_request_at: null,
  };
  const html = renderToStaticMarkup(
    React.createElement(ClientCard, {
      client,
      busy: false,
      onPause() {},
      onRevoke() {},
    }),
  );
  assert.match(html, /Paused/);
  assert.match(html, /Resume access for Fictional reader/);
  assert.match(html, /No recorded request/);
  assert.match(html, /Approved memory and briefs only/);
  const enabled = renderToStaticMarkup(
    React.createElement(ClientCard, {
      client: { ...client, paused: false },
      busy: true,
      onPause() {},
      onRevoke() {},
    }),
  );
  assert.match(enabled, /Allowed when agent sharing is on/);
  assert.match(enabled, /disabled="" aria-label="Pause access/);
});

test("activity distinguishes processing from delivery and exposes bounded paging", () => {
  const Activity = component("AgentActivity").default;
  const html = renderToStaticMarkup(
    React.createElement(Activity, {
      page: {
        totals: { denied: 5, trimmed: 20 },
        items: [
          {
            id: 7,
            client_name: "Fictional reader",
            occurred_at: 1000,
            operation: "get_facts",
            outcome: "started",
          },
        ],
        next_before: 7,
        session_generation: 1,
      },
      busy: false,
      onRefresh() {},
      onMore() {},
      onClear() {},
    }),
  );
  assert.match(html, /final result not recorded/);
  assert.match(html, /not that a client received/);
  assert.match(html, /Older activity/);
  assert.match(html, /100 events per connection and 1,000 overall/);
  assert.match(html, /Rejected credentials: at least 5/);
  assert.match(html, /Older events removed: at least 20/);
  assert.doesNotMatch(html, /PRE_DRINK_CLIENT_TOKEN/);
});

// Exercise the actual Agents handlers with deferred IPC replies. This small
// hook harness controls state/effect scheduling only; it makes no DOM or native
// accessibility claims and adds no production/test dependency.
function interactiveAgents() {
  const path = fileURLToPath(new URL("../src/Agents.tsx", import.meta.url));
  const source = ts.transpileModule(readFileSync(path, "utf8"), {
    compilerOptions: {
      module: ts.ModuleKind.CommonJS,
      jsx: ts.JsxEmit.ReactJSX,
      esModuleInterop: true,
    },
  }).outputText;
  const slots = [];
  let cursor = 0;
  const effects = [];
  const requests = [];
  const hooks = {
    ...React,
    useState(initial) {
      const i = cursor++;
      if (!(i in slots)) slots[i] = initial;
      return [
        slots[i],
        (value) => {
          slots[i] = typeof value === "function" ? value(slots[i]) : value;
        },
      ];
    },
    useRef(initial) {
      const i = cursor++;
      if (!(i in slots)) slots[i] = { current: initial };
      return slots[i];
    },
    useEffect(effect, dependencies) {
      const i = cursor++;
      if (!(i in slots)) {
        slots[i] = dependencies;
        effects.push(effect);
      }
    },
  };
  const module = { exports: {} };
  const require = createRequire(path);
  const imports = (id) => {
    if (id === "react") return hooks;
    if (id === "@tauri-apps/api/core")
      return {
        invoke: (command, args) =>
          new Promise((resolve, reject) =>
            requests.push({ command, args, resolve, reject }),
          ),
      };
    return id.startsWith("./") ? component(id.slice(2)) : require(id);
  };
  vm.runInThisContext(`(function(module, exports, require) {${source}\n})`, {
    filename: path,
  })(module, module.exports, imports);
  function render() {
    cursor = 0;
    const tree = module.exports.default({ sharing: false, agentSession: null });
    const elements = [];
    const visit = (node) => {
      if (Array.isArray(node)) return node.forEach(visit);
      if (!React.isValidElement(node)) return;
      elements.push(node);
      visit(node.props.children);
    };
    visit(tree);
    return elements;
  }
  render();
  effects.forEach((effect) => effect());
  return { render, requests };
}
const settleHandlers = () => new Promise((resolve) => setImmediate(resolve));
const activityFixture = (id) => ({
  totals: { denied: 0, trimmed: 0 },
  items: [
    {
      id,
      client_name: "Fictional reader",
      occurred_at: 1000,
      operation: "get_facts",
      outcome: "completed",
    },
  ],
  next_before: null,
  session_generation: 1,
});
const readerFixture = {
  id: "fictional",
  name: "Fictional reader",
  allow_chats: false,
  provider: null,
  can_curate: false,
  paused: false,
  last_request_at: 1000,
};
function child(view, name) {
  return view.find((node) => node.type?.name === name);
}

test("delayed refresh cannot overwrite a confirmed pause with old access status", async () => {
  const ui = interactiveAgents();
  ui.requests[0].resolve([readerFixture]);
  ui.requests[1].resolve(activityFixture(1));
  await settleHandlers();
  child(ui.render(), "AgentActivity").props.onRefresh();
  const stale = ui.requests.slice(2, 4);
  child(ui.render(), "ClientCard").props.onPause();
  const pause = ui.requests.find((r) => r.command === "set_client_paused");
  assert.equal(pause.args.paused, true);
  pause.resolve();
  await settleHandlers();
  const latest = ui.requests.slice(-2);
  latest[0].resolve([{ ...readerFixture, paused: true }]);
  latest[1].resolve(activityFixture(2));
  await settleHandlers();
  assert.equal(child(ui.render(), "ClientCard").props.client.paused, true);
  stale[0].resolve([readerFixture]);
  stale[1].resolve(activityFixture(1));
  await settleHandlers();
  assert.equal(child(ui.render(), "ClientCard").props.client.paused, true);
  assert.equal(child(ui.render(), "AgentActivity").props.page.items[0].id, 2);
});

test("clearing activity cannot be undone on screen by an older refresh reply", async () => {
  const ui = interactiveAgents();
  ui.requests[0].resolve([readerFixture]);
  ui.requests[1].resolve(activityFixture(1));
  await settleHandlers();
  child(ui.render(), "AgentActivity").props.onRefresh();
  const stale = ui.requests.slice(2, 4);
  child(ui.render(), "AgentActivity").props.onClear();
  ui.render()
    .find((n) => n.type === "input" && n.props.type === "checkbox")
    .props.onChange({ target: { checked: true } });
  ui.render()
    .find((n) => n.type === "button" && n.props.children === "Clear activity")
    .props.onClick();
  ui.requests.find((r) => r.command === "clear_agent_activity").resolve();
  await settleHandlers();
  const latest = ui.requests.slice(-2);
  latest[0].resolve([{ ...readerFixture, last_request_at: null }]);
  latest[1].resolve({ ...activityFixture(1), items: [] });
  await settleHandlers();
  stale[0].resolve([readerFixture]);
  stale[1].resolve(activityFixture(1));
  await settleHandlers();
  assert.equal(child(ui.render(), "AgentActivity").props.page.items.length, 0);
  assert.equal(
    child(ui.render(), "ClientCard").props.client.last_request_at,
    null,
  );
});

test("single revoke requires a reviewed confirmation and targets only that connection", async () => {
  const ui = interactiveAgents();
  ui.requests[0].resolve([
    readerFixture,
    { ...readerFixture, id: "second", name: "Fictional second" },
  ]);
  ui.requests[1].resolve(activityFixture(1));
  await settleHandlers();
  child(ui.render(), "ClientCard").props.onRevoke();
  assert.equal(ui.requests.length, 2);
  const action = () =>
    ui
      .render()
      .find(
        (n) =>
          n.type === "button" && n.props.children === "Revoke this connection",
      );
  assert.equal(action().props.disabled, true);
  action().props.onClick();
  assert.equal(ui.requests.length, 2);
  ui.render()
    .find((n) => n.type === "button" && n.props.children === "Cancel")
    .props.onClick();
  assert.equal(action(), undefined);
  child(ui.render(), "ClientCard").props.onRevoke();
  ui.render()
    .find((n) => n.type === "input" && n.props.type === "checkbox")
    .props.onChange({ target: { checked: true } });
  action().props.onClick();
  const revoke = ui.requests.find((r) => r.command === "revoke_client");
  assert.equal(revoke.args.id, readerFixture.id);
  assert.equal(
    ui.requests.some((r) => r.command === "revoke_all_clients"),
    false,
  );
  revoke.resolve();
  await settleHandlers();
  ui.requests.slice(-2)[0].resolve([{ ...readerFixture, id: "second" }]);
  ui.requests.slice(-2)[1].resolve(activityFixture(2));
  await settleHandlers();
  assert.equal(child(ui.render(), "ClientCard").props.client.id, "second");
});

test("denied-only activity can be cleared and explains the missing-attempt boundary", () => {
  const html = renderToStaticMarkup(
    React.createElement(component("AgentActivity").default, {
      page: {
        ...activityFixture(1),
        items: [],
        totals: { denied: 12, trimmed: 0 },
      },
      busy: false,
      onRefresh() {},
      onMore() {},
      onClear() {},
    }),
  );
  assert.match(html, /Rejected credentials: at least 12/);
  assert.match(html, /before a request can be parsed are not counted/);
  assert.match(html, /<button class="button secondary">Clear activity/);
});

const cabinetMemory = {
  id: "fictional-cabinet-note",
  kind: "decision",
  content: "Use native plants.",
  approved: false,
  private: false,
  sensitive: false,
  source_restricted: false,
  source_message_id: null,
  source_conversation_id: null,
  source_title: null,
  created_at: 0,
  updated_at: 0,
};

test("cabinet review and sharing labels remain independent", () => {
  const { memoryLabels } = component("MemoryCard");
  assert.deepEqual(memoryLabels(cabinetMemory), {
    review: "Draft",
    access: "Not approved for sharing",
    eligible: false,
  });
  assert.deepEqual(memoryLabels({ ...cabinetMemory, approved: true }), {
    review: "Reviewed",
    access: "Eligible for sharing",
    eligible: true,
  });
  for (const restriction of ["private", "sensitive", "source_restricted"]) {
    const status = memoryLabels({
      ...cabinetMemory,
      approved: true,
      [restriction]: true,
    });
    assert.equal(status.review, "Reviewed");
    assert.equal(status.access, "Withheld");
    assert.equal(status.eligible, false);
  }
  assert.equal(
    memoryLabels({
      ...cabinetMemory,
      approved: true,
      publication_basis: "automatic_policy",
    }).review,
    "Auto-published",
  );
});

test("memory inspector retains full inert text and describes conditional access", () => {
  const { default: Card, MemoryInspector } = component("MemoryCard");
  const content =
    "Fictional garden. ".repeat(1000) + "<script>fictional-tail</script>";
  const memory = {
    ...cabinetMemory,
    content,
    curator_provider: "grok",
    curator_revision: 3,
  };
  const card = renderToStaticMarkup(
    React.createElement(Card, { memory, onRead() {} }),
  );
  assert.match(card, /Read note/);
  assert.match(card, /grok contribution/);
  assert.doesNotMatch(card, /<script>/);
  const inspector = renderToStaticMarkup(
    React.createElement(MemoryInspector, {
      memory,
      onClose() {},
      onSource() {},
    }),
  );
  assert.match(inspector, /aria-label="Memory details"/);
  assert.match(inspector, /fictional-tail/);
  assert.match(inspector, /Eligibility is not an access grant/);
  assert.match(inspector, /revision 3/);
  assert.doesNotMatch(inspector, /<script>/);
});
