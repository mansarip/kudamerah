// Example: talks to the JSON API in apps/server/src/modules/notes.rs.

const form = document.querySelector("#note-form");
const list = document.querySelector("#notes");

async function api(path, options) {
  const res = await fetch(path, options);
  if (!res.ok) throw new Error((await res.json().catch(() => null))?.error ?? `HTTP ${res.status}`);
  return res.status === 204 ? null : res.json();
}

function item(note) {
  const li = document.createElement("li");
  const remove = document.createElement("button");
  remove.textContent = "Delete";
  remove.addEventListener("click", async () => {
    await api(`/api/notes/${note.id}`, { method: "DELETE" });
    li.remove();
  });
  li.append(note.body, " ", remove);
  return li;
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const note = await api("/api/notes", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ body: form.elements.body.value }),
    });
    list.prepend(item(note));
    form.reset();
  } catch (err) {
    alert(err.message);
  }
});

list.replaceChildren(...(await api("/api/notes")).map(item));
