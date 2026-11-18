// The interpreter outside the browser: for the example checks and the conformance tests.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { Interp } from "../src/engine/interp";
import { Plugin } from "../src/engine/wasm";

const root = fileURLToPath(new URL("../../", import.meta.url));
export async function loadInterp() {
  const plugin = await Plugin.load(readFileSync(root + "src/solveng.wasm"));
  return new Interp(plugin, {
    unitWords: readFileSync(root + "src/unit-words.typ", "utf8"),
    constants: readFileSync(root + "src/constants.typ", "utf8"),
  });
}
