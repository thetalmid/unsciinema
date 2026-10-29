/** Shared handle to the live renderer (bound by PreviewPane lifecycle). */
import type { PreviewManager } from "./preview";

let active: PreviewManager | null = null;

export function setActiveManager(m: PreviewManager | null): void {
  active = m;
}

export function getActiveManager(): PreviewManager | null {
  return active;
}
