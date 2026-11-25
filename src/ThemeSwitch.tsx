import { useEffect, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Moon, Sun } from "lucide-react";

type Theme = "light" | "dark";
export default function ThemeSwitch() {
  const [theme, setTheme] = useState<Theme>(() =>
    document.documentElement.dataset.theme === "dark" ? "dark" : "light",
  );
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem("pre-drink-theme", theme);
    } catch {
      // Appearance still works for this session if storage is unavailable.
    }
    if (isTauri())
      void Promise.all([
        getCurrentWindow().setTheme(theme),
        getCurrentWindow().setBackgroundColor(
          theme === "dark" ? "#23211f" : "#dfd8cc",
        ),
      ]).catch(() => {});
  }, [theme]);
  return (
    <div className="theme-switch" role="group" aria-label="Appearance">
      <button
        type="button"
        aria-pressed={theme === "light"}
        onClick={() => setTheme("light")}
      >
        <Sun size={13} /> Light
      </button>
      <button
        type="button"
        aria-pressed={theme === "dark"}
        onClick={() => setTheme("dark")}
      >
        <Moon size={13} /> Dark
      </button>
    </div>
  );
}
