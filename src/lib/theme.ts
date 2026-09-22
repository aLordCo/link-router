import type { ThemePref } from "./services/tauri";

export type Theme = "light" | "dark";

export const THEME_CACHE_KEY = "linkRouter.theme";

export function resolveTheme(pref: ThemePref, systemDark: boolean): Theme {
  if (pref === "light") return "light";
  if (pref === "dark") return "dark";
  return systemDark ? "dark" : "light";
}

export function systemPrefersDark(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
  );
}

export function cachedTheme(): Theme | null {
  try {
    const value = localStorage.getItem(THEME_CACHE_KEY);
    if (value === "light" || value === "dark") return value;
  } catch {
    // storage unavailable
  }
  return null;
}