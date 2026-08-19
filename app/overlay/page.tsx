"use client";

/**
 * Page: Overlay
 * Rendering: SSG (Tauri static export) with client interactivity
 * Reason: Transparent screen dimmer window; next.config uses output: "export"
 * Last Updated: 2026-08-19
 */

import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  DEFAULT_OPACITY,
  UPDATE_OPACITY_EVENT,
  parseOpacityPayload,
} from "@/lib/events";

export default function OverlayPage() {
  const [opacity, setOpacity] = useState(DEFAULT_OPACITY);

  useEffect(() => {
    let cancelled = false;
    let unlisten: UnlistenFn | undefined;

    async function setupClickThroughAndListener() {
      const appWindow = getCurrentWindow();
      await appWindow.setIgnoreCursorEvents(true);

      const stop = await listen(UPDATE_OPACITY_EVENT, (event) => {
        const payload = parseOpacityPayload(event.payload);
        setOpacity(payload.opacity);
      });

      if (cancelled) {
        stop();
        return;
      }

      unlisten = stop;
    }

    setupClickThroughAndListener();

    return () => {
      cancelled = true;
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  return (
    <main
      className="pointer-events-none fixed inset-0 h-screen w-screen bg-black"
      style={{ opacity }}
      aria-hidden="true"
    />
  );
}
