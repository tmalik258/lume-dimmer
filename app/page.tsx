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
  SET_DIMMER_ENABLED_EVENT,
  UPDATE_OPACITY_EVENT,
  type DimmerEnabledPayload,
  type OpacityPayload,
} from "@/lib/events";

export default function ControlPanelPage() {
  const [opacity, setOpacity] = useState(DEFAULT_OPACITY);
  const [dimmerOn, setDimmerOn] = useState(true);

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

  const handleDimmerToggle = async () => {
    const next = !dimmerOn;
    const payload: DimmerEnabledPayload = { enabled: next };
    await emit(SET_DIMMER_ENABLED_EVENT, payload);
    setDimmerOn(next);
  };

  return (
    <main className="absolute inset-3 flex flex-col justify-center rounded-2xl border border-beige/12 bg-ink px-6 text-beige">
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
        <span className="text-sm text-beige-muted">Dimmer</span>
        <input
          type="checkbox"
          checked={dimmerOn}
          onChange={handleDimmerToggle}
          className="sr-only"
          aria-label="Toggle dimmer"
        />
        <span
          aria-hidden="true"
          className={`relative h-6 w-11 rounded-full ${
            dimmerOn ? "bg-beige" : "bg-ink-raised"
          }`}
        >
          <span
            className={`absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-ink ${
              dimmerOn ? "translate-x-5" : "translate-x-0"
            }`}
          />
        </span>
      </label>
    </main>
  );
}
