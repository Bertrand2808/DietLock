/** The summary of the selected period. An unknown period is seven days. No summary yet is null. */
export function pickPeriod(summary, periodDays) {
  if (!summary) return null;
  return periodDays === 15 ? summary.fifteenDays : summary.sevenDays;
}
