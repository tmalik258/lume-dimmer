export const UPDATE_OPACITY_EVENT = "update-opacity";

export const SET_DIMMER_ENABLED_EVENT = "set-dimmer-enabled";

export const DEFAULT_OPACITY = 0.5;

export const MAX_OPACITY = 0.9;

export type OpacityPayload = {
  opacity: number;
};

export type DimmerEnabledPayload = {
  enabled: boolean;
};
