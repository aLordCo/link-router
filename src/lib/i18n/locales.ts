export type Locale = "es" | "en";

export type LocalePref = "system" | Locale;

export const SUPPORTED_LOCALES: readonly Locale[] = ["es", "en"] as const;

export function normalizeLocale(tag: string): Locale {
  return /^es([-_]|$)/i.test(tag) ? "es" : "en";
}

export function resolveLocale(pref: LocalePref, systemTag: string): Locale {
  if (pref === "system") return normalizeLocale(systemTag);
  return pref;
}