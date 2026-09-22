<script lang="ts">
  import { isSchemeRegistered, registerScheme } from "../lib/services/deepLink";
  import { listBrowserProfiles, type BrowserProfile } from "../lib/services/tauri";
  import { i18n, theme } from "../lib/stores/appState.svelte";
  import { t, type LocalePref, type MessageKey } from "../lib/i18n";
  import type { ThemePref } from "../lib/services/tauri";
  import RulesManager from "../lib/components/RulesManager.svelte";

  type Status = {
    key: MessageKey;
    vars?: Record<string, string>;
  };

  function statusText(status: Status | null): string {
    if (!status) return "";
    return status.vars ? t(status.key, status.vars) : t(status.key);
  }

  let httpStatus = $state<Status | null>({ key: "settings.notChecked" });
  let checking = $state(false);

  let detected = $state<BrowserProfile[]>([]);
  let detectedStatus = $state<Status | null>(null);

  async function loadProfiles(): Promise<void> {
    detectedStatus = { key: "settings.loading" };
    try {
      detected = await listBrowserProfiles();
      detectedStatus = detected.length === 0 ? { key: "settings.noBrowsers" } : null;
    } catch (e) {
      detected = [];
      detectedStatus = { key: "common.error", vars: { e: String(e) } };
    }
  }

  async function refresh(): Promise<void> {
    checking = true;
    try {
      const http = await isSchemeRegistered("https");
      httpStatus = http ? { key: "settings.httpsOn" } : { key: "settings.httpsOff" };
    } catch (e) {
      httpStatus = { key: "common.error", vars: { e: String(e) } };
    } finally {
      checking = false;
    }
  }

  async function makeDefault(): Promise<void> {
    checking = true;
    try {
      await registerScheme("https");
      await registerScheme("http");
      await refresh();
    } catch (e) {
      httpStatus = { key: "common.error", vars: { e: String(e) } };
    } finally {
      checking = false;
    }
  }

  function onLanguageChange(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value as LocalePref;
    void i18n.setLocale(value).catch(() => {});
  }

  function onThemeChange(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value as ThemePref;
    void theme.setTheme(value).catch(() => {});
  }
</script>

<section class="mx-auto w-full max-w-lg space-y-4 px-6 py-16">
  <h1 class="text-xl font-semibold tracking-tight text-strong">{t("settings.title")}</h1>

  <div class="surface-card rounded-xl p-5">
    <h2 class="text-sm font-medium text-strong">{t("settings.languageTitle")}</h2>
    <p class="mt-1 text-xs text-muted">{t("settings.languageHint")}</p>
    <select
      class="field mt-4 w-full rounded-md px-3 py-2 text-sm"
      value={i18n.pref}
      onchange={onLanguageChange}
    >
      <option value="system">{t("settings.languageOptionSystem")}</option>
      <option value="es">{t("settings.languageOptionEs")}</option>
      <option value="en">{t("settings.languageOptionEn")}</option>
    </select>
  </div>

  <div class="surface-card rounded-xl p-5">
    <h2 class="text-sm font-medium text-strong">{t("settings.themeTitle")}</h2>
    <p class="mt-1 text-xs text-muted">{t("settings.themeHint")}</p>
    <select
      class="field mt-4 w-full rounded-md px-3 py-2 text-sm"
      value={theme.pref}
      onchange={onThemeChange}
    >
      <option value="system">{t("settings.themeOptionSystem")}</option>
      <option value="light">{t("settings.themeOptionLight")}</option>
      <option value="dark">{t("settings.themeOptionDark")}</option>
    </select>
  </div>

  <div class="surface-card rounded-xl p-5">
    <h2 class="text-sm font-medium text-strong">{t("settings.defaultBrowser")}</h2>
    <p class="mt-1 text-xs text-muted">{t("settings.defaultHint")}</p>

    <div class="mt-4 flex items-center gap-3">
      <button
        type="button"
        class="btn-solid rounded-md px-4 py-2 text-sm font-medium disabled:opacity-50"
        onclick={makeDefault}
        disabled={checking}
      >
        {t("settings.setDefault")}
      </button>
      <button
        type="button"
        class="btn-outline rounded-md px-4 py-2 text-sm disabled:opacity-50"
        onclick={refresh}
        disabled={checking}
      >
        {t("settings.check")}
      </button>
    </div>
    <div class="mt-3 text-xs text-muted">{statusText(httpStatus)}</div>
  </div>

  <div class="surface-card rounded-xl p-5">
    <RulesManager />
  </div>

  <div class="surface-card rounded-xl p-5">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-medium text-strong">{t("settings.detectedBrowsers")}</h2>
      <button
        type="button"
        class="btn-outline rounded-md px-3 py-1 text-xs"
        onclick={loadProfiles}
      >
        {t("settings.detect")}
      </button>
    </div>
    <p class="mt-1 text-xs text-muted">{t("settings.detectedHint")}</p>

    <ul class="mt-4 space-y-2">
      {#each detected as profile (profile.id)}
        <li class="surface-deep flex items-center gap-3 rounded-lg px-3 py-2">
          <span class="avatar-chip flex h-7 w-7 shrink-0 items-center justify-center rounded text-xs font-semibold">
            {profile.browserName.slice(0, 1).toUpperCase()}
          </span>
          <span class="flex min-w-0 flex-col">
            <span class="text-strong truncate text-sm">{profile.name}</span>
            <span class="text-muted truncate text-xs">{profile.executable}</span>
          </span>
          <span class="text-muted ml-auto shrink-0 text-xs">{profile.id}</span>
        </li>
      {/each}
    </ul>
    {#if detectedStatus}
      <p class="text-muted mt-3 text-xs">{statusText(detectedStatus)}</p>
    {/if}
  </div>
</section>