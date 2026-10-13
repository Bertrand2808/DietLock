// Raw UI: no framework, no bundler. Rules live in Rust.
const { invoke } = window.__TAURI__.core;

const $ = (id) => document.getElementById(id);
let pending = null; // save waiting for a replace confirmation: { command, entry, after }

function showError(message) {
  $("error").textContent = message;
  $("error").hidden = !message;
}

function weighInText(w) {
  if (!w) return "pas de pesée";
  return `${w.weight} kg${w.fasted ? "" : " (pas à jeun)"}`;
}

function intakeText(i) {
  if (!i) return "pas d'alimentation";
  const water = i.water === null ? "" : `, eau ${i.water} L`;
  return `${i.energyKcal} kcal (P ${i.proteinG} g, G ${i.carbsG} g, L ${i.fatG} g${water})`;
}

function render(listing) {
  $("date").max = listing.today;
  $("intake-date").max = listing.today;
  if (!$("date").value) $("date").value = listing.proposedDate;
  $("trend").textContent =
    listing.trend === null ? "pas encore de tendance" : `${listing.trend} kg`;
  $("days").replaceChildren(
    ...listing.days.map((d) => {
      const li = document.createElement("li");
      li.textContent = `${d.date} : ${weighInText(d.weighIn)} | ${intakeText(d.intake)}`;
      return li;
    }),
  );
}

async function refresh() {
  try {
    render(await invoke("list_days"));
    if (!$("intake-date").value) {
      $("intake-date").value = await invoke("proposed_intake_date");
    }
  } catch (e) {
    showError(String(e));
  }
}

function askReplace(text, command, entry, after) {
  pending = { command, entry, after };
  $("confirm-text").textContent = text;
  $("confirm").hidden = false;
}

async function save(command, entry, replace, after) {
  showError("");
  try {
    const outcome = await invoke(command, { ...entry, replace });
    if (outcome.status === "alreadyFilled") {
      const text =
        command === "save_weigh_in"
          ? `Le ${entry.date} a déjà une pesée : ${outcome.old.weight} kg. ` +
            `La remplacer par ${entry.weightText} kg ?`
          : `Le ${entry.date} a déjà une alimentation : ${intakeText(outcome.old)}. ` +
            `La remplacer ?`;
      askReplace(text, command, entry, after);
      return;
    }
    after();
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
    "save_weigh_in",
    {
      weightText: $("weight").value,
      fasted: $("fasted").checked,
      date: $("date").value,
    },
    false,
    () => {
      $("weight").value = "";
    },
  );
});

$("intake-form").addEventListener("submit", (event) => {
  event.preventDefault();
  closeConfirm();
  save(
    "save_intake",
    {
      proteinText: $("protein").value,
      carbsText: $("carbs").value,
      fatText: $("fat").value,
      waterText: $("water").value,
      date: $("intake-date").value,
    },
    false,
    () => {
      for (const id of ["protein", "carbs", "fat", "water"]) $(id).value = "";
    },
  );
});

$("confirm-replace").addEventListener("click", () => {
  const p = pending;
  closeConfirm();
  if (p) save(p.command, p.entry, true, p.after);
});

$("confirm-cancel").addEventListener("click", closeConfirm);

refresh();
