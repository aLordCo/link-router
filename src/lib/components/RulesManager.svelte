<script lang="ts">
  import {
    createRule,
    deleteRule,
    getRules,
    listBrowserProfiles,
    patternText,
    updateRule,
    type BrowserProfile,
    type Rule,
    type RulePattern,
    type RulePatternKind,
  } from "../services/tauri";

  const KIND_OPTIONS: { value: RulePatternKind; label: string; placeholder: string }[] = [
    { value: "exact", label: "Exacta", placeholder: "github.com o https://github.com/x" },
    { value: "domain", label: "Dominio", placeholder: "github.com o *.github.com" },
    { value: "regex", label: "Regex", placeholder: "^https://.*\\\\.notion\\\\.site/.+" },
    { value: "path", label: "Prefijo de ruta", placeholder: "https://meet.google.com/*" },
    { value: "sourceApp", label: "App origen", placeholder: "Slack" },
  ];

  let rules = $state<Rule[]>([]);
  let profiles = $state<BrowserProfile[]>([]);
  let status = $state<string | null>(null);
  let busy = $state(false);

  let kind = $state<RulePatternKind>("domain");
  let pattern = $state("");
  let targetProfileId = $state("");
  let priority = $state(0);

  const kindMeta = $derived(
    KIND_OPTIONS.find((option) => option.value === kind) ?? KIND_OPTIONS[0],
  );

  async function refresh(): Promise<void> {
    try {
      const [loadedRules, loadedProfiles] = await Promise.all([getRules(), listBrowserProfiles()]);
      rules = loadedRules;
      profiles = loadedProfiles;
      if (targetProfileId && !loadedProfiles.some((p) => p.id === targetProfileId)) {
        targetProfileId = "";
      }
    } catch (e) {
      status = `Error: ${String(e)}`;
    }
  }

  $effect(() => {
    void refresh();
  });

  function buildPattern(): RulePattern | null {
    const value = pattern.trim();
    if (!value) return null;
    return kind === "sourceApp" ? { kind, appName: value } : { kind, pattern: value };
  }

  async function add(): Promise<void> {
    const built = buildPattern();
    if (!built || !targetProfileId) {
      status = "Completá el patrón y elegí un perfil destino.";
      return;
    }
    busy = true;
    status = null;
    try {
      const nextPriority = rules.reduce((max, rule) => Math.max(max, rule.priority), 0) + 1;
      await createRule({
        id: crypto.randomUUID(),
        priority: priority || nextPriority,
        enabled: true,
        pattern: built,
        targetProfileId,
      });
      pattern = "";
      await refresh();
    } catch (e) {
      status = `Error al crear la regla: ${String(e)}`;
    } finally {
      busy = false;
    }
  }

  async function toggle(rule: Rule): Promise<void> {
    status = null;
    try {
      await updateRule({ ...rule, enabled: !rule.enabled });
      await refresh();
    } catch (e) {
      status = `Error: ${String(e)}`;
    }
  }

  async function remove(id: string): Promise<void> {
    status = null;
    try {
      await deleteRule(id);
      await refresh();
    } catch (e) {
      status = `Error: ${String(e)}`;
    }
  }

  function kindLabel(rule: Rule): string {
    return KIND_OPTIONS.find((option) => option.value === rule.pattern.kind)?.label ??
      rule.pattern.kind;
  }

  function patternOf(rule: Rule): string {
    return patternText(rule.pattern);
  }
</script>

<div>
  <div class="flex items-center justify-between">
    <h2 class="text-sm font-medium text-zinc-300">Reglas de enrutamiento</h2>
    <span class="text-xs text-zinc-600">{rules.length} activa{rules.length === 1 ? "" : "s"}</span>
  </div>
  <p class="mt-1 text-xs text-zinc-500">
    Una regla mapea una URL a un perfil sin pedir confirmación. Menor número = mayor prioridad.
  </p>

  <form
    class="mt-4 space-y-2 rounded-lg border border-zinc-800 bg-zinc-950/60 p-3"
    onsubmit={(e) => {
      e.preventDefault();
      void add();
    }}
  >
    <div class="flex flex-wrap gap-2">
      <select
        class="rounded-md border border-zinc-700 bg-zinc-900 px-2 py-1.5 text-xs text-zinc-200"
        bind:value={kind}
      >
        {#each KIND_OPTIONS as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
      <input
        type="text"
        class="min-w-0 flex-1 rounded-md border border-zinc-700 bg-zinc-900 px-2 py-1.5 text-xs text-zinc-200 placeholder-zinc-600"
        placeholder={kindMeta.placeholder}
        bind:value={pattern}
      />
      <input
        type="number"
        min="0"
        class="w-16 rounded-md border border-zinc-700 bg-zinc-900 px-2 py-1.5 text-xs text-zinc-200"
        placeholder="prio"
        bind:value={priority}
      />
      <select
        class="max-w-[220px] rounded-md border border-zinc-700 bg-zinc-900 px-2 py-1.5 text-xs text-zinc-200"
        bind:value={targetProfileId}
      >
        <option value="" disabled>Perfil destino…</option>
        {#each profiles as profile (profile.id)}
          <option value={profile.id}>{profile.name}</option>
        {/each}
      </select>
      <button
        type="submit"
        class="rounded-md bg-zinc-100 px-3 py-1.5 text-xs font-medium text-zinc-900 hover:bg-white disabled:opacity-50"
        disabled={busy}
      >
        {busy ? "Guardando…" : "Agregar"}
      </button>
    </div>
  </form>

  {#if status}
    <p class="mt-3 text-xs text-red-400">{status}</p>
  {/if}

  <ul class="mt-4 space-y-2">
    {#each rules as rule (rule.id)}
      <li class="flex items-center gap-3 rounded-lg border border-zinc-800 bg-zinc-950/60 px-3 py-2">
        <span
          class="shrink-0 rounded bg-zinc-800 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider text-zinc-400"
        >
          {kindLabel(rule)}
        </span>
        <span class="min-w-0 flex-1">
          <span class="block truncate font-mono text-xs text-zinc-200">{patternOf(rule)}</span>
          <span class="block truncate text-[11px] text-zinc-500" title={rule.targetProfileId}>
            {rule.targetProfileId} · prio {rule.priority}
          </span>
        </span>
        <button
          type="button"
          class="rounded-md border border-zinc-700 px-2 py-1 text-[11px] {rule.enabled ? 'text-emerald-400' : 'text-zinc-500'}"
          onclick={() => toggle(rule)}
        >
          {rule.enabled ? "activa" : "pausada"}
        </button>
        <button
          type="button"
          class="rounded-md border border-zinc-700 px-2 py-1 text-[11px] text-zinc-400 hover:border-red-700 hover:text-red-400"
          onclick={() => remove(rule.id)}
        >
          eliminar
        </button>
      </li>
    {/each}
  </ul>
</div>