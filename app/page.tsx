"use client";

/**
 * Page: Control Panel
 * Rendering: SSG (Tauri static export) with client interactivity
 * Reason: Desktop control window; next.config uses output: "export"
 * Last Updated: 2026-08-19
 */

import { useState } from "react";
import type { ChangeEvent } from "react";
import { emit } from "@tauri-apps/api/event";
import {
  DEFAULT_OPACITY,
  MAX_OPACITY,
  UPDATE_OPACITY_EVENT,
  type OpacityPayload,
} from "@/lib/events";
import { getOverlayWindow } from "@/lib/windows";

export default function ControlPanelPage() {
  const [opacity, setOpacity] = useState(DEFAULT_OPACITY);
  const [overlayOn, setOverlayOn] = useState(true);

  const dimPercent = Math.round((opacity / MAX_OPACITY) * 100);

  const handleSliderChange = async (e: ChangeEvent<HTMLInputElement>) => {
    const value = parseFloat(e.target.value);
    if (Number.isNaN(value)) {
      throw new Error(
        `Expected opacity slider to produce a number, got ${JSON.stringify(e.target.value)}`,
      );
    }

    setOpacity(value);
    const payload: OpacityPayload = { opacity: value };
    await emit(UPDATE_OPACITY_EVENT, payload);
  };

  const handleOverlayToggle = async () => {
    const overlay = await getOverlayWindow();
    if (overlayOn) {
      await overlay.hide();
      setOverlayOn(false);
      return;
    }

    await overlay.show();
    setOverlayOn(true);
  };

  return (
    <main className="flex h-screen flex-col justify-center bg-ink px-6 text-beige">
      <p className="mb-1 text-center text-[11px] font-medium tracking-[0.35em] text-beige-muted">
        LUME
      </p>
      <p className="mb-6 text-center text-4xl font-medium tabular-nums text-beige">
        {dimPercent}%
      </p>
      <input
        type="range"
        min="0"
        max={MAX_OPACITY}
        step="0.05"
        value={opacity}
        onChange={handleSliderChange}
        className="opacity-slider w-full cursor-pointer"
        aria-label="Dimmer opacity"
      />
      <div className="mt-2 flex w-full justify-between text-[11px] tracking-wide text-beige-muted">
        <span>Clear</span>
        <span>Max</span>
      </div>
      <label className="mt-6 flex cursor-pointer items-center justify-between border-t border-beige/12 pt-4">
        <span className="text-sm text-beige-muted">Overlay</span>
        <input
          type="checkbox"
          checked={overlayOn}
          onChange={handleOverlayToggle}
          className="sr-only"
          aria-label="Toggle overlay"
        />
        <span
          aria-hidden="true"
          className={`relative h-6 w-11 rounded-full ${
            overlayOn ? "bg-beige" : "bg-ink-raised"
          }`}
        >
          <span
            className={`absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-ink ${
              overlayOn ? "translate-x-5" : "translate-x-0"
            }`}
          />
        </span>
      </label>
    </main>
  );
}
