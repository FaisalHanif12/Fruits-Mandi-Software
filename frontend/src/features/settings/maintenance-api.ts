// Typed wrappers around the plan 22 commands: the Day settings, the number grouping, the daily books
// check, updates and the support file (D5 early track, B5 contract). The inputs are text or plain
// values; every answer is the backend's (CLAUDE.md rule 1).

import type { DailyHealth } from '../../bindings/DailyHealth';
import type { DaySettings } from '../../bindings/DaySettings';
import type { Grouping } from '../../bindings/Grouping';
import type { SetNumberGroupingInput } from '../../bindings/SetNumberGroupingInput';
import type { SupportInfoInput } from '../../bindings/SupportInfoInput';
import type { SupportInfoSaved } from '../../bindings/SupportInfoSaved';
import type { UpdateCheck } from '../../bindings/UpdateCheck';
import type { UpdateInstallInput } from '../../bindings/UpdateInstallInput';
import type { UpdateInstalled } from '../../bindings/UpdateInstalled';
import { callCommand } from '../../shared/api';

/** The closing time and the automatic close (PRD 21, Day). The time is the backup screen's too. */
export function settingsDayGet(): Promise<DaySettings> {
  return callCommand<DaySettings>('settings_day_get');
}

export function settingsDaySet(input: DaySettings): Promise<DaySettings> {
  return callCommand<DaySettings>('settings_day_set', { input });
}

/** The number grouping (PRD 21, Language): 1,316,040 or 13,16,040. */
export function settingsSetNumberGrouping(input: SetNumberGroupingInput): Promise<Grouping> {
  return callCommand<Grouping>('settings_set_number_grouping', { input });
}

/** Runs the daily books check now (NFR-OBS-5). */
export function healthRun(): Promise<DailyHealth> {
  return callCommand<DailyHealth>('health_run');
}

/** The result of the last check, or null when none has run yet. */
export function healthLast(): Promise<DailyHealth | null> {
  return callCommand<DailyHealth | null>('health_last');
}

/** Asks the update server whether a newer version is on offer (NFR-INS-3). */
export function updatesCheck(): Promise<UpdateCheck> {
  return callCommand<UpdateCheck>('updates_check');
}

/**
 * Installs the version the Owner saw, after he agreed. `confirmed` is true only after he pressed
 * yes in the confirm box. On Windows the installer closes the app, so the answer may never come.
 */
export function updatesInstall(input: UpdateInstallInput): Promise<UpdateInstalled> {
  return callCommand<UpdateInstalled>('updates_install', { input });
}

/** Writes the support file (log, version, check results) to the path, or to the support folder. */
export function supportInfoSave(input: SupportInfoInput): Promise<SupportInfoSaved> {
  return callCommand<SupportInfoSaved>('support_info_save', { input });
}
