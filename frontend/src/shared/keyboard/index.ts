// Keyboard engine (PRD 7.2 and 7.3, Code Standards F27 to F30). Plan 01 has the shortcut
// registry. Arrow navigation and focus management arrive in plan 04.
export { SHORTCUTS, matchesShortcut, type Shortcut } from './shortcuts';
export { useShortcut } from './useShortcut';
