// What the documentation is made of. Pages are data: prose in a small Markdown dialect (see `md` below), live
// examples that run in the browser on solveng's own engine, parameter tables, callouts and quizzes.
//
// Markdown dialect, in every `md` string:
//   paragraphs separated by a blank line · `- ` bullet lists · `1. ` numbered lists · `### ` subheadings
//   `code` · **bold** · *italic* · [text](/docs/units) links to pages (or https://… outside) · $…$ is NOT math here,
//   write formulas as `code` or in an example.

/** A control in an example: its value replaces {{name}} in the code, the example runs again as you move it. */
export type Control =
  | { kind: "slider"; name: string; label?: string; min: number; max: number; step: number; value: number; suffix?: string }
  | { kind: "select"; name: string; label?: string; options: string[]; value: string }
  | { kind: "toggle"; name: string; label?: string; value: boolean; on: string; off: string };

export type Example = {
  /** A complete solveng document, as Typst source: imports, `#show: math`, then the worksheet. */
  code: string;
  title?: string;
  /** md shown under the example */
  caption?: string;
  controls?: Control[];
  /** The example shows a problem on purpose (it is checked to report one). */
  expectError?: boolean;
};

export type Param = { name: string; type: string; default?: string; md: string };

export type Quiz = {
  question: string;          // md
  code?: string;             // a document to show (not run) with the question
  options: string[];         // md each
  answer: number;            // index into options
  explain: string;           // md, shown after answering
};

export type Block =
  | { md: string }
  | { h2: string }
  | { h3: string }
  | { example: Example }
  | { code: string; lang?: string }
  | { callout: "tip" | "note" | "warning" | "fun"; title?: string; md: string }
  | { params: Param[] }
  | { table: { head: string[]; rows: string[][] } }
  | { quiz: Quiz }
  | { widget: "unit-table" | "option-explorer" | "operator-cards"; props?: Record<string, unknown> };

export type Group = "start" | "guides" | "functions" | "modifiers" | "operations" | "reference";

export type Page = {
  /** The path under /docs/: "units", "functions/graph", "operations/tri" */
  slug: string;
  title: string;
  group: Group;
  /** A heading within the group in the sidebar: "Signals" for the signal operations */
  section?: string;
  /** One sentence, md */
  summary: string;
  /** For functions and operations: how it's called, e.g. "tri(x)" or "graph(equation, x-range: auto, …)" */
  signature?: string;
  /** The import that brings it, e.g. "#import operations: tri" */
  import?: string;
  returns?: string;
  blocks: Block[];
  /** slugs of related pages */
  related?: string[];
  /** extra words for the search */
  keywords?: string[];
  /** a small emoji or symbol for cards and the sidebar of guides */
  icon?: string;
};
