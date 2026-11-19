// Where a row of a dues page leads (PRD FR-9.1: each row opens that person's history). The addresses are plain
// data, so a test can say where a click goes. A person who borrowed opens the ادھار رقم screen with his name
// chosen and every amount empty (PRD FR-7.79); a seller with an advance opens the پیشگی screen the same way. Nothing
// is saved by opening them.

import type { ReportParty } from '../../bindings/ReportParty';
import type { PickerValue } from '../../shared/ui';

/** A person of a dues list as a whole picker value, so the screen that opens already has him chosen. */
export function pickerOfParty(party: ReportParty): PickerValue {
  return {
    id: party.account_id,
    code: party.code,
    name_ur: party.name_ur,
    name_en: party.name_en,
    ...(party.tag === null || party.tag === '' ? {} : { tag: party.tag }),
  };
}

/** The address of the ادھار رقم screen with one person chosen: his loans open under the form (PRD FR-16.5). */
export function lentSearchOf(party: ReportParty) {
  return { type: 'money_lent' as const, party: pickerOfParty(party) };
}
