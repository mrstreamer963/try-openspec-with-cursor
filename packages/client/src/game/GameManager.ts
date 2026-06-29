import type { IncomingEvent, StateSnapshot } from './types';
import type { MainToWorkerMessage, WorkerMessage } from './protocol';

type SnapshotListener = (snapshot: StateSnapshot) => void;
type ReadyListener = () => void;
type ErrorListener = (message: string) => void;

export class GameManager {
  private worker: Worker;
  private snapshotListeners = new Set<SnapshotListener>();
  private readyListeners = new Set<ReadyListener>();
  private errorListeners = new Set<ErrorListener>();
  private latestSnapshot: StateSnapshot | null = null;
  private _ready = false;
  private started = false;

  constructor() {
    this.worker = new Worker(new URL('../worker/gameWorker.ts', import.meta.url), {
      type: 'module',
    });
    this.worker.onmessage = (e: MessageEvent<WorkerMessage>) => {
      const msg = e.data;
      if (msg.kind === 'snapshot') {
        this.latestSnapshot = msg.data;
        this.snapshotListeners.forEach((fn) => fn(msg.data));
      } else if (msg.kind === 'ready') {
        this._ready = true;
        this.readyListeners.forEach((fn) => fn());
      } else if (msg.kind === 'error') {
        this.errorListeners.forEach((fn) => fn(msg.message));
      }
    };
    this.worker.onerror = (e) => {
      this.errorListeners.forEach((fn) => fn(e.message || 'Worker failed to load'));
    };
  }

  get ready(): boolean {
    return this._ready;
  }

  get snapshot(): StateSnapshot | null {
    return this.latestSnapshot;
  }

  onSnapshot(listener: SnapshotListener): () => void {
    this.snapshotListeners.add(listener);
    if (this.latestSnapshot) listener(this.latestSnapshot);
    return () => this.snapshotListeners.delete(listener);
  }

  onReady(listener: ReadyListener): () => void {
    this.readyListeners.add(listener);
    if (this._ready) listener();
    return () => this.readyListeners.delete(listener);
  }

  onError(listener: ErrorListener): () => void {
    this.errorListeners.add(listener);
    return () => this.errorListeners.delete(listener);
  }

  start(contentJson: string): void {
    if (this.started) return;
    this.started = true;
    this.worker.postMessage({ kind: 'start', contentJson } satisfies MainToWorkerMessage);
  }

  sendEvent(event: IncomingEvent): void {
    this.worker.postMessage({ kind: 'event', event } satisfies MainToWorkerMessage);
  }

  destroy(): void {
    this.worker.terminate();
  }
}
