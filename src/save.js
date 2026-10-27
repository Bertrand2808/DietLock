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
export async function saveBlocks(fields, saveWeighIn, saveIntake) {
  const outcome = { weighIn: "skipped", intake: "skipped" };
  if (weighInStarted(fields)) outcome.weighIn = await saveWeighIn();
  if (intakeStarted(fields)) outcome.intake = await saveIntake();
  return outcome;
}
