// The 15-day report image. The days come from the Rust `report` command.
// This file only paints them and hands the PNG to the clipboard.

import { activityTypeLabel } from "./activities.js";

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
  for (const block of blocks) height += cardHeight(block.lines) + 8;
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
  return canvas;
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
