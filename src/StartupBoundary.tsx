import { Component, type ReactNode } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";

export default class StartupBoundary extends Component<
  { children: ReactNode },
  { failed: boolean; locked: boolean }
> {
  state = { failed: false, locked: !isTauri() };
  static getDerivedStateFromError() {
    return { failed: true };
  }
  componentDidMount() {
    const root = document.getElementById("root");
    const startup = document.getElementById("startup");
    if (root) root.hidden = false;
    if (startup) startup.hidden = true;
    window.dispatchEvent(new Event("pre-drink-interface-mounted"));
    document.querySelector<HTMLInputElement>("#root input")?.focus();
  }
  componentDidCatch() {
    // Hide the failed interface and stop sharing before allowing a reload.
    // Never report the exception: it may contain private rendered content.
    if (isTauri())
      void invoke("lock_vault")
        .then(() => this.setState({ locked: true }))
        .catch(() => {});
  }
  render() {
    if (!this.state.failed) return this.props.children;
    return (
      <main className="startup-screen">
        <section className="startup-card" role="alert">
          <p className="startup-brand">pre-drink.</p>
          <h1>The interface couldn’t load</h1>
          <p>
            {this.state.locked
              ? "Your vault is locked. Try loading the interface again."
              : "Quit and reopen Pre-Drink to lock the vault and restart the interface."}
          </p>
          {this.state.locked && (
            <button onClick={() => window.location.reload()}>Try again</button>
          )}
        </section>
      </main>
    );
  }
}
