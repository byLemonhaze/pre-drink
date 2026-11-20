// Keep startup independent of the React/application module graph.
import { invoke, isTauri } from "@tauri-apps/api/core";

if (isTauri())
  void invoke("startup_stage", { stage: "module-ready" }).catch(() => {});

void import("./render").catch(() => {
  window.dispatchEvent(new Event("pre-drink-startup-failed"));
});
