<script lang="ts">
  import { fly } from "svelte/transition";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { appState } from "../lib/stores/appState.svelte";
  import { openInDefaultBrowser } from "../lib/services/tauri";
  import { t } from "../lib/i18n";
  import { describeUrl, isSanitized } from "../lib/utils/url";
  import BrowserOption from "../lib/components/BrowserOption.svelte";

  const { decision, url: rawUrl, profiles, busy, error } = $derived(appState);

  const showPrompt = $derived(decision?.type === "prompt");
  const cleanUrl = $derived<string | null>(decision?.url ?? null);
  const segments = $derived(cleanUrl ? describeUrl(cleanUrl) : null);
  const sanitized = $derived(!!cleanUrl && !!rawUrl && isSanitized(rawUrl, cleanUrl));

  let container = $state<HTMLElement | null>(null);
  let selectedIndex = $state(0);
  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    decision;
    selectedIndex = 0;
    copied = false;
  });

  $effect(() => {
    container?.focus();
  });

  function onTest(): void {
    void appState.evaluate(
      "https://github.com/octocat/Hello-World?tab=readme-ov-file&utm_source=newsletter&fbclid=xyz",
    );
  }

  function clampIndex(value: number): number {
    if (profiles.length === 0) return 0;
    return ((value % profiles.length) + profiles.length) % profiles.length;
  }

  function moveSelection(step: number): void {
    selectedIndex = clampIndex(selectedIndex + step);
  }

  async function pick(profileId: string): Promise<void> {
    await appState.open(profileId);
  }

  function pickSelected(): void {
    if (profiles.length === 0) return;
    void pick(profiles[selectedIndex].id);
  }

  async function openDefault(): Promise<void> {
    if (!cleanUrl) return;
    try {
      await openInDefaultBrowser(cleanUrl);
      appState.reset();
    } catch {
      // fallo silencioso: el usuario puede reintentar desde el estado idle
    }
  }

  async function copyClean(): Promise<void> {
    if (!cleanUrl) return;
    try {
      await writeText(cleanUrl);
      copied = true;
      if (copyTimer) clearTimeout(copyTimer);
      copyTimer = setTimeout(() => (copied = false), 1000);
    } catch {
      copied = false;
    }
  }

  function hideWindow(): void {
    try {
      void getCurrentWindow().hide();
    } catch {
      appState.reset();
    }
  }

  function onKeydown(e: KeyboardEvent): void {
    const key = e.key;

    if (key === "Escape") {
      e.preventDefault();
      hideWindow();
      return;
    }

    if ((e.metaKey || e.ctrlKey) && key.toLowerCase() === "c") {
      e.preventDefault();
      void copyClean();
      return;
    }

    if (key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) {
        void openDefault();
      } else {
        pickSelected();
      }
      return;
    }

    if (key === "Tab") {
      e.preventDefault();
      moveSelection(e.shiftKey ? -1 : 1);
      return;
    }

    if (key === "ArrowDown") {
      e.preventDefault();
      moveSelection(1);
      return;
    }

    if (key === "ArrowUp") {
      e.preventDefault();
      moveSelection(-1);
      return;
    }

    if (["Shift", "Control", "Meta", "Alt"].includes(key)) return;

    if (/^[1-9]$/.test(key) && !e.metaKey && !e.ctrlKey && !e.altKey) {
      const index = Number(key) - 1;
      if (index < profiles.length) {
        e.preventDefault();
        void pick(profiles[index].id);
      }
    }
  }
</script>

<div class="flex min-h-full flex-col">
  <div class="flex flex-1 items-center justify-center px-6 py-8">
    <div
      role="listbox"
      aria-label={t("prompter.ariaLabel")}
      aria-activedescendant={showPrompt ? `profile-opt-${selectedIndex}` : undefined}
      tabindex="-1"
      bind:this={container}
      onkeydown={onKeydown}
      class="palette-card w-full max-w-[560px] overflow-hidden outline-none"
      transition:fly={{ y: 8, duration: 90 }}
    >
      <header
        class="flex items-center gap-3 border-b px-5 py-4"
        style="border-color: var(--p-card-border)"
      >
        {#if segments}
          <div class="min-w-0 flex-1 text-sm leading-relaxed">
            <span class="palette-faint">{segments.scheme}//</span>
            {#if segments.subdomain}
              <span class="palette-muted">{segments.subdomain}.</span>
            {/if}
            <span class="font-semibold">{segments.primary}</span>
            <span class="palette-muted break-all">{segments.tail}</span>
          </div>
        {:else if rawUrl}
          <div class="palette-muted min-w-0 flex-1 truncate text-sm">{rawUrl}</div>
        {:else}
          <div class="palette-faint flex-1 text-xs uppercase tracking-wider">{t("prompter.incomingUrl")}</div>
        {/if}
        {#if sanitized}
          <span
            class="palette-pill shrink-0"
            title={t("prompter.sanitizedTitle")}
          >
            {t("prompter.sanitizedPill")}
          </span>
        {/if}
        {#if copied}
          <span
            class="palette-pill palette-pill-copied shrink-0"
            transition:fly={{ y: 4, duration: 70 }}
          >
            {t("prompter.copied")}
          </span>
        {/if}
      </header>

      {#if busy}
        <p class="palette-faint px-5 py-12 text-center text-sm">{t("prompter.evaluating")}</p>
      {:else if error}
        <p class="text-danger px-5 py-12 text-center text-sm">{error}</p>
      {:else if showPrompt}
        <div class="max-h-[280px] overflow-y-auto p-2">
          {#each profiles as p, i (p.id)}
            <BrowserOption
              profile={p}
              index={i}
              selected={i === selectedIndex}
              onSelect={moveSelection}
              onPick={pick}
            />
          {/each}
        </div>
      {:else if decision?.type === "launch"}
        <p class="palette-muted px-5 py-12 text-center text-sm">{t("prompter.autoRulePrefix")}<span class="font-mono">{decision.profileId}</span>{t("prompter.autoRuleSuffix")}</p>
      {:else if decision?.type === "none"}
        <p class="palette-muted px-5 py-12 text-center text-sm">
          {t("prompter.noRuleNoBrowsers")}
        </p>
      {:else}
        <div class="flex flex-col items-center gap-4 px-5 py-12">
          <div class="flex flex-col items-center gap-2">
            <p class="palette-faint text-center text-xs uppercase tracking-wider">
              {t("prompter.statusWaiting")}
            </p>
            <p class="palette-muted max-w-md text-center text-sm leading-relaxed">
              {@html t("prompter.intro")}
            </p>
          </div>
          <button
            type="button"
            class="rounded-lg px-4 py-2 text-sm font-medium transition-colors"
            style="background: var(--p-item-hover)"
            onclick={onTest}
          >
            {t("prompter.trySample")}
          </button>
          <p class="palette-faint max-w-md text-center text-xs leading-relaxed">
            {t("prompter.setupHint")}
          </p>
        </div>
      {/if}

      <footer
        class="flex items-center justify-between gap-3 border-t px-5 py-2.5"
        style="border-color: var(--p-card-border)"
      >
        <span class="palette-faint text-[11px]">
          {t("prompter.footerKeys")}
        </span>
        <span class="palette-faint shrink-0 text-[11px]">
          {t(profiles.length === 1 ? "prompter.profile.one" : "prompter.profile.other", {
            n: profiles.length,
          })}
        </span>
      </footer>
    </div>
  </div>
</div>