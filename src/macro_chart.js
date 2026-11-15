// Hand-drawn bar chart of one macro. Values come from the Rust summary. This file only places them.

const SVG = "http://www.w3.org/2000/svg";
const WIDTH = 420;
const HEIGHT = 150;
const LEFT = 8;
const RIGHT = 8;
const TOP = 22;
const BOTTOM = 22;
const BAR_SHARE = 0.6;
const ZERO_STUB_HEIGHT = 2;

function svg(name, attrs) {
  const node = document.createElementNS(SVG, name);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, value);
  return node;
}

/**
 * Where each bar goes. `days` are oldest first, `tenthG == null` is a day without intake.
 * A hole has no bar. A day at 0 g has a short stub, so it stays distinct from a hole.
 * `meanG` is the displayed mean in whole grams, or null. An empty period has no bar and no mean.
 */
export function macroBars(days, meanG) {
  const entered = days.filter((day) => day.tenthG != null);
  if (entered.length === 0) return { empty: true, bars: [], slots: [], mean: null };
  const innerW = WIDTH - LEFT - RIGHT;
  const innerH = HEIGHT - TOP - BOTTOM;
  const baseline = TOP + innerH;
  const slot = innerW / days.length;
  const meanTenth = meanG == null ? null : meanG * 10;
  // The 1 avoids dividing by zero when every day is 0 g.
  const top = Math.max(1,...entered.map((day) => day.tenthG), meanTenth ?? 0);
  const slots = days.map((day, index) => ({ x: LEFT + slot * (index + 0.5), label: day.label }));
  const bars = [];
  days.forEach((day, index) => {
    if (day.tenthG == null) return;
    const height = day.tenthG === 0 ? ZERO_STUB_HEIGHT : (day.tenthG / top) * innerH;
    bars.push({
      x: slots[index].x - (slot * BAR_SHARE) / 2,
      y: baseline - height,
      width: slot * BAR_SHARE,
      height,
      label: day.label,
      text: day.text,
      title: `${day.label} · ${day.text}`,
    });
  });
  const mean =
    meanTenth == null
      ? null
      : { y: baseline - (meanTenth / top) * innerH, x1: LEFT, x2: WIDTH - RIGHT, text: `moyenne ${meanG} g` };
  return { empty: false, bars, slots, mean, baseline };
}

function textNode(className, attrs, content) {
  const node = svg("text", { class: className, ...attrs });
  node.textContent = content;
  return node;
}

// Beyond this many days the bars are too narrow for a value or a weekday label.
const LABELLED_DAYS_MAX = 7;

export function renderMacroChart(host, days, meanG, periodDays = days.length) {
  host.replaceChildren();
  const geometry = macroBars(days, meanG);
  const labelled = days.length <= LABELLED_DAYS_MAX;
  if (geometry.empty) {
    const note = document.createElement("p");
    note.className = "empty";
    note.textContent = `Aucune saisie sur ces ${periodDays} jours.`;
    host.append(note);
    return;
  }
  const root = svg("svg", {
    class: "mc",
    viewBox: `0 0 ${WIDTH} ${HEIGHT}`,
    role: "img",
    "aria-label": "Protéines par jour",
  });
  root.append(
    svg("line", { class: "mc-base", x1: LEFT, x2: WIDTH - RIGHT, y1: geometry.baseline, y2: geometry.baseline }),
  );
  for (const bar of geometry.bars) {
    const rect = svg("rect", {
      class: "mc-bar",
      x: bar.x.toFixed(1),
      y: bar.y.toFixed(1),
      width: bar.width.toFixed(1),
      height: bar.height.toFixed(1),
      rx: 2,
    });
    const title = svg("title", {});
    title.textContent = bar.title;
    rect.append(title);
    root.append(rect);
    if (!labelled) continue;
    root.append(
      textNode("mc-val", { x: (bar.x + bar.width / 2).toFixed(1), y: (bar.y - 4).toFixed(1), "text-anchor": "middle" }, bar.text),
    );
  }
  for (const slot of geometry.slots) {
    const axisLabel = labelled ? slot.label : slot.label.split(" ").pop();
    root.append(textNode("mc-x", { x: slot.x.toFixed(1), y: HEIGHT - 6, "text-anchor": "middle" }, axisLabel));
  }
  if (geometry.mean) {
    const { mean } = geometry;
    root.append(svg("line", { class: "mc-mean", x1: mean.x1, x2: mean.x2, y1: mean.y.toFixed(1), y2: mean.y.toFixed(1) }));
    root.append(textNode("mc-mean-lbl", { x: mean.x2, y: (mean.y - 4).toFixed(1), "text-anchor": "end" }, mean.text));
  }
  host.append(root);
}
