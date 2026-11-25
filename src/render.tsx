import React from "react";
import ReactDOM from "react-dom/client";
import VaultGate from "./VaultGate";
import StartupBoundary from "./StartupBoundary";
import "./styles.css";
import "./security.css";
import "./cabinet.css";

ReactDOM.createRoot(document.getElementById("root")!, {
  // The boundary provides recovery. Exceptions may contain private UI content.
  onCaughtError: () => {},
}).render(
  <React.StrictMode>
    <StartupBoundary>
      <VaultGate />
    </StartupBoundary>
  </React.StrictMode>,
);
