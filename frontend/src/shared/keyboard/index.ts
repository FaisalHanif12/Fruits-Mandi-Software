// Keyboard engine (PRD 7.2 and 7.3, Code Standards F27 to F30). Every key of a screen comes from
// here: shortcuts, the Esc order, arrows in forms, menus and lists, and the focus regions.
export {
  ALL_SHORTCUTS,
  SHORTCUTS,
  matchesShortcut,
  type Shortcut,
  type ShortcutKey,
  type ShortcutScope,
} from './shortcuts';
export { useShortcut } from './useShortcut';
export {
  registerEscape,
  runEscape,
  useEscape,
  useEscapeDispatcher,
  type EscapeLevel,
} from './escape';
export { arrowSide, elementDirection, intoScreenSide, menuSide, type Side } from './direction';
export { focusRegion, registerFocusRegion, useFocusRegion, type FocusRegion } from './focusRegions';
export { pickNeighbour, shouldLeaveField, type Box, type Move } from './fieldNavigation';
export { focusNextField, formNavigationProps, useFormNavigation } from './formNavigation';
export {
  DEFAULT_PAGE_SIZE,
  moveIndex,
  rovingKeyHandler,
  useRovingKeys,
  type RovingOptions,
} from './rovingFocus';
