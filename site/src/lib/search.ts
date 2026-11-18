// Search over every page: titles, signatures, summaries, headings and keywords, ranked by where the words match.
import { GROUPS, pages } from "../content";
import type { Page } from "../content/types";
import { plainText } from "./md";

export type Hit = { page: Page; anchor?: string; label: string; detail: string; group: string; score: number };

type Entry = { page: Page; anchor?: string; label: string; detail: string; title: string; text: string; boost: number };

let index: Entry[] | null = null;
function build(): Entry[] {
  const groupTitle = new Map(GROUPS.map(g => [g.id, g.title]));
  const out: Entry[] = [];
  for (const p of pages) {
    const summary = plainText(p.summary);
    const name = p.slug.split("/").pop()!;
    out.push({
      page: p, label: p.title, detail: summary, boost: 3,
      title: (p.title + " " + name).toLowerCase(),
      text: [summary, p.signature ?? "", p.import ?? "", ...(p.keywords ?? []), groupTitle.get(p.group) ?? "", p.section ?? ""].join(" ").toLowerCase(),
    });
    for (const b of p.blocks) {
      const h = "h2" in b ? b.h2 : "h3" in b ? b.h3 : null;
      if (h) out.push({ page: p, anchor: h, label: `${p.title} › ${plainText(h)}`, detail: "", title: plainText(h).toLowerCase(), text: "", boost: 1 });
    }
  }
  return out;
}

export function search(q: string, limit = 30): Hit[] {
  index ??= build();
  const words = q.toLowerCase().trim().split(/\s+/).filter(Boolean);
  if (!words.length) return [];
  const groupTitle = new Map(GROUPS.map(g => [g.id, g.title]));
  const hits: Hit[] = [];
  for (const e of index) {
    let score = 0;
    let all = true;
    for (const w of words) {
      const t = e.title.indexOf(w);
      if (e.title === w || e.title.startsWith(w + " ")) score += 12;
      else if (t === 0) score += 8;
      else if (t > 0) score += 5;
      else if (e.text.includes(w)) score += 1.5;
      else { all = false; break; }
    }
    if (!all) continue;
    hits.push({ page: e.page, anchor: e.anchor, label: e.label, detail: e.detail, group: e.page.section ?? groupTitle.get(e.page.group) ?? "", score: score * e.boost - e.label.length * 0.01 });
  }
  return hits.sort((a, b) => b.score - a.score).slice(0, limit);
}
