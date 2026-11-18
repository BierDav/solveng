import { render } from "@solidjs/web";
import App from "./App";
import { warmUp } from "./engine/client";
import "./styles/global.css";
import "./styles/docs.css";
import "./styles/bench.css";
import "./styles/landing.css";

render(() => <App />, document.getElementById("root")!);
// the engine loads while the page settles, so the first example answers at once
setTimeout(warmUp, 300);
