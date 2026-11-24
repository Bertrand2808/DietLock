// The 15-day report image. The days come from the Rust `report` command.
// This file paints them on a canvas and hands the PNG to the clipboard.
// The image is a narrow portrait column, so a phone shows it at full size.

import { activityTypeLabel } from "./activities.js";
import { trendRuns } from "./chart.js";

export const REPORT_FOOTER = "Case vide = rien saisi. Énergie = 4P+4G+9L.";

export const REPORT_COLORS = {
  bg: "#f7f4ef",
  ink: "#2a2622",
  muted: "#857c72",
  line: "#e5ded4",
  card: "#ffffff",
  accent: "#c0643d",
};

/** Logical width. The page shows the canvas at this width. */
export const REPORT_WIDTH = 480;
/** Pixel density of the canvas and of the exported PNG. */
export const REPORT_SCALE = 2;

const PAD = 20;
const GAP = 8;
const ROW = 22;
const CARD_PAD = 12;
const BODY_FONT = "15px sans-serif";
const BOLD_FONT = "600 15px sans-serif";
const INNER_WIDTH = REPORT_WIDTH - PAD * 2;
const WATER_COLUMN = 80;
const SCALE_LABEL_WIDTH = 64;
const HEADER_HEIGHT = 82;
const TREND_HEIGHT = 72;
const CURVE_HEIGHT = 196;
const FOOTER_HEIGHT = 36;

export function copiedMessage(report) {
  return `Rapport copié · 15 jours, du ${report.fromLabel} au ${report.toLabel}`;
}

function plural(count, singular, pluralForm) {
  return count > 1 ? pluralForm : singular;
}

function countedDays(count) {
  if (count === 0) return "aucun jour saisi";
  return `${count} ${plural(count, "jour saisi", "jours saisis")}`;
}

/** Rows of the summary block. A mean without a counted day reads « — » (R12). */
export function summaryRows(summary) {
  const grams = (value) => (value == null ? "—" : `${value} g`);
  return [
    {
      label: "Énergie",
      note: countedDays(summary.intakeDays),
      value: summary.energyKcal == null ? "—" : `${grouped(summary.energyKcal)} kcal`,
    },
    { label: "Protéines", note: countedDays(summary.intakeDays), value: grams(summary.proteinG) },
    { label: "Glucides", note: countedDays(summary.intakeDays), value: grams(summary.carbsG) },
    { label: "Lipides", note: countedDays(summary.intakeDays), value: grams(summary.fatG) },
    {
      label: "Eau",
      note: countedDays(summary.waterDays),
      value: summary.water == null ? "—" : `${summary.water} L`,
    },
    {
      label: "Activité",
      note: "",
      value: `${summary.activityDays} ${plural(summary.activityDays, "jour", "jours")}`,
    },
  ];
}

function grouped(kcal) {
  return String(kcal).replace(/\B(?=(\d{3})+(?!\d))/g, " ");
}

/** Words wrapped to `maxWidth`. A single word wider than that stays on its own line. */
export function wrapText(ctx, text, maxWidth) {
  const lines = [];
  let current = "";
  for (const word of text.split(" ")) {
    const candidate = current ? `${current} ${word}` : word;
    if (current && ctx.measureText(candidate).width > maxWidth) {
      lines.push(current);
      current = word;
    } else {
      current = candidate;
    }
  }
  if (current) lines.push(current);
  return lines;
}

/**
 * What one day card says, line by line. A missing value adds nothing, so its
 * cell stays empty (R3). `activityLines` are already wrapped.
 */
function layoutCard(ctx, day) {
  ctx.font = BODY_FONT;
  const activityWidth = INNER_WIDTH - CARD_PAD * 2 - WATER_COLUMN;
  const activityLines = [];
  for (const activity of day.activities ?? []) {
    const label = activityTypeLabel(activity.kind);
    const text = activity.note ? `${label} · ${activity.note}` : label;
    activityLines.push(...wrapText(ctx, text, activityWidth));
  }
  const hasStats = day.energyKcal != null || day.proteinG != null;
  const hasWater = day.water != null;
  const lowerRows = Math.max(activityLines.length, hasWater ? 1 : 0);
  const rows = 1 + (hasStats ? 1 : 0) + lowerRows;
  return {
    day,
    activityLines,
    hasStats,
    height: CARD_PAD * 2 + rows * ROW,
  };
}

function weightText(day) {
  if (!day.weight) return "";
  return day.notFasted ? `${day.weight} · pas à jeun` : day.weight;
}

function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

function paintBox(ctx, y, height) {
  roundRect(ctx, PAD, y, INNER_WIDTH, height, 8);
  ctx.fillStyle = REPORT_COLORS.card;
  ctx.fill();
  ctx.strokeStyle = REPORT_COLORS.line;
  ctx.lineWidth = 1;
  ctx.stroke();
}

function summaryHeight() {
  return CARD_PAD * 2 + 7 * ROW;
}

/**
 * Paints the report onto `canvas` at `REPORT_SCALE`. The clipboard exports
 * that same canvas, and the « Rapport » page shows it.
 */
export function paintReport(canvas, report) {
  canvas.width = REPORT_WIDTH * REPORT_SCALE;
  const ctx = canvas.getContext("2d");
  const cards = report.days.map((day) => layoutCard(ctx, day));
  let height = HEADER_HEIGHT;
  if (report.trend) height += TREND_HEIGHT;
  if (report.summary) height += summaryHeight() + GAP * 2;
  if (report.points) height += CURVE_HEIGHT;
  for (const card of cards) height += card.height + GAP;
  height += FOOTER_HEIGHT + PAD;

  canvas.height = height * REPORT_SCALE;
  ctx.scale(REPORT_SCALE, REPORT_SCALE);
  ctx.fillStyle = REPORT_COLORS.bg;
  ctx.fillRect(0, 0, REPORT_WIDTH, height);
  ctx.textBaseline = "top";
  ctx.fillStyle = REPORT_COLORS.ink;
  ctx.font = "600 22px sans-serif";
  ctx.fillText("15 jours", PAD, PAD);
  ctx.fillStyle = REPORT_COLORS.muted;
  ctx.font = BODY_FONT;
  ctx.fillText(`du ${report.fromLabel} au ${report.toLabel}`, PAD, PAD + 30);

  let y = HEADER_HEIGHT;
  if (report.trend) y = paintTrend(ctx, report.trend, y);
  if (report.summary) y = paintSummary(ctx, report.summary, y);
  if (report.points) y = paintCurve(ctx, report.points, y);
  for (const card of cards) y = paintCard(ctx, card, y);
  ctx.fillStyle = REPORT_COLORS.muted;
  ctx.font = BODY_FONT;
  ctx.textAlign = "left";
  ctx.fillText(REPORT_FOOTER, PAD, y + 8);
  return canvas;
}

function paintCard(ctx, card, y) {
  paintBox(ctx, y, card.height);
  const left = PAD + CARD_PAD;
  const right = REPORT_WIDTH - PAD - CARD_PAD;
  let rowY = y + CARD_PAD;
  ctx.fillStyle = REPORT_COLORS.ink;
  ctx.font = BOLD_FONT;
  ctx.textAlign = "left";
  ctx.fillText(card.day.label, left, rowY);
  ctx.font = BODY_FONT;
  ctx.textAlign = "right";
  ctx.fillText(weightText(card.day), right, rowY);
  rowY += ROW;
  if (card.hasStats) {
    ctx.textAlign = "left";
    if (card.day.energyKcal != null) ctx.fillText(`${grouped(card.day.energyKcal)} kcal`, left, rowY);
    ctx.textAlign = "right";
    if (card.day.proteinG != null) {
      ctx.fillText(`P ${card.day.proteinG} · G ${card.day.carbsG} · L ${card.day.fatG}`, right, rowY);
    }
    rowY += ROW;
  }
  ctx.textAlign = "right";
  if (card.day.water != null) ctx.fillText(`${card.day.water} L`, right, rowY);
  ctx.textAlign = "left";
  for (const line of card.activityLines) {
    ctx.fillText(line, left, rowY);
    rowY += ROW;
  }
  return y + card.height + GAP;
}

function paintSummary(ctx, summary, y) {
  const height = summaryHeight();
  paintBox(ctx, y, height);
  const left = PAD + CARD_PAD;
  const right = REPORT_WIDTH - PAD - CARD_PAD;
  let rowY = y + CARD_PAD;
  ctx.fillStyle = REPORT_COLORS.muted;
  ctx.font = BODY_FONT;
  ctx.textAlign = "left";
  ctx.fillText("Moyenne par jour", left, rowY);
  rowY += ROW;
  for (const row of summaryRows(summary)) {
    ctx.textAlign = "left";
    ctx.fillStyle = REPORT_COLORS.ink;
    ctx.font = BODY_FONT;
    ctx.fillText(row.label, left, rowY);
    if (row.note) {
      ctx.fillStyle = REPORT_COLORS.muted;
      ctx.fillText(`· ${row.note}`, left + 92, rowY);
    }
    ctx.textAlign = "right";
    ctx.fillStyle = REPORT_COLORS.ink;
    ctx.font = BOLD_FONT;
    ctx.fillText(row.value, right, rowY);
    rowY += ROW;
  }
  ctx.textAlign = "left";
  return y + height + GAP * 2;
}

function paintTrend(ctx, trend, y) {
  paintBox(ctx, y, 60);
  ctx.textAlign = "left";
  ctx.fillStyle = REPORT_COLORS.muted;
  ctx.font = BODY_FONT;
  ctx.fillText("Tendance sur 15 jours", PAD + 14, y + 8);
  ctx.fillStyle = REPORT_COLORS.ink;
  ctx.font = "600 18px sans-serif";
  ctx.fillText(
    `${formatTenth(trend.startTenthKg)} → ${formatTenth(trend.endTenthKg)} kg`,
    PAD + 14,
    y + 31,
  );
  ctx.fillStyle = REPORT_COLORS.accent;
  ctx.textAlign = "right";
  ctx.fillText(formatDelta(trend.variationTenthKg), REPORT_WIDTH - PAD - 14, y + 31);
  ctx.textAlign = "left";
  return y + TREND_HEIGHT;
}

function paintCurve(ctx, points, y) {
  const left = PAD + SCALE_LABEL_WIDTH;
  const right = REPORT_WIDTH - PAD - 20;
  const top = y + 8;
  const bottom = y + 150;
  const innerW = right - left;
  const innerH = bottom - top;
  const kg = (tenth) => tenth / 10;
  const values = [];
  for (const day of points) {
    if (day.weighTenthKg != null) values.push(kg(day.weighTenthKg));
    if (day.trendTenthKg != null) values.push(kg(day.trendTenthKg));
  }
  const scale = values.length === 0 ? { lo: 77, hi: 79 } : scaleOf(values);
  const xAt = (index) => (points.length <= 1 ? left : left + (innerW * index) / (points.length - 1));
  const yAt = (value) => top + innerH * (1 - (value - scale.lo) / (scale.hi - scale.lo));

  ctx.strokeStyle = REPORT_COLORS.line;
  ctx.lineWidth = 1;
  ctx.beginPath();
  const ticks = scaleTicks(scale);
  for (const tick of ticks) {
    ctx.moveTo(left, yAt(tick));
    ctx.lineTo(right, yAt(tick));
  }
  ctx.stroke();

  ctx.fillStyle = REPORT_COLORS.muted;
  ctx.font = BODY_FONT;
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  if (values.length > 0) {
    for (const tick of ticks) ctx.fillText(`${formatKg(tick)} kg`, left - 8, yAt(tick));
  }
  ctx.textBaseline = "top";

  ctx.strokeStyle = REPORT_COLORS.accent;
  ctx.lineWidth = 2;
  for (const run of trendRuns(points)) {
    ctx.beginPath();
    run.forEach((index, place) => {
      const x = xAt(index);
      const py = yAt(kg(points[index].trendTenthKg));
      if (place === 0) ctx.moveTo(x, py);
      else ctx.lineTo(x, py);
    });
    ctx.stroke();
    ctx.fillStyle = REPORT_COLORS.accent;
    for (const index of run) {
      ctx.beginPath();
      ctx.arc(xAt(index), yAt(kg(points[index].trendTenthKg)), 3, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  ctx.fillStyle = REPORT_COLORS.muted;
  for (const [index, day] of points.entries()) {
    if (day.weighTenthKg == null) continue;
    ctx.beginPath();
    ctx.arc(xAt(index), yAt(kg(day.weighTenthKg)), 2.5, 0, Math.PI * 2);
    ctx.fill();
  }

  ctx.font = BODY_FONT;
  const mid = Math.round((points.length - 1) / 2);
  ctx.textAlign = "left";
  ctx.fillText(points[0]?.label ?? "", left, bottom + 8);
  ctx.textAlign = "center";
  if (points[mid]) ctx.fillText(points[mid].label, xAt(mid), bottom + 8);
  ctx.textAlign = "right";
  const last = points[points.length - 1];
  if (last) ctx.fillText(last.label, right, bottom + 8);
  ctx.textAlign = "left";
  return y + CURVE_HEIGHT;
}

/** Top, middle and bottom of the scale, in kg. The middle can be a half. */
function scaleTicks(scale) {
  return [scale.hi, (scale.lo + scale.hi) / 2, scale.lo];
}

function formatKg(kg) {
  return String(kg).replace(".", ",");
}

function scaleOf(values) {
  let lo = Math.floor(Math.min(...values) - 0.1);
  let hi = Math.ceil(Math.max(...values));
  if (hi - lo < 2) {
    lo = Math.floor((lo + hi) / 2 - 1);
    hi = lo + 2;
  }
  return { lo, hi };
}

function formatTenth(tenth) {
  const sign = tenth < 0 ? "−" : "";
  const abs = Math.abs(tenth);
  return `${sign}${Math.floor(abs / 10)},${abs % 10}`;
}

function formatDelta(tenth) {
  const sign = tenth < 0 ? "−" : tenth > 0 ? "+" : "";
  const abs = Math.abs(tenth);
  return `${sign}${Math.floor(abs / 10)},${abs % 10} kg`;
}

export function pngBlob(canvas) {
  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) resolve(blob);
      else reject(new Error("png"));
    }, "image/png");
  });
}

/** The blob promise stays unresolved here so the click gesture is kept. */
export function reportClipboardItems(pngBlobPromise) {
  return [new ClipboardItem({ "image/png": pngBlobPromise })];
}
