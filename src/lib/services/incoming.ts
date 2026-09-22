import { listen } from "@tauri-apps/api/event";
import { getPendingUrls } from "./deepLink";
import { appState, ui } from "../stores/appState.svelte";

const INCOMING_URL_EVENT = "dispatch://incoming-url";

export async function watchIncomingUrls(): Promise<void> {
  try {
    const pending = await getPendingUrls();
    for (const url of pending) {
      ui.page = "prompter";
      void appState.evaluate(url);
    }
  } catch {
    // sin deep-link pendiente: no es un error
  }

  await listen<{ url: string; sourceApp: string | null }>(
    INCOMING_URL_EVENT,
    (event) => {
      ui.page = "prompter";
      void appState.evaluate(event.payload.url, event.payload.sourceApp);
    },
  );
}