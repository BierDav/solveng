// The engine in a worker: documents are calculated off the page's thread.
import wasmUrl from "../../../src/solveng.wasm?url";
import unitWords from "../../../src/unit-words.typ?raw";
import constants from "../../../src/constants.typ?raw";
import { Interp } from "./interp";
import { Plugin } from "./wasm";

const ready = (async () => {
  const bytes = await (await fetch(wasmUrl)).arrayBuffer();
  const plugin = await Plugin.load(bytes);
  return new Interp(plugin, { unitWords, constants });
})();

export type Request = { id: number; kind: "run"; src: string } | { id: number; kind: "names" };

self.onmessage = async (e: MessageEvent<Request>) => {
  const req = e.data;
  try {
    const it = await ready;
    const t0 = performance.now();
    const result = req.kind === "run" ? it.run(req.src) : it.moduleNames();
    self.postMessage({ id: req.id, ok: true, result, ms: performance.now() - t0 });
  } catch (err) {
    self.postMessage({ id: req.id, ok: false, error: String((err as Error).message ?? err) });
  }
};
