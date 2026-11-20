import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const source = readFileSync(
  new URL("../public/startup.js", import.meta.url),
  "utf8",
);
function boot() {
  const events = new Map();
  const elements = new Map(
    ["startup", "startup-title", "startup-detail", "startup-retry", "root"].map(
      (id) => [
        id,
        {
          hidden: id === "startup-retry" || id === "root",
          textContent: "",
          addEventListener(name, handler) {
            this[name] = handler;
          },
        },
      ],
    ),
  );
  let timeout;
  let cleared = false;
  let reloads = 0;
  class HTMLScriptElement {}
  vm.runInNewContext(source, {
    HTMLScriptElement,
    document: { getElementById: (id) => elements.get(id) },
    window: {
      setTimeout(handler, ms) {
        assert.equal(ms, 10000);
        timeout = handler;
        return 1;
      },
      clearTimeout(id) {
        assert.equal(id, 1);
        cleared = true;
      },
      addEventListener(name, handler) {
        events.set(name, handler);
      },
      location: {
        reload() {
          reloads++;
        },
      },
    },
  });
  return {
    elements,
    fire: (name, payload) => events.get(name)?.(payload),
    timeout: () => timeout(),
    script: () => new HTMLScriptElement(),
    cleared: () => cleared,
    reloads: () => reloads,
  };
}
test("stalled bootstrap exposes a retry instead of an empty root", () => {
  const app = boot();
  app.timeout();
  assert.equal(app.elements.get("startup").hidden, false);
  assert.equal(app.elements.get("root").hidden, true);
  assert.equal(app.elements.get("startup-retry").hidden, false);
  app.elements.get("startup-retry").click();
  assert.equal(app.reloads(), 1);
});
test("missing scripts and rejected application imports show recovery", () => {
  for (const event of ["error", "pre-drink-startup-failed"]) {
    const app = boot();
    app.fire(event, { target: app.script() });
    assert.match(
      app.elements.get("startup-title").textContent,
      /couldn’t finish/,
    );
    assert.equal(app.elements.get("startup-retry").hidden, false);
  }
});
test("mounted interface removes fallback and ignores late boot failures", () => {
  const app = boot();
  app.fire("pre-drink-interface-mounted");
  assert.equal(app.cleared(), true);
  app.timeout();
  app.fire("pre-drink-startup-failed");
  assert.equal(app.elements.get("root").hidden, false);
  assert.equal(app.elements.get("startup").hidden, true);
  assert.equal(app.elements.get("startup-retry").hidden, true);
});
test("unrelated resource errors do not replace the loading state", () => {
  const app = boot();
  app.fire("error", { target: {} });
  assert.equal(app.elements.get("startup-retry").hidden, true);
});
