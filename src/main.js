// Raw UI of slice 1: no framework, no bundler. Rules live in Rust.
const { invoke } = window.__TAURI__.core;

const $ = (id) => document.getElementById(id);
let pending = null; // weigh-in waiting for a replace confirmation

function showError(message) {
  $("error").textContent = message;
  $("error").hidden = !message;
}

function render(listing) {
  $("date").max = listing.today;
  if (!$("date").value) $("date").value = listing.proposedDate;
  $("trend").textContent =
    listing.trend === null ? "pas encore de tendance" : `${listing.trend} kg`;
  const list = $("weigh-ins");
  list.replaceChildren(
    ...listing.weighIns.map((w) => {
      const li = document.createElement("li");
      li.textContent = `${w.date} : ${w.weight} kg${w.fasted ? "" : " (pas à jeun)"}`;
      return li;
    }),
  );
}

async function refresh() {
  try {
    render(await invoke("list_weigh_ins"));
  } catch (e) {
    showError(String(e));
  }
}

async function save(entry, replace) {
  showError("");
  try {
    const outcome = await invoke("save_weigh_in", { ...entry, replace });
    if (outcome.status === "alreadyFilled") {
      pending = entry;
      $("confirm-text").textContent =
        `Le ${entry.date} a déjà une pesée : ${outcome.old.weight} kg. ` +
        `La remplacer par ${entry.weightText} kg ?`;
      $("confirm").hidden = false;
      return;
    }
    $("weight").value = "";
    await refresh();
  } catch (e) {
    showError(String(e));
  }
}

function closeConfirm() {
  pending = null;
  $("confirm").hidden = true;
}

$("weigh-in-form").addEventListener("submit", (event) => {
  event.preventDefault();
  closeConfirm();
  save(
    {
      weightText: $("weight").value,
      fasted: $("fasted").checked,
      date: $("date").value,
    },
    false,
  );
});

$("confirm-replace").addEventListener("click", () => {
  const entry = pending;
  closeConfirm();
  if (entry) save(entry, true);
});

$("confirm-cancel").addEventListener("click", closeConfirm);

refresh();
