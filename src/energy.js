import { groupKcal } from "./delete.js";

/** Signed whole kcal. A tie keeps the plus sign, so it is not an absent gap. */
function signedKcal(kcal) {
  const sign = kcal < 0 ? "\u2212" : "+";
  return `${sign}${groupKcal(Math.abs(kcal))}`;
}

/**
 * French copy for the week ending yesterday.
 * `kind: "absent"` is no intake in that week: the sentence has no digit.
 * A present mean of 0 kcal is a real zero. `gapKcal == null` is no pill and no gap.
 * A gap of 0 keeps the words "+0 kcal" and is not a colored pill.
 * `yesterdayKcal == null` omits "hier". A real 0 kcal yesterday stays "hier 0".
 * `pill.tone` is "down" when the gap is negative and "up" when it is positive.
 */
export function energyCopy(week, yesterdayKcal = null) {
  if (!week || week.kind !== "present") {
    return {
      kind: "absent",
      kcalText: null,
      sentence: "Pas de moyenne sur 7 jours.",
      note: "",
      pill: null,
    };
  }
  const counted = week.days === 1 ? "1 jour saisi" : `${week.days} jours saisis`;
  const parts = [counted];
  if (yesterdayKcal != null) parts.push(`hier ${groupKcal(yesterdayKcal)}`);
  let pill = null;
  if (week.gapKcal === 0) {
    parts.push("+0 kcal");
  } else if (week.gapKcal != null) {
    pill = {
      text: `${signedKcal(week.gapKcal)} vs semaine d'avant`,
      tone: week.gapKcal < 0 ? "down" : "up",
    };
  }
  return {
    kind: "present",
    kcalText: groupKcal(week.kcal),
    sentence: "",
    note: parts.join(" · "),
    pill,
  };
}
