export function weighInStarted(fields) {
  return fields.weight.trim() !== "";
}

export function intakeStarted(fields) {
  return ["protein", "carbs", "fat", "water"].some((key) => fields[key].trim() !== "");
}

/**
 * Attempts each started block on its own. A refusal or a confirmation in one
 * block does not skip the other (S1).
 * The callbacks resolve to "saved", "error", "confirm" or any other status.
 * They must not reject: a thrown error would skip the remaining block.
 */
export function blockStatus(status) {
  if (status === "unchanged") return "skipped";
  if (status === "alreadyFilled") return "confirm";
  if (status === "saved") return "saved";
  return status;
}

export function intakeConfirmText(entry) {
  const macrosBlank = ["proteinText", "carbsText", "fatText"].every(
    (key) => entry[key].trim() === "",
  );
  if (macrosBlank && entry.old.water) {
    return `Le ${entry.label} a déjà une eau : ${entry.old.water} L. La remplacer par ${entry.waterText.trim()} L ?`;
  }
  const oldWater = entry.old.water ? `, eau ${entry.old.water} L` : "";
  const nextWater = entry.waterText.trim() ? `, eau ${entry.waterText.trim()} L` : "";
  return (
    `Le ${entry.label} a déjà une alimentation : ${entry.old.energy} kcal ` +
    `(P ${entry.old.proteinG} g, G ${entry.old.carbsG} g, L ${entry.old.fatG} g${oldWater}). ` +
    `La remplacer par P ${entry.proteinText} g, G ${entry.carbsText} g, L ${entry.fatText} g${nextWater} ?`
  );
}

export async function saveBlocks(fields, saveWeighIn, saveIntake) {
  const outcome = { weighIn: "skipped", intake: "skipped" };
  if (weighInStarted(fields)) outcome.weighIn = await saveWeighIn();
  if (intakeStarted(fields)) outcome.intake = await saveIntake();
  return outcome;
}
