// The page's side of the engine worker: one worker, requests answered in order, restarted when a document hangs.
import type { RunResult } from "./interp";

export type RunResponse = RunResult & { ms: number };

const TIMEOUT = 15000;
let worker: Worker | null = null;
let next = 1;
const pending = new Map<number, { resolve: (v: any) => void; reject: (e: Error) => void; timer: ReturnType<typeof setTimeout> }>();

function start(): Worker {
  const w = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
  w.onmessage = (e: MessageEvent) => {
    const { id, ok, result, error, ms } = e.data;
    const p = pending.get(id);
    if (!p) return;
    clearTimeout(p.timer);
    pending.delete(id);
    if (ok) p.resolve(Array.isArray(result?.blocks) ? { ...result, ms } : result);
    else p.reject(new Error(error));
  };
  return w;
}

function request<T>(msg: Record<string, unknown>): Promise<T> {
  worker ??= start();
  const id = next++;
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => {
      // a calculation that doesn't end: start over with a fresh engine
      worker?.terminate();
      worker = null;
      for (const [k, p] of pending) { clearTimeout(p.timer); p.reject(new Error("the calculation took too long and was stopped")); pending.delete(k); }
    }, TIMEOUT);
    pending.set(id, { resolve, reject, timer });
    worker!.postMessage({ id, ...msg });
  });
}

export const runDocument = (src: string) => request<RunResponse>({ kind: "run", src });
export const moduleNames = () => request<Record<string, string[]>>({ kind: "names" });
/** Starts the engine early, so the first example answers quickly. */
export const warmUp = () => { void runDocument(""); };
