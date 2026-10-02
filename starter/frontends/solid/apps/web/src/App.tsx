import { createResource } from "solid-js";
import Notes from "./Notes"; // starter:example

async function fetchHealth(): Promise<string> {
  const res = await fetch("/api/health");
  return res.ok ? (await res.json()).status : `error ${res.status}`;
}

export default function App() {
  const [health] = createResource(fetchHealth);

  return (
    <main>
      <h1>Kudamerah</h1>
      <p>
        Server: <output>{health.error ? "unreachable" : (health() ?? "checking…")}</output>
      </p>
      <Notes /> {/* starter:example */}
    </main>
  );
}
