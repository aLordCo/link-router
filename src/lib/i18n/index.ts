import { i18n } from "../stores/appState.svelte";
import { es } from "./es";
import { en, type MessageKey } from "./en";
import {
  normalizeLocale,
  resolveLocale,
  SUPPORTED_LOCALES,
  type Locale,
  type LocalePref,
} from "./locales";

export { en, es, normalizeLocale, resolveLocale, SUPPORTED_LOCALES };
export type { Locale, LocalePref, MessageKey };

const dictionaries: Record<Locale, Record<MessageKey, string>> = { en, es };

export function t(
  key: MessageKey,
  vars?: Record<string, string | number>,
): string {
  const text = dictionaries[i18n.locale][key] ?? en[key] ?? key;
  if (!vars) return text;
  let resolved = text;
  for (const [name, value] of Object.entries(vars)) {
    resolved = resolved.split(`{${name}}`).join(String(value));
  }
  return resolved;
}