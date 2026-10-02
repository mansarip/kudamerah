// Example: talks to the JSON API in apps/server/src/modules/notes.rs.

import { createResource, For } from "solid-js";

type Note = { id: number; body: string; created_at: string };

async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, init);
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    throw new Error(body?.error ?? `HTTP ${res.status}`);
  }
  return res.status === 204 ? (undefined as T) : res.json();
}

export default function Notes() {
  const [notes, { mutate }] = createResource(() => api<Note[]>("/api/notes"));

  async function add(event: SubmitEvent & { currentTarget: HTMLFormElement }) {
    event.preventDefault();
    const form = event.currentTarget;
    try {
      const note = await api<Note>("/api/notes", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ body: new FormData(form).get("body") }),
      });
      mutate((list) => [note, ...(list ?? [])]);
      form.reset();
    } catch (err) {
      alert((err as Error).message);
    }
  }

  async function remove(id: number) {
    await api<void>(`/api/notes/${id}`, { method: "DELETE" });
    mutate((list) => list?.filter((note) => note.id !== id));
  }

  return (
    <section>
      <h2>Notes</h2>
      <form onSubmit={add}>
        <input name="body" required maxLength={1000} placeholder="Write a note…" aria-label="Note" />
        <button>Add</button>
      </form>
      <ul>
        <For each={notes()}>
          {(note) => (
            <li>
              {note.body} <button onClick={() => remove(note.id)}>Delete</button>
            </li>
          )}
        </For>
      </ul>
    </section>
  );
}
