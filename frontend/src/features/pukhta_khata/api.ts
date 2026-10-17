// Typed wrappers around the commands of the پختہ کھاتہ accounts (plans 13 and 15; module
// `pukhta_khata`, Tech Stack 5.2). Components never call invoke() (Code Standards F4). The two
// reads only read (Code Standards B29); `pukhta_make_regular` writes.

import type { MadeRegular } from '../../bindings/MadeRegular';
import type { MakeRegularInput } from '../../bindings/MakeRegularInput';
import type { PukhtaPeopleInput } from '../../bindings/PukhtaPeopleInput';
import type { PukhtaPerson } from '../../bindings/PukhtaPerson';
import type { PukhtaRegisterInput } from '../../bindings/PukhtaRegisterInput';
import type { PukhtaRegisterRow } from '../../bindings/PukhtaRegisterRow';
import type { PukhtaSide } from '../../bindings/PukhtaSide';
import { callCommand } from '../../shared/api';

/**
 * The one-time people already recorded on one side, newest first, matched by the text in either
 * name or the phone (PRD FR-7.29). The text is searched as typed and never converted (F41).
 */
export function searchPukhtaPeople(text: string, side: PukhtaSide): Promise<PukhtaPerson[]> {
  const input: PukhtaPeopleInput = { text, side };
  return callCommand<PukhtaPerson[]>('search_pukhta_people', { input });
}

/** The register of the one-time accounts (PRD FR-13.6, R9). Empty dates mean no limit. */
export function pukhtaRegister(input: PukhtaRegisterInput): Promise<PukhtaRegisterRow[]> {
  return callCommand<PukhtaRegisterRow[]>('pukhta_register', { input });
}

/**
 * Makes a one-time person a regular account and moves his open balance to it with a جنرل entry
 * (PRD FR-13.8): one action, one transaction. `account_input` is the account form's input.
 */
export function pukhtaMakeRegular(input: MakeRegularInput): Promise<MadeRegular> {
  return callCommand<MadeRegular>('pukhta_make_regular', { input });
}
