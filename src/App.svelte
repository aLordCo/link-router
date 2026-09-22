<script lang="ts">
  import { onMount } from "svelte";
  import Prompter from "./routes/Prompter.svelte";
  import Settings from "./routes/Settings.svelte";
  import { ui } from "./lib/stores/appState.svelte";
  import { watchIncomingUrls } from "./lib/services/incoming";

  onMount(() => {
    void watchIncomingUrls();
  });
</script>

<svelte:head>
  <title>LinkRouter</title>
</svelte:head>

<div class="flex h-screen flex-col">
  <header
    class="flex shrink-0 items-center justify-between gap-4 border-b px-4 py-2.5"
    style="border-color: var(--p-card-border); background: var(--p-card)"
  >
    <div class="flex items-center gap-2">
      <span
        class="size-2.5 rounded-full"
        style="background: var(--p-accent)"
      ></span>
      <span class="text-sm font-semibold tracking-tight">LinkRouter</span>
    </div>

    <nav
      class="flex items-center gap-1 rounded-lg p-1"
      style="background: var(--p-item-hover)"
    >
      <button
        type="button"
        class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors {ui.page === 'prompter' ? 'palette-item-selected' : 'palette-muted hover:text-[var(--p-text)]'}"
        onclick={() => (ui.page = "prompter")}
      >
        Prompter
      </button>
      <button
        type="button"
        class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors {ui.page === 'settings' ? 'palette-item-selected' : 'palette-muted hover:text-[var(--p-text)]'}"
        onclick={() => (ui.page = "settings")}
      >
        Configuración
      </button>
    </nav>
  </header>

  <main class="flex-1 overflow-y-auto">
    {#if ui.page === "prompter"}
      <Prompter />
    {:else}
      <Settings />
    {/if}
  </main>
</div>