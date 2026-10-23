// The real answers and requests of the backend for plan 22 (the audit viewer, the Day settings, the
// number grouping, the books check, updates and the support file; B5). The backend wrote them in its
// own test (backend/app/tests/fixtures, b5_*.json); they are copied here, never edited, and read as
// the generated types. A reader takes `unknown`, because the money in a binding is a branded type
// that a plain number is not. Times in them are the stand-in `<time>`: a test that shows a time
// puts a real one in with `at`.

import type { AuditListInput } from '../../src/bindings/AuditListInput';
import type { AuditOptions } from '../../src/bindings/AuditOptions';
import type { AuditPage } from '../../src/bindings/AuditPage';
import type { DailyHealth } from '../../src/bindings/DailyHealth';
import type { DaySettings } from '../../src/bindings/DaySettings';
import type { Grouping } from '../../src/bindings/Grouping';
import type { SetNumberGroupingInput } from '../../src/bindings/SetNumberGroupingInput';
import type { SupportInfoInput } from '../../src/bindings/SupportInfoInput';
import type { SupportInfoSaved } from '../../src/bindings/SupportInfoSaved';
import type { UiError } from '../../src/bindings/UiError';
import type { UpdateCheck } from '../../src/bindings/UpdateCheck';
import type { UpdateInstallInput } from '../../src/bindings/UpdateInstallInput';
import type { UpdateInstalled } from '../../src/bindings/UpdateInstalled';
import auditxfirstxpage from '../fixtures/b5_audit_list_first_page.json';
import auditxfirstxpagexrequest from '../fixtures/b5_audit_list_first_page_request.json';
import auditxsettings from '../fixtures/b5_audit_list_settings.json';
import auditxsettingsxrequest from '../fixtures/b5_audit_list_settings_request.json';
import auditxrefused from '../fixtures/b5_audit_list_refused.json';
import auditxrefusedxrequest from '../fixtures/b5_audit_list_refused_request.json';
import auditxoptions from '../fixtures/b5_audit_options.json';
import dayxget from '../fixtures/b5_settings_day_get.json';
import dayxset from '../fixtures/b5_settings_day_set.json';
import dayxsetxrequest from '../fixtures/b5_settings_day_set_request.json';
import dayxsetxrefused from '../fixtures/b5_settings_day_set_refused.json';
import dayxsetxrefusedxrequest from '../fixtures/b5_settings_day_set_refused_request.json';
import groupingxrequest from '../fixtures/b5_settings_set_number_grouping_request.json';
import groupingxanswer from '../fixtures/b5_settings_set_number_grouping.json';
import healthxrun from '../fixtures/b5_health_run.json';
import healthxlast from '../fixtures/b5_health_last.json';
import healthxnone from '../fixtures/b5_health_last_none.json';
import supportxrequest from '../fixtures/b5_support_info_save_request.json';
import supportxsaved from '../fixtures/b5_support_info_save.json';
import supportxrefused from '../fixtures/b5_support_info_save_refused.json';
import supportxrefusedxrequest from '../fixtures/b5_support_info_save_refused_request.json';
import updatexavailable from '../fixtures/b5_updates_check_available.json';
import updatexnone from '../fixtures/b5_updates_check_none.json';
import updatexfailed from '../fixtures/b5_updates_check_failed.json';
import installxrequest from '../fixtures/b5_updates_install_request.json';
import installxok from '../fixtures/b5_updates_install_ok.json';
import installxnotxconfirmed from '../fixtures/b5_updates_install_not_confirmed.json';
import installxchanged from '../fixtures/b5_updates_install_changed.json';
import installxnotxsigned from '../fixtures/b5_updates_install_not_signed.json';

function readAuditListInput(value: unknown): AuditListInput {
  return value as AuditListInput;
}
function readAuditOptions(value: unknown): AuditOptions {
  return value as AuditOptions;
}
function readAuditPage(value: unknown): AuditPage {
  return value as AuditPage;
}
function readDailyHealth(value: unknown): DailyHealth {
  return value as DailyHealth;
}
function readDaySettings(value: unknown): DaySettings {
  return value as DaySettings;
}
function readGrouping(value: unknown): Grouping {
  return value as Grouping;
}
function readSetNumberGroupingInput(value: unknown): SetNumberGroupingInput {
  return value as SetNumberGroupingInput;
}
function readSupportInfoInput(value: unknown): SupportInfoInput {
  return value as SupportInfoInput;
}
function readSupportInfoSaved(value: unknown): SupportInfoSaved {
  return value as SupportInfoSaved;
}
function readUiError(value: unknown): UiError {
  return value as UiError;
}
function readUpdateCheck(value: unknown): UpdateCheck {
  return value as UpdateCheck;
}
function readUpdateInstallInput(value: unknown): UpdateInstallInput {
  return value as UpdateInstallInput;
}
function readUpdateInstalled(value: unknown): UpdateInstalled {
  return value as UpdateInstalled;
}
function readNull(value: unknown): null {
  return value as null;
}

export const AUDIT_FIRST_PAGE = readAuditPage(auditxfirstxpage);
export const AUDIT_FIRST_PAGE_REQUEST = readAuditListInput(auditxfirstxpagexrequest);
export const AUDIT_SETTINGS = readAuditPage(auditxsettings);
export const AUDIT_SETTINGS_REQUEST = readAuditListInput(auditxsettingsxrequest);
export const AUDIT_REFUSED = readUiError(auditxrefused);
export const AUDIT_REFUSED_REQUEST = readAuditListInput(auditxrefusedxrequest);
export const AUDIT_OPTIONS = readAuditOptions(auditxoptions);
export const DAY_GET = readDaySettings(dayxget);
export const DAY_SET = readDaySettings(dayxset);
export const DAY_SET_REQUEST = readDaySettings(dayxsetxrequest);
export const DAY_SET_REFUSED = readUiError(dayxsetxrefused);
export const DAY_SET_REFUSED_REQUEST = readDaySettings(dayxsetxrefusedxrequest);
export const GROUPING_REQUEST = readSetNumberGroupingInput(groupingxrequest);
export const GROUPING_ANSWER = readGrouping(groupingxanswer);
export const HEALTH_RUN = readDailyHealth(healthxrun);
export const HEALTH_LAST = readDailyHealth(healthxlast);
export const HEALTH_NONE = readNull(healthxnone);
export const SUPPORT_REQUEST = readSupportInfoInput(supportxrequest);
export const SUPPORT_SAVED = readSupportInfoSaved(supportxsaved);
export const SUPPORT_REFUSED = readUiError(supportxrefused);
export const SUPPORT_REFUSED_REQUEST = readSupportInfoInput(supportxrefusedxrequest);
export const UPDATE_AVAILABLE = readUpdateCheck(updatexavailable);
export const UPDATE_NONE = readUpdateCheck(updatexnone);
export const UPDATE_FAILED = readUiError(updatexfailed);
export const INSTALL_REQUEST = readUpdateInstallInput(installxrequest);
export const INSTALL_OK = readUpdateInstalled(installxok);
export const INSTALL_NOT_CONFIRMED = readUiError(installxnotxconfirmed);
export const INSTALL_CHANGED = readUiError(installxchanged);
export const INSTALL_NOT_SIGNED = readUiError(installxnotxsigned);
