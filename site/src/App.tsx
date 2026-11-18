import { lazy, Match, Switch } from "solid-js";
import { path } from "./lib/router";
import Docs from "./pages/Docs";
import Landing from "./pages/Landing";

const Playground = lazy(() => import("./pages/Playground"));

export default function App() {
  return (
    <Switch fallback={<Landing />}>
      <Match when={path().startsWith("/docs")}><Docs /></Match>
      <Match when={path().startsWith("/playground")}><Playground /></Match>
    </Switch>
  );
}
