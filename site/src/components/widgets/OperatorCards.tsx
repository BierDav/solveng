// The three operators of a worksheet, side by side, each with a tiny live example.
import { For } from "solid-js";
import Workbench from "../Workbench";

const CARDS = [
  {
    op: ":=", name: "defines", cls: "define",
    text: "Names a value or a function. Undefined names stay names until they get a value.",
    code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, V
#show: math
$R := 4.7 kΩ$, $U := 5 #V$`,
  },
  {
    op: "=>", name: "shows the value", cls: "value",
    text: "Calculates numerically and shows the result, in the unit you ask for.",
    code: `#import "@preview/solveng:3.0.0": *
#import units: kΩ, V, mA
#show: math
$R := 4.7 kΩ$, $U := 5 #V$
$U / R => mA$`,
  },
  {
    op: "->", name: "keeps it exact", cls: "symbolic",
    text: "Simplifies symbolically: fractions, roots and π stay. With solve(x) it solves.",
    code: `#import "@preview/solveng:3.0.0": *
#import modifiers: solve
#show: math
$(x^2 - 1)/(x - 1) ->$
$x^2 = 2 -> solve(x)$`,
  },
];

export default function OperatorCards() {
  return (
    <div class="op-cards">
      <For each={CARDS}>
        {c => (
          <div class={["op-card", c.cls]}>
            <div class="op-sym mono">{c.op}</div>
            <div class="op-name">{c.name}</div>
            <p>{c.text}</p>
            <Workbench example={{ code: c.code }} compact />
          </div>
        )}
      </For>
    </div>
  );
}
