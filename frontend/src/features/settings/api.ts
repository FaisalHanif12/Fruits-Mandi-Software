// Typed wrappers around the settings, preferences and user commands (plan 04 contract 4).

import type { FirmSettings } from '../../bindings/FirmSettings';
import type { PrefsSetInput } from '../../bindings/PrefsSetInput';
import type { SetDefaultLanguageInput } from '../../bindings/SetDefaultLanguageInput';
import type { SetSecurityInput } from '../../bindings/SetSecurityInput';
import type { Settings } from '../../bindings/Settings';
import { callCommand } from '../../shared/api';

export function settingsGet(): Promise<Settings> {
  return callCommand<Settings>('settings_get');
}

export function settingsSetFirm(input: FirmSettings): Promise<FirmSettings> {
  return callCommand<FirmSettings>('settings_set_firm', { input });
}

export function settingsSetSecurity(input: SetSecurityInput): Promise<null> {
  return callCommand<null>('settings_set_security', { input });
}

export function settingsSetDefaultLanguage(input: SetDefaultLanguageInput): Promise<null> {
  return callCommand<null>('settings_set_default_language', { input });
}

/** Saves one preference of the signed-in user: language, text size or last screen. */
export function prefsSet(input: PrefsSetInput): Promise<null> {
  return callCommand<null>('prefs_set', { input });
}
