import { Window } from "@tauri-apps/api/window";

export const OVERLAY_WINDOW_LABEL = "overlay";

export async function getOverlayWindow(): Promise<Window> {
  const overlay = await Window.getByLabel(OVERLAY_WINDOW_LABEL);
  if (!overlay) {
    throw new Error(`Expected window with label ${OVERLAY_WINDOW_LABEL}`);
  }
  return overlay;
}
