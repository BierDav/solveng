// The options that change how numbers look, all on one worksheet: move them and watch the same results change.
import Workbench from "../Workbench";

const CODE = `#import "@preview/solveng:3.0.0": *
#import units: V, mA, kΩ, uF, ms, mW
#show: math.with(
  digits: {{digits}},
  trailing-zeros: {{zeros}},
  notation: "{{notation}}",
  number-form: "{{form}}",
  fraction-style: "{{fractions}}",
  decimal-separator: "{{separator}}",
  multiplication: "{{multiplication}}",
  show-approximation: {{approx}},
)

$U := 12 #V$, $R := 4.7 kΩ$, $C := 2.2 uF$
$I := U / R => mA$
$tau := R C => ms$
$P := U^2 / R => mW$
$x := 1/3 + 1/7 =>$
$y := 123456.789 =>$, $z := 0.000042 =>$`;

export default function OptionExplorer() {
  return (
    <Workbench
      example={{
        title: "Option explorer",
        code: CODE,
        controls: [
          { kind: "slider", name: "digits", label: "digits", min: 1, max: 12, step: 1, value: 4 },
          { kind: "select", name: "notation", label: "notation", options: ["auto", "plain", "scientific", "engineering"], value: "auto" },
          { kind: "select", name: "form", label: "number-form", options: ["auto", "decimal", "fraction", "input"], value: "auto" },
          { kind: "select", name: "fractions", label: "fraction-style", options: ["stacked", "slash"], value: "stacked" },
          { kind: "select", name: "separator", label: "decimal-separator", options: [".", ","], value: "." },
          { kind: "select", name: "multiplication", label: "multiplication", options: ["dot", "times"], value: "dot" },
          { kind: "toggle", name: "zeros", label: "trailing-zeros", value: false, on: "true", off: "false" },
          { kind: "toggle", name: "approx", label: "show-approximation", value: true, on: "true", off: "false" },
        ],
        caption: "Every control is an option of `math`; the code above shows exactly what it sets.",
      }}
    />
  );
}
