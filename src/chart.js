// Hand-drawn weight chart. Values come from the Rust curve. This file only places them.

const SVG = "http://www.w3.org/2000/svg";

function svg(name, attrs) {
  const node = document.createElementNS(SVG, name);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, value);
  return node;
}

// Monotone cubic (Fritsch–Carlson). Smooth, and it does not overshoot the points.
function smooth(pts) {
  if (pts.length < 2) return "";
  const n = pts.length;
  const dx = [];
  const slope = [];
  const tangent = [];
  for (let i = 0; i < n - 1; i++) {
    dx[i] = pts[i + 1][0] - pts[i][0];
    slope[i] = (pts[i + 1][1] - pts[i][1]) / dx[i];
  }
  tangent[0] = slope[0];
  tangent[n - 1] = slope[n - 2];
  for (let i = 1; i < n - 1; i++) {
    if (slope[i - 1] * slope[i] <= 0) {
      tangent[i] = 0;
    } else {
      const width = dx[i - 1] + dx[i];
      tangent[i] =
        (3 * width) / ((2 * dx[i] + dx[i - 1]) / slope[i - 1] + (dx[i] + 2 * dx[i - 1]) / slope[i]);
    }
  }
  let d = `M${pts[0][0].toFixed(1)},${pts[0][1].toFixed(1)}`;
  for (let i = 0; i < n - 1; i++) {
    const [x0, y0] = pts[i];
    const [x1, y1] = pts[i + 1];
    const h = dx[i] / 3;
    d += ` C${(x0 + h).toFixed(1)},${(y0 + tangent[i] * h).toFixed(1)} ${(x1 - h).toFixed(1)},${(y1 - tangent[i + 1] * h).toFixed(1)} ${x1.toFixed(1)},${y1.toFixed(1)}`;
  }
  return d;
}

function yScale(values) {
  if (values.length === 0) return { lo: 77, hi: 79, ticks: [77] };
  let lo = Math.floor(Math.min(...values) - 0.1);
  let hi = Math.ceil(Math.max(...values));
  if (hi - lo < 2) {
    lo = Math.floor((lo + hi) / 2 - 1);
    hi = lo + 2;
  }
  const step = hi - lo > 2 ? Math.ceil((hi - lo) / 2) : 1;
  const ticks = [];
  for (let v = lo; v <= hi + 1e-9; v += step) ticks.push(v);
  const top = Math.max(...values);
  while (ticks[ticks.length - 1] < top - 1e-9) ticks.push(ticks[ticks.length - 1] + step);
  return { lo, hi: ticks[ticks.length - 1], ticks };
}

function trendRuns(days) {
  const runs = [];
  let current = [];
  days.forEach((day, index) => {
    if (day.trendTenthKg == null) {
      if (current.length) runs.push(current);
      current = [];
    } else {
      current.push(index);
    }
  });
  if (current.length) runs.push(current);
  return runs;
}

/**
 * Draws the curve into `host`. `days` is oldest first, already trimmed to the window.
 * `notice` is the empty or single-weigh-in copy, or null when the series is plotted.
 */
export function renderChart(host, days, notice, span) {
  host.replaceChildren();
  const width = 606;
  const height = 236;
  const left = 4;
  const right = 30;
  const top = 12;
  const bottom = 22;
  const innerW = width - left - right;
  const innerH = height - top - bottom;
  const count = days.length;
  const xAt = (index) => (count <= 1 ? left + innerW : left + (innerW * index) / (count - 1));
  const kg = (tenth) => tenth / 10;
  const values = [];
  for (const day of days) {
    if (day.weighTenthKg != null) values.push(kg(day.weighTenthKg));
    if (day.trendTenthKg != null) values.push(kg(day.trendTenthKg));
  }
  const bare = notice?.kind === "empty";
  const scale = yScale(bare ? [] : values);
  const yAt = (value) => top + innerH * (1 - (value - scale.lo) / (scale.hi - scale.lo));
  const baseline = top + innerH;

  const root = svg("svg", {
    class: "wc",
    viewBox: `0 0 ${width} ${height}`,
    role: "img",
    "aria-label": "Courbe de poids",
  });
  const defs = svg("defs", {});
  const gradient = svg("linearGradient", { id: "weight-fill", x1: "0", y1: "0", x2: "0", y2: "1" });
  gradient.append(
    svg("stop", { offset: "0", "stop-color": "var(--acc)", "stop-opacity": "0.16" }),
    svg("stop", { offset: "1", "stop-color": "var(--acc)", "stop-opacity": "0" }),
  );
  defs.append(gradient);
  root.append(defs);

  for (const tick of scale.ticks) {
    const y = yAt(tick).toFixed(1);
    root.append(svg("line", { class: "wc-grid", x1: left, x2: left + innerW, y1: y, y2: y }));
    if (!bare) {
      root.append(
        svg("text", { class: "wc-y", x: width - 2, y: (yAt(tick) + 3.5).toFixed(1), "text-anchor": "end" }),
      );
      root.lastChild.textContent = String(tick);
    }
  }

  const mid = Math.round((count - 1) / 2);
  const labels = [0, mid, Math.max(count - 1, 0)];
  labels.forEach((index, place) => {
    const day = days[index];
    if (!day) return;
    const anchor = place === 0 ? "start" : place === 2 ? "end" : "middle";
    const node = svg("text", {
      class: "wc-x",
      x: xAt(index).toFixed(1),
      y: height - 6,
      "text-anchor": anchor,
    });
    node.textContent = day.label;
    root.append(node);
  });

  const runs = bare ? [] : trendRuns(days);
  const markTrend = (index) => {
    const cx = xAt(index).toFixed(1);
    const cy = yAt(kg(days[index].trendTenthKg)).toFixed(1);
    root.append(svg("circle", { class: "wc-halo", cx, cy, r: 7 }));
    root.append(svg("circle", { class: "wc-end", cx, cy, r: 3.6 }));
  };
  for (const run of runs) {
    if (run.length < 2) {
      markTrend(run[0]);
      continue;
    }
    const pts = run.map((index) => [xAt(index), yAt(kg(days[index].trendTenthKg))]);
    const line = smooth(pts);
    const last = pts[pts.length - 1];
    const first = pts[0];
    root.append(
      svg("path", {
        d: `${line} L${last[0].toFixed(1)},${baseline.toFixed(1)} L${first[0].toFixed(1)},${baseline.toFixed(1)} Z`,
        fill: "url(#weight-fill)",
      }),
    );
    root.append(svg("path", { class: "wc-trend", d: line }));
  }

  const radius = span > 15 ? 2.2 : 2.8;
  for (const [index, day] of days.entries()) {
    if (day.weighTenthKg == null) continue;
    root.append(
      svg("circle", {
        class: "wc-pt",
        cx: xAt(index).toFixed(1),
        cy: yAt(kg(day.weighTenthKg)).toFixed(1),
        r: radius,
      }),
    );
  }

  const lastRun = runs[runs.length - 1];
  if (lastRun && lastRun.length > 1) markTrend(lastRun[lastRun.length - 1]);

  if (notice?.kind === "single") {
    const index = days.findIndex((day) => day.weighTenthKg != null);
    if (index !== -1) {
      const cx = xAt(index);
      const cy = yAt(kg(days[index].weighTenthKg));
      root.append(svg("circle", { class: "wc-halo soft", cx: cx.toFixed(1), cy: cy.toFixed(1), r: 7 }));
      root.append(svg("circle", { class: "wc-one", cx: cx.toFixed(1), cy: cy.toFixed(1), r: 3.6 }));
      const anchor = cx > width / 2 ? "end" : "start";
      const label = svg("text", {
        class: "wc-lbl",
        x: (anchor === "end" ? cx - 10 : cx + 10).toFixed(1),
        y: (cy - 10).toFixed(1),
        "text-anchor": anchor,
      });
      label.textContent = `${notice.weight} kg`;
      root.append(label);
    }
  }

  host.append(root);
}
