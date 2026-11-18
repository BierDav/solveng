// A small CBOR codec: the plugin takes and gives CBOR, as Typst's `cbor.encode(…)` / `cbor(…)` do.

export type CborValue = null | boolean | number | string | Uint8Array | CborValue[] | { [k: string]: CborValue };

/** A number that goes as a float, also when it's whole (Typst's float(…)). */
export class Float { constructor(public v: number) {} }

export function encode(value: unknown): Uint8Array {
  const out: number[] = [];
  const head = (major: number, n: number) => {
    if (n < 24) out.push((major << 5) | n);
    else if (n < 0x100) out.push((major << 5) | 24, n);
    else if (n < 0x10000) out.push((major << 5) | 25, n >> 8, n & 255);
    else if (n < 0x100000000) out.push((major << 5) | 26, (n >>> 24) & 255, (n >> 16) & 255, (n >> 8) & 255, n & 255);
    else {
      const hi = Math.floor(n / 0x100000000), lo = n >>> 0;
      out.push((major << 5) | 27, (hi >>> 24) & 255, (hi >> 16) & 255, (hi >> 8) & 255, hi & 255, (lo >>> 24) & 255, (lo >> 16) & 255, (lo >> 8) & 255, lo & 255);
    }
  };
  const bytes = (b: Uint8Array) => { for (const x of b) out.push(x); };
  const enc = (v: unknown): void => {
    if (v === null || v === undefined) out.push(0xf6);
    else if (v === true) out.push(0xf5);
    else if (v === false) out.push(0xf4);
    else if (typeof v === "number" || v instanceof Float) {
      const float = v instanceof Float;
      if (float) v = (v as Float).v;
      const n = v as number;
      if (!float && Number.isInteger(n) && Math.abs(n) < 2 ** 53) n >= 0 ? head(0, n) : head(1, -1 - n);
      else {
        const dv = new DataView(new ArrayBuffer(8));
        dv.setFloat64(0, n);
        out.push(0xfb);
        bytes(new Uint8Array(dv.buffer));
      }
    } else if (typeof v === "string") {
      const b = new TextEncoder().encode(v);
      head(3, b.length);
      bytes(b);
    } else if (v instanceof Uint8Array) {
      head(2, v.length);
      bytes(v);
    } else if (Array.isArray(v)) {
      head(4, v.length);
      v.forEach(enc);
    } else if (typeof v === "object") {
      const entries = Object.entries(v as object).filter(([, x]) => x !== undefined);
      head(5, entries.length);
      for (const [k, x] of entries) { enc(k); enc(x); }
    } else throw new Error(`cbor: can't encode ${typeof v}`);
  };
  enc(value);
  return new Uint8Array(out);
}

export function decode(b: Uint8Array): any {
  let p = 0;
  const dv = new DataView(b.buffer, b.byteOffset, b.byteLength);
  const len = (ai: number): number => {
    if (ai < 24) return ai;
    if (ai === 24) return b[p++];
    if (ai === 25) { const n = dv.getUint16(p); p += 2; return n; }
    if (ai === 26) { const n = dv.getUint32(p); p += 4; return n; }
    if (ai === 27) { const n = dv.getUint32(p) * 0x100000000 + dv.getUint32(p + 4); p += 8; return n; }
    throw new Error("cbor: indefinite lengths aren't supported");
  };
  const half = (h: number) => {
    const s = h >> 15 ? -1 : 1, e = (h >> 10) & 31, f = h & 1023;
    return e === 0 ? s * 2 ** -14 * (f / 1024) : e === 31 ? (f ? NaN : s * Infinity) : s * 2 ** (e - 15) * (1 + f / 1024);
  };
  const dec = (): any => {
    const ib = b[p++], major = ib >> 5, ai = ib & 31;
    switch (major) {
      case 0: return len(ai);
      case 1: return -1 - len(ai);
      case 2: { const n = len(ai); const r = b.slice(p, p + n); p += n; return r; }
      case 3: { const n = len(ai); const r = new TextDecoder().decode(b.subarray(p, p + n)); p += n; return r; }
      case 4: { const n = len(ai); const a = []; for (let i = 0; i < n; i++) a.push(dec()); return a; }
      case 5: { const n = len(ai); const o: Record<string, any> = {}; for (let i = 0; i < n; i++) { const k = dec(); o[String(k)] = dec(); } return o; }
      case 6: len(ai); return dec();
      case 7:
        if (ai === 20) return false;
        if (ai === 21) return true;
        if (ai === 22 || ai === 23) return null;
        if (ai === 25) { const r = half(dv.getUint16(p)); p += 2; return r; }
        if (ai === 26) { const r = dv.getFloat32(p); p += 4; return r; }
        if (ai === 27) { const r = dv.getFloat64(p); p += 8; return r; }
    }
    throw new Error(`cbor: unexpected byte ${ib}`);
  };
  return dec();
}
