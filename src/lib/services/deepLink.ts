import { invoke } from "@tauri-apps/api/core";
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import type { UnlistenFn } from "@tauri-apps/api/event";

export type OpenUrlHandler = (urls: string[]) => void;

export async function getPendingUrls(): Promise<string[]> {
  const urls = await getCurrent();
  return (urls ?? []).map((u) => u.toString());
}

export function onUrlOpened(handler: OpenUrlHandler): Promise<UnlistenFn> {
  return onOpenUrl((urls) => handler(urls.map((u) => u.toString())));
}

export async function registerScheme(scheme: string): Promise<void> {
  await invoke("register_scheme", { scheme });
}

export function isSchemeRegistered(scheme: string): Promise<boolean> {
  return invoke("check_scheme", { scheme });
}