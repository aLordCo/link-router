<script lang="ts">
  import { isSchemeRegistered, registerScheme } from "../lib/services/deepLink";
  import { listBrowserProfiles, type BrowserProfile } from "../lib/services/tauri";
  import RulesManager from "../lib/components/RulesManager.svelte";

  let httpStatus = $state<string>("sin verificar");
  let checking = $state(false);

  let detected = $state<BrowserProfile[]>([]);
  let detectedStatus = $state<string | null>(null);

  async function loadProfiles(): Promise<void> {
    detectedStatus = "Cargando…";
    try {
      detected = await listBrowserProfiles();
      detectedStatus = detected.length === 0 ? "No se encontraron navegadores" : null;
    } catch (e) {
      detected = [];
      detectedStatus = `Error: ${String(e)}`;
    }
  }

  async function refresh(): Promise<void> {
    checking = true;
    try {
      const http = await isSchemeRegistered("https");
      httpStatus = http ? "LinkRouter es el handler de https" : "LinkRouter no es el handler de https";
    } catch (e) {
      httpStatus = `Error: ${String(e)}`;
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
      httpStatus = `Error: ${String(e)}`;
    } finally {
      checking = false;
    }
  }
</script>

<section class="mx-auto w-full max-w-lg space-y-4 px-6 py-16">
  <h1 class="text-xl font-semibold tracking-tight">Ajustes</h1>

  <div class="rounded-xl border border-zinc-800 bg-zinc-900/60 p-5">
    <h2 class="text-sm font-medium text-zinc-300">Navegador predeterminado</h2>
    <p class="mt-1 text-xs text-zinc-500">Registra LinkRouter como gestor de enlaces http/https.</p>

    <div class="mt-4 flex items-center gap-3">
      <button
        type="button"
        class="rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-900 hover:bg-white disabled:opacity-50"
        onclick={makeDefault}
        disabled={checking}
      >
        Establecer como predeterminado
      </button>
      <button
        type="button"
        class="rounded-md border border-zinc-700 px-4 py-2 text-sm text-zinc-300 hover:border-zinc-500 disabled:opacity-50"
        onclick={refresh}
        disabled={checking}
      >
        Verificar
      </button>
    </div>
    <div class="mt-3 text-xs text-zinc-400">{httpStatus}</div>
  </div>

  <div class="rounded-xl border border-zinc-800 bg-zinc-900/60 p-5">
    <RulesManager />
  </div>

  <div class="rounded-xl border border-zinc-800 bg-zinc-900/60 p-5">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-medium text-zinc-300">Navegadores detectados</h2>
      <button
        type="button"
        class="rounded-md border border-zinc-700 px-3 py-1 text-xs text-zinc-300 hover:border-zinc-500"
        onclick={loadProfiles}
      >
        Detectar
      </button>
    </div>
    <p class="mt-1 text-xs text-zinc-500">Perfiles detectados por el Sistema.</p>

    <ul class="mt-4 space-y-2">
      {#each detected as profile (profile.id)}
        <li class="flex items-center gap-3 rounded-lg border border-zinc-800 bg-zinc-950/60 px-3 py-2">
          <span
            class="flex h-7 w-7 shrink-0 items-center justify-center rounded bg-zinc-800 text-xs font-semibold text-zinc-300"
          >
            {profile.browserName.slice(0, 1).toUpperCase()}
          </span>
          <span class="flex min-w-0 flex-col">
            <span class="truncate text-sm text-zinc-100">{profile.name}</span>
            <span class="truncate text-xs text-zinc-500">{profile.executable}</span>
          </span>
          <span class="ml-auto shrink-0 text-xs text-zinc-500">{profile.id}</span>
        </li>
      {/each}
    </ul>
    {#if detectedStatus}
      <p class="mt-3 text-xs text-zinc-400">{detectedStatus}</p>
    {/if}
  </div>
</section>