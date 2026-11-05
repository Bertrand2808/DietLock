/**
 * Screen switch. ⌘1 / ⌘2 on macOS, Ctrl+1 / Ctrl+2 elsewhere.
 * Returns "today", "weight", or null.
 */
export function screenShortcut(event, mac) {
  if (event.repeat || event.altKey || event.shiftKey) return null;
  const held = mac ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;
  if (!held) return null;
  if (event.key === "1") return "today";
  if (event.key === "2") return "weight";
  return null;
}

/** ⌘↩ saves. Enter alone does not, nor Enter combined with Ctrl, Alt or Shift. */
export function savesOnEnter(event) {
  return (
    event.key === "Enter" &&
    event.metaKey &&
    !event.ctrlKey &&
    !event.altKey &&
    !event.shiftKey
  );
}
