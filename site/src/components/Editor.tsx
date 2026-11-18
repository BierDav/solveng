// A code editor for Typst: a textarea over its highlighted copy, so typing stays native (undo, IME, mobile).
import { createMemo } from "solid-js";
import { highlight } from "../lib/highlight";

export default function Editor(props: {
  value: string;
  onInput: (v: string) => void;
  minRows?: number;
  class?: string;
  label?: string;
  readOnly?: boolean;
}) {
  const html = createMemo(() => highlight(props.value) + "\n");
  const lines = createMemo(() => props.value.split("\n").length);
  let ta!: HTMLTextAreaElement;

  const onKeyDown = (e: KeyboardEvent) => {
    if (props.readOnly) return;
    // Tab indents, as in an editor; Escape lets focus move on
    if (e.key === "Tab" && !e.shiftKey) {
      e.preventDefault();
      const { selectionStart: s, selectionEnd: t, value } = ta;
      const next = value.slice(0, s) + "  " + value.slice(t);
      ta.value = next;
      ta.selectionStart = ta.selectionEnd = s + 2;
      props.onInput(next);
    } else if (e.key === "Escape") ta.blur();
  };

  return (
    <div class={["editor", props.class]} style={{ "--rows": String(Math.max(props.minRows ?? 3, lines())) }}>
      <div class="editor-gutter" aria-hidden="true">
        {Array.from({ length: lines() }, (_, i) => i + 1).join("\n")}
      </div>
      <div class="editor-body">
        <pre class="editor-hl" aria-hidden="true" innerHTML={html()} />
        <textarea
          ref={el => (ta = el)}
          class="editor-input"
          spellcheck={false}
          autocapitalize="off"
          autocomplete="off"
          aria-label={props.label ?? "Typst source"}
          readonly={props.readOnly}
          value={props.value}
          onInput={e => props.onInput(e.currentTarget.value)}
          onKeyDown={onKeyDown}
        />
      </div>
    </div>
  );
}
