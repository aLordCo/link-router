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
  import { t, type MessageKey } from "../i18n";

  const KIND_OPTIONS: {
    value: RulePatternKind;
    labelKey: MessageKey;
    placeholder: string;
  }[] = [
    { value: "exact", labelKey: "rules.kindExact", placeholder: "github.com o https://github.com/x" },
    { value: "domain", labelKey: "rules.kindDomain", placeholder: "github.com o *.github.com" },
    { value: "regex", labelKey: "rules.kindRegex", placeholder: "^https://.*\\\\.notion\\\\.site/.+" },
    { value: "path", labelKey: "rules.kindPath", placeholder: "https://meet.google.com/*" },
    { value: "sourceApp", labelKey: "rules.kindSourceApp", placeholder: "Slack" },
  ];

  type Status = {
    key: MessageKey;
    vars?: Record<string, string>;
  };

  function statusText(status: Status | null): string {
    if (!status) return "";
    return status.vars ? t(status.key, status.vars) : t(status.key);
  }

  let rules = $state<Rule[]>([]);
  let profiles = $state<BrowserProfile[]>([]);
  let status = $state<Status | null>(null);
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
      status = { key: "common.error", vars: { e: String(e) } };
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
      status = { key: "rules.createError" };
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
      status = { key: "rules.createRuleError", vars: { e: String(e) } };
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
      status = { key: "common.error", vars: { e: String(e) } };
    }
  }

  async function remove(id: string): Promise<void> {
    status = null;
    try {
      await deleteRule(id);
      await refresh();
    } catch (e) {
      status = { key: "common.error", vars: { e: String(e) } };
    }
  }

  function kindLabel(rule: Rule): string {
    const option = KIND_OPTIONS.find((option) => option.value === rule.pattern.kind);
    return option ? t(option.labelKey) : rule.pattern.kind;
  }

  function patternOf(rule: Rule): string {
    return patternText(rule.pattern);
  }
</script>

<div>
  <div class="flex items-center justify-between">
    <h2 class="text-strong text-sm font-medium">{t("rules.title")}</h2>
    <span class="text-faint text-xs">
      {t(rules.length === 1 ? "rules.activeCountOne" : "rules.activeCountOther", {
        n: rules.length,
      })}
    </span>
  </div>
  <p class="text-muted mt-1 text-xs">
    {t("rules.hint")}
  </p>

  <form
    class="surface-deep mt-4 space-y-2 rounded-lg p-3"
    onsubmit={(e) => {
      e.preventDefault();
      void add();
    }}
  >
    <div class="flex flex-wrap gap-2">
      <select
        class="field rounded-md px-2 py-1.5 text-xs"
        bind:value={kind}
      >
        {#each KIND_OPTIONS as option (option.value)}
          <option value={option.value}>{t(option.labelKey)}</option>
        {/each}
      </select>
      <input
        type="text"
        class="field min-w-0 flex-1 rounded-md px-2 py-1.5 text-xs"
        placeholder={kindMeta.placeholder}
        bind:value={pattern}
      />
      <input
        type="number"
        min="0"
        class="field w-16 rounded-md px-2 py-1.5 text-xs"
        placeholder="prio"
        bind:value={priority}
      />
      <select
        class="field max-w-[220px] rounded-md px-2 py-1.5 text-xs"
        bind:value={targetProfileId}
      >
        <option value="" disabled>{t("rules.targetProfilePlaceholder")}</option>
        {#each profiles as profile (profile.id)}
          <option value={profile.id}>{profile.name}</option>
        {/each}
      </select>
      <button
        type="submit"
        class="btn-solid rounded-md px-3 py-1.5 text-xs font-medium disabled:opacity-50"
        disabled={busy}
      >
        {busy ? t("rules.saving") : t("rules.add")}
      </button>
    </div>
  </form>

  {#if status}
    <p class="text-danger mt-3 text-xs">{statusText(status)}</p>
  {/if}

  <ul class="mt-4 space-y-2">
    {#each rules as rule (rule.id)}
      <li class="surface-deep flex items-center gap-3 rounded-lg px-3 py-2">
        <span class="badge shrink-0 rounded px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider">
          {kindLabel(rule)}
        </span>
        <span class="min-w-0 flex-1">
          <span class="text-strong block truncate font-mono text-xs">{patternOf(rule)}</span>
          <span class="text-muted block truncate text-[11px]" title={rule.targetProfileId}>
            {rule.targetProfileId} · prio {rule.priority}
          </span>
        </span>
        <button
          type="button"
          class="btn-outline rounded-md px-2 py-1 text-[11px] {rule.enabled ? 'text-accent' : 'text-muted'}"
          onclick={() => toggle(rule)}
        >
          {rule.enabled ? t("rules.active") : t("rules.paused")}
        </button>
        <button
          type="button"
          class="btn-outline-danger rounded-md px-2 py-1 text-[11px]"
          onclick={() => remove(rule.id)}
        >
          {t("rules.delete")}
        </button>
      </li>
    {/each}
  </ul>
</div>