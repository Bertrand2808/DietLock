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
import { renderChart } from "./chart.js";
import { deleteFailureText, deleteQuestion, deleteThenReload, groupKcal } from "./delete.js";
import { energyCopy } from "./energy.js";
import { savesOnEnter, screenShortcut } from "./keys.js";
import { blockStatus, intakeConfirmText, saveBlocks } from "./save.js";

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
let screenName = "today";
let span = 15;
let curve = null;
let curveLoaded = false;

const mac = /\bMac/i.test(navigator.platform) || /\bMac/i.test(navigator.userAgent);

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
  document.querySelector("#screen-today .head").classList.toggle("with-trend", listing.isTrend);
  renderEnergy(listing.energyWeek);
}

function renderEnergy(week) {
  const copy = energyCopy(week);
  const figure = $("energy-figure");
  const note = $("energy-note");
  figure.replaceChildren();
  if (copy.kind === "present") {
    figure.className = "trend";
    const num = document.createElement("span");
    num.className = "num";
    num.textContent = copy.kcalText;
    const unit = document.createElement("small");
    unit.textContent = " kcal";
    figure.append(num, unit);
  } else {
    figure.className = "energy-empty";
    figure.textContent = copy.sentence;
  }
  note.textContent = copy.note;
}

function deleteButton(day, block, name) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "del";
  button.textContent = block === "weighIn" ? "× pesée" : "× alim.";
  button.setAttribute("aria-label", `Supprimer ${name} du ${day.label}`);
  button.addEventListener("click", () => askDelete(day, block));
  return button;
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
    const acts = document.createElement("span");
    acts.className = "acts";
    if (day.weighIn) acts.append(deleteButton(day, "weighIn", "la pesée"));
    if (day.intake) acts.append(deleteButton(day, "intake", "l'alimentation"));
    row.append(date, weight, detail, acts);
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

function formatDelta(tenth) {
  const sign = tenth < 0 ? "\u2212" : tenth > 0 ? "+" : "";
  const abs = Math.abs(tenth);
  return `${sign}${Math.floor(abs / 10)},${abs % 10} kg`;
}

function weighCount(n) {
  if (n === 0) return "aucune pesée";
  if (n === 1) return "1 pesée";
  return `${n} pesées`;
}

function visibleDays() {
  if (!curve) return [];
  const take = Math.min(span, curve.days.length);
  return curve.days.slice(curve.days.length - take);
}

function paintScreenTabs() {
  const today = screenName === "today";
  $("screen-today").hidden = !today;
  $("screen-weight").hidden = today;
  $("tab-today").setAttribute("aria-selected", today ? "true" : "false");
  $("tab-weight").setAttribute("aria-selected", today ? "false" : "true");
}

function paintSpan() {
  for (const button of $("span-seg").querySelectorAll("button")) {
    button.setAttribute("aria-pressed", Number(button.dataset.span) === span ? "true" : "false");
  }
}

function paintWeightHeadline() {
  const headline = $("w-headline");
  const sub = $("w-sub");
  headline.replaceChildren();
  sub.replaceChildren();
  if (curve.trend) {
    headline.classList.remove("greet");
    const num = document.createElement("span");
    num.className = "num";
    num.textContent = curve.trend;
    const unit = document.createElement("small");
    unit.textContent = " kg de tendance";
    headline.append(num, unit);
    const delta = span === 15 ? curve.shortVariation : curve.longVariation;
    const tail = "moyenne à jeun sur 7 jours";
    if (delta == null) {
      sub.textContent = tail;
    } else {
      const strong = document.createElement("b");
      strong.textContent = formatDelta(delta);
      sub.append(strong, ` en ${span} jours · ${tail}`);
    }
    return;
  }
  const notice = curve.notice;
  if (notice?.kind === "empty") {
    headline.classList.add("greet");
    headline.textContent = notice.headline;
    sub.textContent = notice.subtitle;
    return;
  }
  if (notice?.kind === "single") {
    headline.classList.remove("greet");
    const num = document.createElement("span");
    num.className = "num";
    num.textContent = notice.weight;
    const unit = document.createElement("small");
    unit.textContent = notice.today ? " kg ce matin" : " kg";
    headline.append(num, unit);
  } else {
    headline.classList.remove("greet");
  }
  sub.textContent = "La tendance apparaîtra après 4 pesées à jeun.";
}

function paintChartMessage(notice, days) {
  if (notice?.kind !== "empty" && notice?.kind !== "single") return;
  const box = document.createElement("div");
  const body = document.createElement("div");
  const small = document.createElement("small");
  if (notice.kind === "empty") {
    box.className = "chartmsg";
    const key = mac ? "⌘1" : "Ctrl+1";
    body.append(notice.chartTitle);
    small.textContent = notice.chartNote.replace("Aujourd'hui,", `Aujourd'hui (${key}),`);
  } else {
    const index = days.findIndex((day) => day.weighTenthKg != null);
    const onLeft = index === -1 || index > days.length / 2;
    box.className = onLeft ? "chartmsg side" : "chartmsg";
    body.append(notice.title);
    const breakAt = notice.note.indexOf(" sur 7 jours");
    if (breakAt === -1) {
      small.textContent = notice.note;
    } else {
      small.append(notice.note.slice(0, breakAt));
      small.append(document.createElement("br"));
      small.append(notice.note.slice(breakAt + 1));
    }
  }
  body.append(small);
  box.append(body);
  $("chartbox").append(box);
}

function renderWeight() {
  if (!curve) return;
  paintSpan();
  paintWeightHeadline();
  const days = visibleDays();
  const n = days.filter((day) => day.weighTenthKg != null).length;
  $("w-range").textContent = `${span} derniers jours`;
  $("w-count").textContent = weighCount(n);
  const hint = $("w-hint");
  hint.replaceChildren();
  const key = mac ? "⌘" : "Ctrl";
  const piece = (digit, label) => {
    const mod = document.createElement("kbd");
    mod.textContent = key;
    const num = document.createElement("kbd");
    num.textContent = digit;
    hint.append(mod, num, ` ${label}`);
  };
  piece("1", "Aujourd'hui · ");
  piece("2", "Poids");
  renderChart($("chartbox"), days, curve.notice, span);
  paintChartMessage(curve.notice, days);
}

function renderWeightError(message) {
  const box = $("chartbox");
  box.replaceChildren();
  const note = document.createElement("p");
  note.className = "empty";
  note.textContent = message;
  box.append(note);
}

async function loadCurve() {
  curve = await invoke("weight_curve");
  curveLoaded = true;
}

async function showScreen(name) {
  if (name !== "today") closePicker();
  screenName = name;
  paintScreenTabs();
  if (name !== "weight") return;
  if (!curveLoaded) {
    try {
      await loadCurve();
    } catch (e) {
      renderWeightError(errorText(e));
      return;
    }
  }
  renderWeight();
}

async function refresh() {
  render(await invoke("list_days"));
  if (!curveLoaded) return;
  try {
    await loadCurve();
  } catch (e) {
    curve = null;
    curveLoaded = false;
    if (screenName === "weight") renderWeightError(errorText(e));
    return;
  }
  if (screenName === "weight") renderWeight();
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

function intakeQuestion(entry, old) {
  return intakeConfirmText({
    label: entry.label,
    proteinText: entry.proteinText,
    carbsText: entry.carbsText,
    fatText: entry.fatText,
    waterText: entry.waterText,
    old: {
      energy: groupKcal(old.energyKcal),
      proteinG: old.proteinG,
      carbsG: old.carbsG,
      fatG: old.fatG,
      water: old.water,
    },
  });
}

async function commitWeigh(entry, replace) {
  try {
    const outcome = await invoke("save_weigh_in", {
      weightText: entry.weightText,
      fasted: entry.fasted,
      date: entry.date,
      replace,
    });
    const status = blockStatus(outcome.status);
    if (status === "confirm") {
      queue.push({
        kind: "weigh",
        entry,
        text:
          `Le ${entry.label} a déjà une pesée : ${outcome.old.weight} kg. ` +
          `La remplacer par ${entry.weightText} kg ?`,
      });
    }
    return status;
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
    const status = blockStatus(outcome.status);
    if (status === "confirm") {
      queue.push({
        kind: "intake",
        entry,
        text: intakeQuestion(entry, outcome.old),
      });
    }
    return status;
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

let pendingDelete = null;

function hideDeleteConfirm() {
  pendingDelete = null;
  $("delete-confirm").hidden = true;
  $("delete-yes").hidden = false;
}

async function askDelete(day, block) {
  if (busy) return;
  hideDeleteConfirm();
  try {
    const outcome = await invoke("delete_entry", { date: day.date, block, confirm: false });
    if (outcome.status === "confirm") {
      pendingDelete = { date: day.date, block };
      $("delete-text").textContent = deleteQuestion(day.label, outcome.target);
      $("delete-confirm").hidden = false;
    } else {
      await refresh();
    }
  } catch (e) {
    $("delete-text").textContent = errorText(e);
    $("delete-confirm").hidden = false;
    $("delete-yes").hidden = true;
  }
}

async function onDeleteConfirmed() {
  const item = pendingDelete;
  if (!item || busy) return;
  busy = true;
  const { deleted, error } = await deleteThenReload(
    () => invoke("delete_entry", { ...item, confirm: true }),
    refresh,
  );
  if (error) {
    $("delete-text").textContent = deleteFailureText(deleted, errorText(error));
    $("delete-yes").hidden = true;
    $("delete-confirm").hidden = false;
  } else {
    hideDeleteConfirm();
  }
  busy = false;
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
    const next = screenShortcut(event, mac);
    if (next) {
      event.preventDefault();
      showScreen(next);
      return;
    }
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

$("tab-today").addEventListener("click", () => showScreen("today"));
$("tab-weight").addEventListener("click", () => showScreen("weight"));
$("span-seg").addEventListener("click", (event) => {
  const button = event.target.closest("button");
  if (!button) return;
  span = Number(button.dataset.span);
  if (curve) renderWeight();
});
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
$("delete-yes").addEventListener("click", onDeleteConfirmed);
$("delete-no").addEventListener("click", hideDeleteConfirm);
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
