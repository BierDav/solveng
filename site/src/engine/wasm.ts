// solveng's engine, the very plugin Typst loads: a host for Typst's wasm-minimal-protocol.
import { decode, encode } from "./cbor";

export class Plugin {
  private instance!: WebAssembly.Instance;
  private args: Uint8Array[] = [];
  private result: Uint8Array | null = null;

  private constructor(private module: WebAssembly.Module) {
    this.instantiate();
  }

  static async load(bytes: BufferSource): Promise<Plugin> {
    return new Plugin(await WebAssembly.compile(bytes));
  }

  private instantiate() {
    const mem = () => new Uint8Array((this.instance.exports.memory as WebAssembly.Memory).buffer);
    this.instance = new WebAssembly.Instance(this.module, {
      typst_env: {
        wasm_minimal_protocol_write_args_to_buffer: (ptr: number) => {
          let at = ptr;
          for (const a of this.args) { mem().set(a, at); at += a.length; }
        },
        wasm_minimal_protocol_send_result_to_host: (ptr: number, len: number) => {
          this.result = mem().slice(ptr, ptr + len);
        },
      },
    });
  }

  /** Calls a plugin function as Typst's `_call` does: bytes pass as they are, anything else as CBOR. */
  call(name: string, ...args: unknown[]): any {
    const f = this.instance.exports[name] as ((...lens: number[]) => number) | undefined;
    if (!f) throw new Error(`solveng: the engine has no function ${name}`);
    this.args = args.map(a => (a instanceof Uint8Array ? a : encode(a)));
    this.result = null;
    let code: number;
    try {
      code = f(...this.args.map(a => a.length));
    } catch (e) {
      // a panic leaves the instance's memory half-done: the next call starts on a fresh one (the worksheet's
      // definitions live on this side, as bytes)
      this.instantiate();
      throw new Error(`solveng: the engine stopped (${(e as Error).message})`);
    }
    const r = this.result ?? new Uint8Array();
    if (code !== 0) throw new Error(new TextDecoder().decode(r));
    return decode(r);
  }
}
