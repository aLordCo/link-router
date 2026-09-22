import { invoke } from "@tauri-apps/api/core";
import type { LocalePref } from "../i18n/locales";

export type ThemePref = "system" | "light" | "dark";

export type AppSettings = {
  locale: LocalePref;
  theme: ThemePref;
};

export type BrowserProfile = {
  id: string;
  name: string;
  browserName: string;
  executable: string;
  profileDir: string | null;
  icon: string | null;
  args: string[];
};

export type RulePattern =
  | { kind: "exact"; pattern: string }
  | { kind: "domain"; pattern: string }
  | { kind: "regex"; pattern: string }
  | { kind: "path"; pattern: string }
  | { kind: "sourceApp"; appName: string };

export function patternText(pattern: RulePattern): string {
  return pattern.kind === "sourceApp" ? pattern.appName : pattern.pattern;
}

export type RulePatternKind = RulePattern["kind"];

export type Rule = {
  id: string;
  priority: number;
  enabled: boolean;
  pattern: RulePattern;
  targetProfileId: string;
};

export type RuleMatch = {
  ruleId: string;
  targetProfileId: string;
  patternKind: string;
};

export type RuleEvaluation = {
  url: string;
  cleanedUrl: string;
  matched: RuleMatch | null;
};

export type RouteDecision =
  | { type: "launch"; url: string; profileId: string }
  | { type: "prompt"; url: string; candidates: BrowserProfile[] }
  | { type: "none"; url: string };

export type RouteError = {
  code: string;
  message: string;
};

function toError(e: unknown): RouteError {
  if (e && typeof e === "object" && "message" in e) {
    const message = String((e as { message: unknown }).message);
    const code =
      "code" in e ? String((e as { code: unknown }).code) : "ipc_error";
    return { code, message };
  }
  return { code: "ipc_error", message: String(e) };
}

export async function evaluateUrlRoute(
  url: string,
  sourceApp: string | null = null,
): Promise<RouteDecision> {
  try {
    return await invoke<RouteDecision>("evaluate_url_route", { url, sourceApp });
  } catch (e) {
    throw toError(e);
  }
}

export async function openInBrowser(url: string, profileId: string): Promise<void> {
  try {
    await invoke("open_in_browser", { url, profileId });
  } catch (e) {
    throw toError(e);
  }
}

export async function openInDefaultBrowser(url: string): Promise<void> {
  try {
    await invoke("open_with_system_default", { url });
  } catch (e) {
    throw toError(e);
  }
}

export async function listBrowserProfiles(): Promise<BrowserProfile[]> {
  try {
    return await invoke<BrowserProfile[]>("list_browser_profiles");
  } catch (e) {
    throw toError(e);
  }
}

export async function evaluateUrl(
  url: string,
  sourceApp: string | null = null,
): Promise<RuleEvaluation> {
  try {
    return await invoke<RuleEvaluation>("evaluate_url", { url, sourceApp });
  } catch (e) {
    throw toError(e);
  }
}

export async function getRules(): Promise<Rule[]> {
  try {
    return await invoke<Rule[]>("get_rules");
  } catch (e) {
    throw toError(e);
  }
}

export async function createRule(rule: Rule): Promise<Rule> {
  try {
    return await invoke<Rule>("create_rule", { rule });
  } catch (e) {
    throw toError(e);
  }
}

export async function updateRule(rule: Rule): Promise<Rule> {
  try {
    return await invoke<Rule>("update_rule", { rule });
  } catch (e) {
    throw toError(e);
  }
}

export async function deleteRule(ruleId: string): Promise<void> {
  try {
    await invoke("delete_rule", { ruleId });
  } catch (e) {
    throw toError(e);
  }
}

export async function getSettings(): Promise<AppSettings> {
  try {
    return await invoke<AppSettings>("get_settings");
  } catch (e) {
    throw toError(e);
  }
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  try {
    await invoke("save_settings", { settings });
  } catch (e) {
    throw toError(e);
  }
}

export async function systemLocale(): Promise<string> {
  try {
    return await invoke<string>("system_locale");
  } catch (e) {
    throw toError(e);
  }
}