<script lang="ts">
  import { fly } from "svelte/transition";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { BrowserProfile } from "../services/tauri";

  let { profile, index, selected, onSelect, onPick } = $props<{
    profile: BrowserProfile;
    index: number;
    selected: boolean;
    onSelect: (index: number) => void;
    onPick: (profileId: string) => void;
  }>();

  function renderableIcon(icon: string | null): string | null {
    if (!icon || !icon.startsWith("/")) return null;
    if (/\.(png|jpe?g|gif|svg|webp|ico)$/i.test(icon)) {
      return convertFileSrc(icon);
    }
    return null;
  }

  const iconUrl = $derived(renderableIcon(profile.icon));
  const hint = $derived(index < 9 ? String(index + 1) : "");
</script>

<button
  type="button"
  id={`profile-opt-${index}`}
  role="option"
  aria-selected={selected}
  class="palette-item flex w-full cursor-default items-center gap-3 rounded-xl px-3 py-1.5 text-left transition-colors duration-75 {selected ? 'palette-item-selected' : ''}"
  onmouseenter={() => onSelect(index)}
  onclick={() => onPick(profile.id)}
  transition:fly={{ y: 4, duration: 60 }}
>
  {#if iconUrl}
    <img src={iconUrl} alt="" class="h-7 w-7 shrink-0 rounded-md" />
  {:else}
    <span
      class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-white/10 text-xs font-semibold"
      style="color:{selected ? 'inherit' : 'var(--p-muted)'}"
    >
      {profile.browserName.slice(0, 1).toUpperCase()}
    </span>
  {/if}
  <span class="flex min-w-0 flex-1 flex-col">
    <span class="truncate text-[13px] font-medium leading-snug">{profile.name}</span>
    <span class="palette-muted truncate text-[11px] leading-snug">{profile.browserName}</span>
  </span>
  {#if hint}
    <span class="palette-keycap shrink-0">{hint}</span>
  {/if}
</button>