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
 * French copy for one macro over the week ending yesterday.
 * `kind: "absent"` is no intake in that week: a dash, never a 0.
 * A present mean of 0 g is a real zero. `gapG == null` is no pill and no gap.
 */
export function macroCopy(week) {
  if (!week || week.kind !== "present") {
    return { kind: "absent", meanText: "—", note: "", pill: null };
  }
  const note = week.days === 1 ? "1 jour saisi" : `${week.days} jours saisis`;
  const pill =
    week.gapG == null
      ? null
      : { text: `${signedGrams(week.gapG)} vs semaine d'avant`, tone: gapTone(week.gapG) };
  return { kind: "present", meanText: `${week.meanG} g/j`, note, pill };
}
