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
