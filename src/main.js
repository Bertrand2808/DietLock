import {
  WEEKDAY_INITIALS,
  addDays,
  canSelect,
  chipView,
  dayClick,
  filledDates,
  frenchDate,
  localToday,
  monthGrid,
  monthOf,
  monthTitle,
  shiftMonth,
  shortcutState,
} from "./calendar.js";
import { savesOnEnter } from "./keys.js";
import { saveBlocks } from "./save.js";

const { invoke } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);

let proposedWeigh = "";
let proposedIntake = "";
let chosenWeigh = null;
let chosenIntake = null;
let todayIso = "";
let forgottenYesterday = false;
let filledWeigh = [];
let filledIntake = [];
let seenWeighDate = null;
let seenIntakeDate = null;
let openKind = null;
let viewYear = 2026;
let viewMonth = 1;
let queue = [];
let busy = false;

function weighDateNow() {
  return chosenWeigh ?? proposedWeigh;
}

function intakeDateNow() {
  return chosenIntake ?? proposedIntake;
}

function errorText(e) {
  if (typeof e === "string") return e;
  if (e && typeof e.message === "string") return e.message;
  return String(e);
}

function groupKcal(n) {
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, "\u202f");
}

function paintChip(id, kind) {
  const el = $(id);
  const date = kind === "weigh" ? weighDateNow() : intakeDateNow();
  const proposed = kind === "weigh" ? proposedWeigh : proposedIntake;
  const view = chipView({
    date,
    proposed,
    today: todayIso,
    forgottenProposed: kind === "intake" && forgottenYesterday,
  });
  el.replaceChildren();
  const strong = document.createElement("b");
  strong.textContent = view.strong;
  el.append(strong);
  if (view.rest) el.append(view.rest);
  const chevron = document.createElement("span");
  chevron.className = "cv";
  chevron.setAttribute("aria-hidden", "true");
  chevron.textContent = "▾";
  el.append(chevron);
  el.classList.toggle("warn", view.forgotten);
  el.classList.toggle("moved", view.moved);
  el.classList.toggle("open", openKind === kind);
  el.dataset.date = date;
  el.setAttribute("aria-expanded", openKind === kind ? "true" : "false");
  if (kind === "intake") $("forgot").hidden = !view.forgotten;
}

function paintChips() {
  paintChip("weigh-chip", "weigh");
  paintChip("intake-chip", "intake");
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
  if (seenIntakeDate !== null && listing.intakeDate !== seenIntakeDate && chosenIntake === null) {
    for (const id of ["protein", "carbs", "fat", "water"]) $(id).value = "";
    $("saved").hidden = true;
  }
  if (seenWeighDate !== null && listing.weighInDate !== seenWeighDate && chosenWeigh === null) {
    $("weight").value = "";
    $("fasted").checked = true;
    $("saved").hidden = true;
  }
  seenIntakeDate = listing.intakeDate;
  seenWeighDate = listing.weighInDate;
  proposedWeigh = listing.weighInDate;
  proposedIntake = listing.intakeDate;
  todayIso = listing.today;
  forgottenYesterday = listing.forgottenYesterday;
  filledWeigh = filledDates(listing.days, "weigh");
  filledIntake = filledDates(listing.days, "intake");
  renderHeadline(listing);
  paintChips();
  if (openKind) renderPop();
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
  // A refused date is not a bad number: leave the typed fields as they are.
  if (message.startsWith("La date ") || message.includes("est dans le futur")) return;
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
  closePicker();
  const weighSnap = {
    weightText: $("weight").value,
    fasted: $("fasted").checked,
    date: weighDateNow(),
    label: frenchDate(weighDateNow()),
  };
  const intakeSnap = {
    proteinText: $("protein").value,
    carbsText: $("carbs").value,
    fatText: $("fat").value,
    waterText: $("water").value,
    date: intakeDateNow(),
    label: frenchDate(intakeDateNow()),
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

function renderPop() {
  const selected = openKind === "weigh" ? weighDateNow() : intakeDateNow();
  const filled = new Set(openKind === "weigh" ? filledWeigh : filledIntake);
  $("date-pop-title").textContent = monthTitle(viewYear, viewMonth);
  const shortcuts = shortcutState(selected, todayIso);
  $("date-today").classList.toggle("sel", shortcuts.today);
  $("date-yesterday").classList.toggle("sel", shortcuts.yesterday);
  const grid = $("date-pop-grid");
  grid.replaceChildren();
  for (const letter of WEEKDAY_INITIALS) {
    const cell = document.createElement("span");
    cell.className = "wd";
    cell.textContent = letter;
    grid.append(cell);
  }
  for (const iso of monthGrid(viewYear, viewMonth)) {
    if (!iso) {
      const blank = document.createElement("span");
      blank.className = "blank";
      grid.append(blank);
      continue;
    }
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = String(Number(iso.slice(8)));
    const future = !canSelect(iso, todayIso);
    const selectedDay = iso === selected;
    button.classList.toggle("future", future);
    button.classList.toggle("today", iso === todayIso);
    button.classList.toggle("sel", selectedDay);
    button.classList.toggle("filled", filled.has(iso) && !selectedDay);
    if (future) button.setAttribute("aria-disabled", "true");
    button.addEventListener("click", () => onDay(iso));
    grid.append(button);
  }
}

function syncToday() {
  const next = localToday();
  if (next === todayIso) return false;
  todayIso = next;
  return true;
}

function openPicker(kind) {
  syncToday();
  openKind = kind;
  const current = monthOf(kind === "weigh" ? weighDateNow() : intakeDateNow());
  viewYear = current.year;
  viewMonth = current.month;
  const host = $(kind === "weigh" ? "weigh-chip" : "intake-chip").parentElement;
  host.appendChild($("date-pop"));
  $("date-pop").hidden = false;
  renderPop();
  paintChips();
}

function closePicker() {
  if (!openKind && $("date-pop").hidden) return;
  openKind = null;
  $("date-pop").hidden = true;
  paintChips();
}

function togglePicker(kind) {
  if (openKind === kind) closePicker();
  else openPicker(kind);
}

function selectDay(kind, iso) {
  if (kind === "weigh") chosenWeigh = iso === proposedWeigh ? null : iso;
  else chosenIntake = iso === proposedIntake ? null : iso;
  $("saved").hidden = true;
  clearErrors();
  closePicker();
}

// D5: a future day is not selected. The message is local, so this click
// never calls a command and never refreshes the form.
function rejectFuture(kind, message) {
  closePicker();
  clearErrors();
  showBlockError(kind, message);
}

function onDay(iso) {
  const kind = openKind;
  if (!kind) return;
  syncToday();
  const choice = dayClick(iso, todayIso);
  if (choice.action === "refuse") {
    rejectFuture(kind, choice.message);
    return;
  }
  selectDay(kind, iso);
}

function stepMonth(delta) {
  const next = shiftMonth(viewYear, viewMonth, delta);
  if (next.year === viewYear && next.month === viewMonth) return;
  viewYear = next.year;
  viewMonth = next.month;
  renderPop();
}

document.addEventListener(
  "keydown",
  (event) => {
    if (event.key === "Escape" && openKind) {
      event.preventDefault();
      closePicker();
      return;
    }
    if (event.key !== "Enter") return;
    event.preventDefault();
    if (savesOnEnter(event)) saveAll();
  },
  true,
);

$("entry").addEventListener("submit", (event) => event.preventDefault());
$("save").addEventListener("click", saveAll);
$("weigh-chip").addEventListener("click", () => togglePicker("weigh"));
$("intake-chip").addEventListener("click", () => togglePicker("intake"));
$("date-today").addEventListener("click", () => onDay(todayIso));
$("date-yesterday").addEventListener("click", () => onDay(addDays(todayIso, -1)));
$("date-prev").addEventListener("click", () => stepMonth(-1));
$("date-next").addEventListener("click", () => stepMonth(1));
document.addEventListener("click", (event) => {
  if (!openKind) return;
  const target = event.target;
  if ($("date-pop").contains(target)) return;
  if (target.closest?.(".chip.pick")) return;
  // Opening repaints the chip and detaches the node that received the click.
  if (!target.isConnected) return;
  closePicker();
});
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

window.addEventListener("focus", () => {
  if (!proposedWeigh && !proposedIntake) return;
  if (!syncToday()) return;
  paintChips();
  if (openKind) renderPop();
});

refresh().catch((e) => showBlockError("weigh", errorText(e)));
