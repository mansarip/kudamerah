const status = document.querySelector("#status");

try {
  const res = await fetch("/api/health");
  status.value = res.ok ? (await res.json()).status : `error ${res.status}`;
} catch {
  status.value = "unreachable";
}
