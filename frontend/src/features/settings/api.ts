// Typed wrappers around the settings, preferences and user commands (plan 04 contract 4).

import type { FirmSettings } from '../../bindings/FirmSettings';
import type { PrefsSetInput } from '../../bindings/PrefsSetInput';
import type { SetDefaultLanguageInput } from '../../bindings/SetDefaultLanguageInput';
import type { PrintingSettings } from '../../bindings/PrintingSettings';
import type { SetFirmInput } from '../../bindings/SetFirmInput';
import type { SetPrintingInput } from '../../bindings/SetPrintingInput';
import type { SetSecurityInput } from '../../bindings/SetSecurityInput';
import type { Settings } from '../../bindings/Settings';
import { callCommand } from '../../shared/api';
import type { ChargesInput, ChargesSettings, OverridesSettings } from './types';

export function settingsGet(): Promise<Settings> {
  return callCommand<Settings>('settings_get');
}

/** Saves the firm. The names have the spelling check, so the words he confirmed go with it. */
export function settingsSetFirm(input: SetFirmInput): Promise<FirmSettings> {
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

/** The global defaults of Settings "Charges" (they are rules). */
export function settingsGetCharges(): Promise<ChargesSettings> {
  return callCommand<ChargesSettings>('settings_get_charges');
}

export function settingsSetCharges(input: ChargesInput): Promise<ChargesSettings> {
  return callCommand<ChargesSettings>('settings_set_charges', { input });
}

/** The line override and credit limit settings (PRD Section 21 "Overrides" and "Checks"). */
export function settingsGetOverrides(): Promise<OverridesSettings> {
  return callCommand<OverridesSettings>('settings_get_overrides');
}

export function settingsSetOverrides(input: OverridesSettings): Promise<OverridesSettings> {
  return callCommand<OverridesSettings>('settings_set_overrides', { input });
}

/** The Printing settings: paper per document, printer, copies, Urdu digits, print after save. */
export function settingsGetPrinting(): Promise<PrintingSettings> {
  return callCommand<PrintingSettings>('settings_get_printing');
}

/** Saves the Printing settings. Copies go as typed; the backend checks 1 to 99 (field `copies`). */
export function settingsSetPrinting(input: SetPrintingInput): Promise<PrintingSettings> {
  return callCommand<PrintingSettings>('settings_set_printing', { input });
}
