// The 15-day report image. The days come from the Rust `report` command.
// This file only paints them and hands the PNG to the clipboard.

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

const WIDTH = 840;
const PAD = 28;

export function copiedMessage(report) {
  return `Rapport copié · 15 jours, du ${report.fromLabel} au ${report.toLabel}`;
}

/** Lines of one card. A missing value adds no line, so the cell stays empty. */
export function cardLines(day) {
  const lines = [];
  if (day.weight) lines.push(day.notFasted ? `${day.weight} · pas à jeun` : day.weight);
  if (day.energyKcal != null) lines.push(`${grouped(day.energyKcal)} kcal`);
  if (day.proteinG != null) lines.push(`P ${day.proteinG} · G ${day.carbsG} · L ${day.fatG}`);
  if (day.water != null) lines.push(`${day.water} L`);
  for (const activity of day.activities ?? []) {
    const label = activityTypeLabel(activity.kind);
    lines.push(activity.note ? `${label} · ${activity.note}` : label);
  }
  return lines;
}

function grouped(kcal) {
  return String(kcal).replace(/\B(?=(\d{3})+(?!\d))/g, "\u202f");
}

function cardHeight(lines) {
  return Math.max(36, 12 + Math.max(lines.length, 1) * 16);
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

/** Paints the report onto `canvas`. The clipboard then exports that canvas. */
export function paintReport(canvas, report) {
  const blocks = report.days.map((day) => ({ day, lines: cardLines(day) }));
  let height = PAD + 56;
  if (report.trend) height += 72;
  if (report.points) height += 188;
  for (const block of blocks) height += cardHeight(block.lines) + 8;
  height += 28;
  height += PAD;

  canvas.width = WIDTH;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  ctx.fillStyle = REPORT_COLORS.bg;
  ctx.fillRect(0, 0, WIDTH, height);
  ctx.textBaseline = "top";
  ctx.fillStyle = REPORT_COLORS.ink;
  ctx.font = "600 22px sans-serif";
  ctx.fillText("15 jours", PAD, PAD);
  ctx.fillStyle = REPORT_COLORS.muted;
  ctx.font = "13px sans-serif";
  ctx.fillText(`du ${report.fromLabel} au ${report.toLabel}`, PAD, PAD + 28);

  let y = PAD + 56;
  if (report.trend) y = paintTrend(ctx, report.trend, y);
  if (report.points) y = paintCurve(ctx, report.points, y);
  for (const block of blocks) {
    const h = cardHeight(block.lines);
    roundRect(ctx, PAD, y, WIDTH - PAD * 2, h, 8);
    ctx.fillStyle = REPORT_COLORS.card;
    ctx.fill();
    ctx.strokeStyle = REPORT_COLORS.line;
    ctx.lineWidth = 1;
    ctx.stroke();
    ctx.fillStyle = REPORT_COLORS.ink;
    ctx.font = "600 13px sans-serif";
    ctx.fillText(block.day.label, PAD + 12, y + 10);
    ctx.font = "13px sans-serif";
    let lineY = y + 10;
    for (const line of block.lines) {
      ctx.fillStyle = REPORT_COLORS.ink;
      ctx.fillText(line, PAD + 160, lineY);
      lineY += 16;
    }
    y += h + 8;
  }
  ctx.fillStyle = REPORT_COLORS.muted;
  ctx.font = "12px sans-serif";
  ctx.fillText(REPORT_FOOTER, PAD, y + 4);
  return canvas;
}

function paintTrend(ctx, trend, y) {
  roundRect(ctx, PAD, y, WIDTH - PAD * 2, 60, 8);
  ctx.fillStyle = REPORT_COLORS.card;
  ctx.fill();
  ctx.strokeStyle = REPORT_COLORS.line;
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.fillStyle = REPORT_COLORS.muted;
  ctx.font = "12px sans-serif";
  ctx.fillText("Tendance sur 15 jours", PAD + 14, y + 10);
  ctx.fillStyle = REPORT_COLORS.ink;
  ctx.font = "600 16px sans-serif";
  ctx.fillText(
    `${formatTenth(trend.startTenthKg)} → ${formatTenth(trend.endTenthKg)} kg`,
    PAD + 14,
    y + 30,
  );
  ctx.fillStyle = REPORT_COLORS.accent;
  ctx.font = "600 16px sans-serif";
  ctx.textAlign = "right";
  ctx.fillText(formatDelta(trend.variationTenthKg), WIDTH - PAD - 14, y + 30);
  ctx.textAlign = "left";
  return y + 72;
}

function paintCurve(ctx, points, y) {
  const left = PAD + 8;
  const right = WIDTH - PAD - 28;
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
  ctx.moveTo(left, top);
  ctx.lineTo(right, top);
  ctx.moveTo(left, bottom);
  ctx.lineTo(right, bottom);
  ctx.stroke();

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

  ctx.font = "11px sans-serif";
  const mid = Math.round((points.length - 1) / 2);
  ctx.textAlign = "left";
  ctx.fillText(points[0]?.label ?? "", left, bottom + 6);
  ctx.textAlign = "center";
  if (points[mid]) ctx.fillText(points[mid].label, xAt(mid), bottom + 6);
  ctx.textAlign = "right";
  const last = points[points.length - 1];
  if (last) ctx.fillText(last.label, right, bottom + 6);
  ctx.textAlign = "left";
  return y + 188;
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
  const sign = tenth < 0 ? "\u2212" : "";
  const abs = Math.abs(tenth);
  return `${sign}${Math.floor(abs / 10)},${abs % 10}`;
}

function formatDelta(tenth) {
  const sign = tenth < 0 ? "\u2212" : tenth > 0 ? "+" : "";
  const abs = Math.abs(tenth);
  return `${sign}${Math.floor(abs / 10)},${abs % 10} kg`;
}

export function paintThumb(thumb, source) {
  const width = 180;
  const scale = width / source.width;
  thumb.width = width;
  thumb.height = Math.max(1, Math.round(source.height * scale));
  thumb.getContext("2d").drawImage(source, 0, 0, thumb.width, thumb.height);
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
