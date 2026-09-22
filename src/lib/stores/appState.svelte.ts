import {
  evaluateUrlRoute,
  getSettings,
  openInBrowser,
  saveSettings,
  systemLocale,
  type AppSettings,
  type BrowserProfile,
  type RouteDecision,
  type RouteError,
} from "../services/tauri";
import { resolveLocale, type Locale, type LocalePref } from "../i18n/locales";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type Page = "prompter" | "settings";

export const ui = $state({ page: "settings" as Page });

export const settings = $state<AppSettings>({ locale: "system", theme: "system" });
export const localeState = $state({ systemTag: navigator.language });

export const i18n = {
  get pref(): LocalePref {
    return settings.locale;
  },
  get locale(): Locale {
    return resolveLocale(settings.locale, localeState.systemTag);
  },
  async init(): Promise<void> {
    try {
      const loaded = await getSettings();
      settings.locale = loaded.locale;
      settings.theme = loaded.theme;
    } catch {
      // defaultes (system/system) keep working in-memory
    }
    try {
      localeState.systemTag = await systemLocale();
    } catch {
      // fall back to navigator.language
    }
  },
  async setLocale(pref: LocalePref): Promise<void> {
    const previous = settings.locale;
    settings.locale = pref;
    try {
      await saveSettings({ locale: pref, theme: settings.theme });
    } catch (e) {
      settings.locale = previous;
      throw e;
    }
  },
};

function messageOf(e: unknown): string {
  if (e && typeof e === "object") {
    const err = e as Partial<RouteError>;
    return err.message ?? String(e);
  }
  return String(e);
}

function hideWindow(): void {
  try {
    void getCurrentWindow().hide();
  } catch {
    // ventana no disponible (tests o entorno sin ventana)
  }
}

export function createAppState() {
  let url = $state<string>("");
  let decision = $state<RouteDecision | null>(null);
  let profiles = $state<BrowserProfile[]>([]);
  let error = $state<string | null>(null);
  let busy = $state<boolean>(false);

  async function evaluate(target: string, sourceApp: string | null = null): Promise<void> {
    busy = true;
    error = null;
    try {
      url = target;
      const result = await evaluateUrlRoute(target, sourceApp);
      if (result.type === "launch") {
        await openInBrowser(result.url, result.profileId);
        reset();
        hideWindow();
        return;
      }
      decision = result;
      profiles = result.type === "prompt" ? result.candidates : [];
    } catch (e) {
      decision = null;
      profiles = [];
      error = messageOf(e);
    } finally {
      busy = false;
    }
  }

  async function open(profileId: string): Promise<void> {
    if (decision === null || decision.type !== "prompt") return;
    busy = true;
    error = null;
    try {
      await openInBrowser(decision.url, profileId);
      hideWindow();
      decision = null;
      profiles = [];
      url = "";
    } catch (e) {
      error = messageOf(e);
    } finally {
      busy = false;
    }
  }

  function reset(): void {
    url = "";
    decision = null;
    profiles = [];
    error = null;
    busy = false;
  }

  return {
    get url() {
      return url;
    },
    get decision() {
      return decision;
    },
    get profiles() {
      return profiles;
    },
    get error() {
      return error;
    },
    get busy() {
      return busy;
    },
    evaluate,
    open,
    reset,
  };
}

export const appState = createAppState();