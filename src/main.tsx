import React from "react";
import ReactDOM from "react-dom/client";
import VaultGate from "./VaultGate";
import "./styles.css";
import "./security.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <VaultGate />
  </React.StrictMode>,
);
