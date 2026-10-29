/**
 * Main-thread wrapper around the converter worker.
 *
 * Cancellation model (mission §7):
 * - generation ids: every convert bumps `convertGen`; responses with older
 *   ids are discarded (stale results never reach the preview);
 * - cooperative cancel for long jobs (GIF): postMessage `cancel`, worker
 *   checks it between frames;
 * - hard cancel: if the worker does not acknowledge a cancel within
 *   `HARD_CANCEL_MS`, the worker is terminated and restarted, the wasm is
 *   re-initialized in the new context, and the retained source bytes are
 *   re-sent — a genuinely interruptible pipeline.
 * - the main thread keeps a copy of the original source bytes precisely so
 *   restarts never need the user's file again.
 */
import type {
  ExportRequest,
  FromWorker,
  ParamsDTO,
  ToWorker,
} from "./protocol";

const HARD_CANCEL_MS = 350;

export interface ClientCallbacks {
  onLoaded(id: number, info: Extract<FromWorker, { t: "loaded" }>["info"]): void;
  onResult(msg: Extract<FromWorker, { t: "result" }>): void;
  onProgress(id: number, done: number, total: number): void;
  onExported(msg: Extract<FromWorker, { t: "exported" }>): void;
  onCancelled(id: number): void;
  onError(id: number, message: string): void;
}

export class WorkerClient {
  private worker: Worker;
  private idGen = 0;
  /** Responses with id <= epoch belong to a dead worker incarnation. */
  private epoch = 0;
  private convertGen = 0;
  private activeJob: { id: number; kind: "gif" | "export" } | null = null;
  private hardCancelTimer: ReturnType<typeof setTimeout> | null = null;
  private retainedSource: Uint8Array | null = null;
  private disposed = false;

  constructor(private callbacks: ClientCallbacks) {
    this.worker = this.spawn();
  }

  private spawn(): Worker {
    const w = new Worker(new URL("./worker.ts", import.meta.url), {
      type: "module",
    });
    w.onmessage = (e: MessageEvent<FromWorker>) => this.route(e.data);
    w.onerror = (e) => {
      // Worker-level failures (syntax/import) must surface, not hang.
      this.callbacks.onError(
        this.convertGen,
        `worker error: ${e.message ?? "unknown"}`,
      );
    };
    return w;
  }

  private route(msg: FromWorker): void {
    if ("id" in msg && msg.id <= this.epoch) return; // stale incarnation
    switch (msg.t) {
      case "loaded":
        this.callbacks.onLoaded(msg.id, msg.info);
        break;
      case "result":
        if (msg.id < this.convertGen) return; // superseded by a newer convert
        this.callbacks.onResult(msg);
        break;
      case "progress":
        this.callbacks.onProgress(msg.id, msg.done, msg.total);
        break;
      case "exported":
        if (this.activeJob?.id === msg.id) this.activeJob = null;
        this.clearHardCancel();
        this.callbacks.onExported(msg);
        break;
      case "cancelled":
        if (this.activeJob?.id === msg.id) this.activeJob = null;
        this.clearHardCancel();
        this.callbacks.onCancelled(msg.id);
        break;
      case "error":
        if (msg.id === -1) return; // (no startup sentinel used; guard anyway)
        if (this.activeJob?.id === msg.id) this.activeJob = null;
        this.clearHardCancel();
        this.callbacks.onError(msg.id, msg.message);
        break;
    }
  }

  private post(msg: ToWorker, transfer: Transferable[] = []): void {
    this.worker.postMessage(msg, transfer);
  }

  private nextId(): number {
    this.idGen += 1;
    return this.idGen;
  }

  private clearHardCancel(): void {
    if (this.hardCancelTimer) {
      clearTimeout(this.hardCancelTimer);
      this.hardCancelTimer = null;
    }
  }

  /** Load (or reload after restart) the immutable source. */
  load(bytes: Uint8Array): number {
    this.retainedSource = bytes.slice();
    const id = this.nextId();
    const payload = bytes.slice();
    this.post({ t: "load", id, bytes: payload }, [
      payload.buffer as ArrayBuffer,
    ]);
    return id;
  }

  /** Re-send the retained source after a hard restart. */
  private reloadRetained(): number | null {
    if (!this.retainedSource) return null;
    const id = this.nextId();
    const payload = this.retainedSource.slice();
    this.post({ t: "load", id, bytes: payload }, [
      payload.buffer as ArrayBuffer,
    ]);
    return id;
  }

  /** Request a conversion; returns the generation id. */
  convert(frame: number, params: ParamsDTO): number {
    const id = this.nextId();
    this.convertGen = id;
    this.post({ t: "convert", id, frame, params });
    return id;
  }

  /** Cooperative cancel of an in-flight GIF/export with hard-cancel backup. */
  cancelActiveJob(): void {
    const job = this.activeJob;
    if (!job) return;
    this.post({ t: "cancel", id: job.id });
    this.clearHardCancel();
    this.hardCancelTimer = setTimeout(() => {
      // Worker did not acknowledge in time → interrupt for real.
      this.hardRestart();
      this.callbacks.onCancelled(job.id);
    }, HARD_CANCEL_MS);
  }

  /** Terminate + restart + resend source. Used for stuck jobs and errors. */
  hardRestart(): void {
    if (this.disposed) return;
    this.epoch = this.idGen;
    this.clearHardCancel();
    this.worker.terminate();
    this.worker = this.spawn();
    this.activeJob = null;
    this.reloadRetained();
  }

  startGifExport(params: ParamsDTO, still: boolean): number {
    const id = this.nextId();
    this.activeJob = { id, kind: "gif" };
    this.post({ t: "gifExport", id, params, still });
    return id;
  }

  requestExport(msg: ExportRequest): number {
    const id = this.nextId();
    const withId = { ...msg, id } as ToWorker;
    const transfer: Transferable[] = [];
    if (withId.t === "exportRaster" && withId.rgba) {
      transfer.push(withId.rgba.buffer as ArrayBuffer);
    }
    if (withId.t === "exportHtml" && withId.woff) {
      transfer.push(withId.woff.buffer as ArrayBuffer);
    }
    if (withId.t !== "exportText") {
      this.activeJob = { id, kind: "export" };
    }
    this.post(withId, transfer);
    return id;
  }

  /** Forget the retained source (user cleared it); no reload on restart. */
  clearRetained(): void {
    this.retainedSource = null;
  }

  dispose(): void {
    this.disposed = true;
    this.clearHardCancel();
    this.worker.terminate();
  }
}
