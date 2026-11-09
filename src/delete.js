export function groupKcal(n) {
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, " ");
}

/** S10: the question shown before a block is deleted, with the values that will go. */
export function deleteQuestion(label, target) {
  if (target.kind === "weighIn") {
    return `Supprimer la pesée du ${label} : ${target.weight} kg ?`;
  }
  const water = target.water === null ? "" : `, eau ${target.water} L`;
  return (
    `Supprimer l'alimentation du ${label} : ${groupKcal(target.energyKcal)} kcal ` +
    `(P ${target.proteinG} g, G ${target.carbsG} g, L ${target.fatG} g${water}) ?`
  );
}

/** Runs the deletion, then the reload, and says which step failed. */
export async function deleteThenReload(remove, reload) {
  try {
    await remove();
  } catch (error) {
    return { deleted: false, error };
  }
  try {
    await reload();
  } catch (error) {
    return { deleted: true, error };
  }
  return { deleted: true, error: null };
}

export function deleteFailureText(deleted, detail) {
  return deleted ? `Supprimé, mais la liste n'a pas pu être rechargée : ${detail}` : detail;
}
