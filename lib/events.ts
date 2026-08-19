export const UPDATE_OPACITY_EVENT = "update-opacity";

export const DEFAULT_OPACITY = 0.5;

export const MAX_OPACITY = 0.9;

export type OpacityPayload = {
  opacity: number;
};

export function parseOpacityPayload(payload: unknown): OpacityPayload {
  if (typeof payload !== "object" || payload === null || !("opacity" in payload)) {
    throw new Error(
      `Expected update-opacity payload to be { opacity: number }, got ${JSON.stringify(payload)}`,
    );
  }

  const opacity = payload["opacity"];
  if (typeof opacity !== "number" || Number.isNaN(opacity)) {
    throw new Error(
      `Expected update-opacity payload.opacity to be a number, got ${JSON.stringify(payload)}`,
    );
  }

  return { opacity };
}
