// What a screen remembers for the Owner (PRD FR-7.45): the choices and filters of a report, an
// analytics view or a list, and the screen he left the app on. The backend keeps the text a screen
// wrote and reads none of it (plan 19). Typed wrappers of the two commands.

import type { ScreenState } from '../../bindings/ScreenState';
import type { ScreenStateGetInput } from '../../bindings/ScreenStateGetInput';
import type { ScreenStateSetInput } from '../../bindings/ScreenStateSetInput';
import { callCommand } from '../api';

/** What a screen kept, or `value: null` when nothing is kept. */
export function prefsScreenGet(input: ScreenStateGetInput): Promise<ScreenState> {
  return callCommand<ScreenState>('prefs_screen_get', { input });
}

/** Keeps the JSON text of a screen. An empty text forgets it. */
export function prefsScreenSet(input: ScreenStateSetInput): Promise<ScreenState> {
  return callCommand<ScreenState>('prefs_screen_set', { input });
}
