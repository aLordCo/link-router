import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";
import { cachedTheme, systemPrefersDark } from "./lib/theme";

document.documentElement.dataset.theme =
  cachedTheme() ?? (systemPrefersDark() ? "dark" : "light");

const target = document.getElementById("app");

if (!target) {
  throw new Error("Root element #app not found");
}

const app = mount(App, { target });

export default app;