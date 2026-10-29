/**
 * Entry point: initialize WASM once in the MAIN context (errors propagate —
 * no render on failure), then mount the Solid 2 app.
 */
import { render } from "@solidjs/web";
import { App } from "./ui/App";
import { wasmReady } from "./wasmSetup";
import "./styles.css";

async function boot(): Promise<void> {
  const root = document.getElementById("root");
  if (!root) throw new Error("#root missing");
  try {
    const tables = await wasmReady();
    render(() => <App tables={tables} />, root);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    root.textContent = "";
    const p = document.createElement("p");
    p.className = "boot-error";
    p.setAttribute("role", "alert");
    p.textContent = `Failed to initialize WebAssembly: ${msg}`;
    root.appendChild(p);
    throw e; // still surfaces to console/network tooling
  }
}

void boot();
