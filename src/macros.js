/** Signed whole grams. A real tie is a plain 0 and has no sign. */
function signedGrams(grams) {
  if (grams === 0) return "0";
  return `${grams < 0 ? "−" : "+"}${Math.abs(grams)}`;
}

function gapTone(grams) {
  if (grams === 0) return "even";
  return grams < 0 ? "down" : "up";
}

/**
 * French copy for one macro over the period ending yesterday.
 * `periodDays` is the length of that period, 7 or 15: it names the previous period in the pill.
 * `kind: "absent"` is no intake in that period: a dash, never a 0.
 * A present mean of 0 g is a real zero. `gapG == null` is no pill and no gap.
 */
export function macroCopy(mean, periodDays = 7) {
  if (!mean || mean.kind !== "present") {
    return { kind: "absent", meanText: "—", note: "", pill: null };
  }
  const note = mean.days === 1 ? "1 jour saisi" : `${mean.days} jours saisis`;
  const pill =
    mean.gapG == null
      ? null
      : { text: `${signedGrams(mean.gapG)} vs ${previousPeriodName(periodDays)}`, tone: gapTone(mean.gapG) };
  return { kind: "present", meanText: `${mean.meanG} g/j`, note, pill };
}

function previousPeriodName(periodDays) {
  return periodDays === 7 ? "semaine d'avant" : `${periodDays} jours d'avant`;
}

export function macroPeriodCaption(periodDays) {
  return `${periodDays} derniers jours, hier compris`;
}
