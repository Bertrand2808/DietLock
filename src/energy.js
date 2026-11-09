import { groupKcal } from "./delete.js";

/** Signed whole kcal. A tie keeps the plus sign, so it is not an absent gap. */
function signedKcal(kcal) {
  const sign = kcal < 0 ? "\u2212" : "+";
  return `${sign}${groupKcal(Math.abs(kcal))}`;
}

/**
 * French copy for the week ending yesterday.
 * `kind: "absent"` is no intake in that week: the sentence has no digit.
 * A present mean of 0 kcal is a real zero. `gapKcal == null` is no comparison.
 */
export function energyCopy(week) {
  if (!week || week.kind !== "present") {
    return {
      kind: "absent",
      kcalText: null,
      sentence: "Pas de moyenne sur 7 jours.",
      note: "",
    };
  }
  const counted = week.days === 1 ? "sur 1 jour saisi" : `sur ${week.days} jours saisis`;
  const gap = week.gapKcal == null ? "pas de comparaison" : `${signedKcal(week.gapKcal)} kcal`;
  return {
    kind: "present",
    kcalText: groupKcal(week.kcal),
    sentence: "",
    note: `${counted} · ${gap}`,
  };
}
