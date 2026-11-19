// What a screen remembers for the Owner (PRD FR-7.45): see api.ts, hooks.ts and remember.ts.
export { prefsScreenGet, prefsScreenSet } from './api';
export { REMEMBER_SETTLE_MS, SCREEN_STATE_KEY, useScreenMemory, type ScreenMemory } from './hooks';
export { Remembered } from './Remembered';
export {
  MAX_REMEMBERED_CHARS,
  canonicalText,
  pickedText,
  type MemoryProps,
  choiceOr,
  flagOf,
  idOrNull,
  isRecord,
  pickedOrNull,
  readRemembered,
  rememberedText,
  textOr,
} from './remember';
