import { savesOnEnter } from "./keys.js";
import { saveBlocks } from "./save.js";

const { invoke } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);

let weighDate = "";
let intakeDate = "";
let weighLabel = "";
let intakeLabel = "";
let seenWeighDate = null;
let seenIntakeDate = null;
let queue = [];
let busy = false;

function errorText(e) {
  if (typeof e === "string") return e;
  if (e && typeof e.message === "string") return e.message;
  return String(e);
}

function groupKcal(n) {
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, "\u202f");
}

function setChip(id, when, label) {
  const el = $(id);
  el.replaceChildren();
  const strong = document.createElement("b");
  strong.textContent = when;
  el.append(strong, ` · ${label}`);
}

function setSubtitle(text) {
  const sub = $("subhead");
  sub.replaceChildren();
  const kcal = text.indexOf("kcal");
  if (kcal === -1) {
    sub.textContent = text;
    return;
  }
  const strong = document.createElement("b");
  strong.textContent = text.slice(0, kcal + 4);
  sub.append(strong, text.slice(kcal + 4));
}

function renderHeadline(listing) {
  const headline = $("headline");
  headline.replaceChildren();
  if (listing.isTrend) {
    headline.classList.remove("greet");
    const num = document.createElement("span");
    num.className = "num";
    num.textContent = listing.primary;
    const unit = document.createElement("small");
    unit.textContent = " kg de tendance";
    headline.append(num, unit);
  } else {
    headline.textContent = listing.primary;
    headline.classList.toggle("greet", listing.primary.length > 0);
  }
  setSubtitle(listing.subtitle);
}

function renderHistory(days) {
  const hist = $("hist");
  const rows = days.filter((d) => d.weightLabel || d.detail);
  hist.replaceChildren();
  if (rows.length === 0) {
    hist.classList.remove("fade");
    const empty = document.createElement("p");
    empty.className = "empty";
    empty.append("Ta première journée apparaîtra ici.");
    empty.append(document.createElement("br"));
    empty.append("Commence par ton poids de ce matin, juste au-dessus.");
    hist.append(empty);
    return;
  }
  hist.classList.add("fade");
  const title = document.createElement("h2");
  title.textContent = "30 derniers jours";
  hist.append(title);
  for (const day of rows) {
    const row = document.createElement("div");
    row.className = "day";
    const date = document.createElement("span");
    date.className = "dt";
    date.textContent = day.label;
    const weight = document.createElement("span");
    weight.className = day.alert ? "w alert" : "w";
    weight.textContent = day.weightLabel ? (day.alert ? `${day.weightLabel} ⚠︎` : day.weightLabel) : "";
    if (day.alert) weight.title = "Écart d'au moins 3 kg avec la tendance de la veille.";
    const detail = document.createElement("span");
    detail.className = "k";
    detail.textContent = day.detail ?? "";
    row.append(date, weight, detail);
    hist.append(row);
  }
}

function render(listing) {
  if (seenIntakeDate !== null && listing.intakeDate !== seenIntakeDate) {
    for (const id of ["protein", "carbs", "fat", "water"]) $(id).value = "";
    $("saved").hidden = true;
  }
  if (seenWeighDate !== null && listing.weighInDate !== seenWeighDate) {
    $("weight").value = "";
    $("fasted").checked = true;
    $("saved").hidden = true;
  }
  seenIntakeDate = listing.intakeDate;
  seenWeighDate = listing.weighInDate;
  weighDate = listing.weighInDate;
  intakeDate = listing.intakeDate;
  weighLabel = listing.weighInLabel;
  intakeLabel = listing.intakeLabel;
  renderHeadline(listing);
  setChip("weigh-chip", listing.weighInWhen, listing.weighInLabel);
  setChip("intake-chip", listing.intakeWhen, listing.intakeLabel);
  $("forgot").hidden = !listing.forgottenYesterday;
  $("intake-chip").classList.toggle("warn", listing.forgottenYesterday);
  renderHistory(listing.days);
}

async function refresh() {
  render(await invoke("list_days"));
}

function clearErrors() {
  $("entry").classList.remove("bad");
  for (const id of ["weight", "protein", "carbs", "fat", "water"]) {
    $(id).classList.remove("bad");
  }
  for (const id of ["weigh-error", "intake-error"]) {
    $(id).textContent = "";
    $(id).hidden = true;
  }
}

function showBlockError(block, message) {
  const el = $(block === "weigh" ? "weigh-error" : "intake-error");
  el.textContent = message;
  el.hidden = false;
  $("entry").classList.add("bad");
  if (block === "weigh") {
    $("weight").classList.add("bad");
    return;
  }
  const lower = message.toLowerCase();
  const fields = [
    ["protéines", "protein"],
    ["glucides", "carbs"],
    ["lipides", "fat"],
    ["eau", "water"],
  ];
  let marked = false;
  for (const [word, id] of fields) {
    if (lower.includes(word)) {
      $(id).classList.add("bad");
      marked = true;
    }
  }
  if (!marked) {
    for (const id of ["protein", "carbs", "fat", "water"]) $(id).classList.add("bad");
  }
}

function intakeConfirmText(entry, old) {
  const oldWater = old.water ? `, eau ${old.water} L` : "";
  const nextWater = entry.waterText.trim() ? `, eau ${entry.waterText.trim()} L` : "";
  return (
    `Le ${entry.label} a déjà une alimentation : ${groupKcal(old.energyKcal)} kcal ` +
    `(P ${old.proteinG} g, G ${old.carbsG} g, L ${old.fatG} g${oldWater}). ` +
    `La remplacer par P ${entry.proteinText} g, G ${entry.carbsText} g, L ${entry.fatText} g${nextWater} ?`
  );
}

async function commitWeigh(entry, replace) {
  try {
    const outcome = await invoke("save_weigh_in", {
      weightText: entry.weightText,
      fasted: entry.fasted,
      date: entry.date,
      replace,
    });
    if (outcome.status === "alreadyFilled") {
      queue.push({
        kind: "weigh",
        entry,
        text:
          `Le ${entry.label} a déjà une pesée : ${outcome.old.weight} kg. ` +
          `La remplacer par ${entry.weightText} kg ?`,
      });
      return "confirm";
    }
    return "saved";
  } catch (e) {
    showBlockError("weigh", errorText(e));
    return "error";
  }
}

async function commitIntake(entry, replace) {
  try {
    const outcome = await invoke("save_intake", {
      proteinText: entry.proteinText,
      carbsText: entry.carbsText,
      fatText: entry.fatText,
      waterText: entry.waterText,
      date: entry.date,
      replace,
    });
    if (outcome.status === "alreadyFilled") {
      queue.push({
        kind: "intake",
        entry,
        text: intakeConfirmText(entry, outcome.old),
      });
      return "confirm";
    }
    return "saved";
  } catch (e) {
    showBlockError("intake", errorText(e));
    return "error";
  }
}

function showNextConfirm() {
  const next = queue[0];
  if (!next) {
    $("confirm").hidden = true;
    return;
  }
  $("confirm-text").textContent = next.text;
  $("confirm").hidden = false;
}

function dayWasSaved(outcome) {
  const failed = outcome.weighIn === "error" || outcome.intake === "error";
  const waiting = outcome.weighIn === "confirm" || outcome.intake === "confirm";
  const saved = outcome.weighIn === "saved" || outcome.intake === "saved";
  return saved && !failed && !waiting;
}

async function saveAll() {
  if (busy || !$("confirm").hidden) return;
  busy = true;
  clearErrors();
  $("saved").hidden = true;
  const weighSnap = {
    weightText: $("weight").value,
    fasted: $("fasted").checked,
    date: weighDate,
    label: weighLabel,
  };
  const intakeSnap = {
    proteinText: $("protein").value,
    carbsText: $("carbs").value,
    fatText: $("fat").value,
    waterText: $("water").value,
    date: intakeDate,
    label: intakeLabel,
  };
  const fields = {
    weight: weighSnap.weightText,
    protein: intakeSnap.proteinText,
    carbs: intakeSnap.carbsText,
    fat: intakeSnap.fatText,
    water: intakeSnap.waterText,
  };
  try {
    const outcome = await saveBlocks(
      fields,
      () => commitWeigh(weighSnap, false),
      () => commitIntake(intakeSnap, false),
    );
    await refresh();
    if (dayWasSaved(outcome)) $("saved").hidden = false;
    showNextConfirm();
  } catch (e) {
    showBlockError("weigh", errorText(e));
  } finally {
    busy = false;
  }
}

async function onReplace() {
  const item = queue.shift();
  $("confirm").hidden = true;
  if (!item || busy) return;
  busy = true;
  try {
    const result =
      item.kind === "weigh"
        ? await commitWeigh(item.entry, true)
        : await commitIntake(item.entry, true);
    await refresh();
    showNextConfirm();
    if (result === "saved" && queue.length === 0 && $("weigh-error").hidden && $("intake-error").hidden) {
      $("saved").hidden = false;
    }
  } finally {
    busy = false;
  }
}

document.addEventListener(
  "keydown",
  (event) => {
    if (event.key !== "Enter") return;
    event.preventDefault();
    if (savesOnEnter(event)) saveAll();
  },
  true,
);

$("entry").addEventListener("submit", (event) => event.preventDefault());
$("save").addEventListener("click", saveAll);
$("confirm-replace").addEventListener("click", onReplace);
$("confirm-cancel").addEventListener("click", () => {
  queue.shift();
  showNextConfirm();
});

for (const id of ["weight", "protein", "carbs", "fat", "water"]) {
  $(id).addEventListener("input", () => {
    $("saved").hidden = true;
  });
}
$("fasted").addEventListener("change", () => {
  $("saved").hidden = true;
});

refresh().catch((e) => showBlockError("weigh", errorText(e)));
