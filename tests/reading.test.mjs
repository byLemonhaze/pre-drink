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
