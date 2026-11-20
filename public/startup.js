// Independent of the app bundle; no error contents, paths or vault data are logged.
(() => {
  const screen = document.getElementById("startup");
  const title = document.getElementById("startup-title");
  const detail = document.getElementById("startup-detail");
  const retry = document.getElementById("startup-retry");
  let ready = false;
  function failed() {
    if (ready) return;
    title.textContent = "Pre-Drink couldn’t finish starting";
    detail.textContent =
      "Try loading the interface again. If this continues, quit and reopen the app.";
    retry.hidden = false;
  }
  const timeout = window.setTimeout(failed, 10000);
  retry.addEventListener("click", () => window.location.reload());
  window.addEventListener("pre-drink-startup-failed", failed);
  window.addEventListener(
    "error",
    (event) => {
      if (event.target instanceof HTMLScriptElement) failed();
    },
    true,
  );
  window.addEventListener(
    "pre-drink-interface-mounted",
    () => {
      ready = true;
      window.clearTimeout(timeout);
      screen.hidden = true;
      document.getElementById("root").hidden = false;
    },
    { once: true },
  );
})();
