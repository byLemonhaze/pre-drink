// Apply only the non-sensitive appearance preference before the first paint.
(() => {
  let theme = "light";
  try {
    if (localStorage.getItem("pre-drink-theme") === "dark") theme = "dark";
  } catch {
    // A blocked preference store must never prevent startup.
  }
  document.documentElement.dataset.theme = theme;
})();
