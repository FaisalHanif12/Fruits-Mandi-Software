// The real answers of the backend for the dead amount (plan 40, PRD FR-10.10, E2E-44): the preview and
// the move of ارشد, the party card of a person in the dead amount, R38 after the move and after the
// recovery, the tiles at each step, the picker hit of a dead person, and the refusals. The backend wrote
// them in its own tests (backend/crates/services/tests/fixtures); they are copied here by
// `scripts/copy-backend-fixtures.mjs`, never edited, and read as the generated types. Money in them is
// paisa, as on the wire. A reader takes `unknown`, because the money in a binding is a branded type
// that a plain number is not.

import type { AccountHit } from '../../src/bindings/AccountHit';
import type { DashboardTiles } from '../../src/bindings/DashboardTiles';
import type { DeadAmountPage } from '../../src/bindings/DeadAmountPage';
import type { DeadMarked } from '../../src/bindings/DeadMarked';
import type { DeadMarkInput } from '../../src/bindings/DeadMarkInput';
import type { DeadPreview } from '../../src/bindings/DeadPreview';
import type { PartyCard } from '../../src/bindings/PartyCard';
import type { PukhtaPerson } from '../../src/bindings/PukhtaPerson';
import cardArshad from '../fixtures/dead_party_card_arshad.json';
import tilesMove from '../fixtures/dead_tiles_after_the_move.json';
import tilesRecovery from '../fixtures/dead_tiles_after_the_recovery.json';
import tilesDayOne from '../fixtures/dead_tiles_end_of_day_1.json';
import markArshad from '../fixtures/masters_dead_mark_arshad.json';
import markArshadRequest from '../fixtures/masters_dead_mark_arshad_request.json';
import searchIncludeDeadRequest from '../fixtures/search_accounts_include_dead_request.json';
import previewArshad from '../fixtures/masters_dead_preview_arshad.json';
import previewNothingOwed from '../fixtures/masters_dead_preview_nothing_owed.json';
import pageAfterMove from '../fixtures/reports_dead_amount_after_the_move.json';
import pageAfterRecovery from '../fixtures/reports_dead_amount_after_the_recovery.json';
import pageAllAfterUndo from '../fixtures/reports_dead_amount_all_after_the_undo.json';
import searchIncludeDead from '../fixtures/search_accounts_include_dead.json';
import pukhtaDeadHidden from '../fixtures/search_pukhta_people_dead_hidden.json';
import pukhtaDeadHiddenRequest from '../fixtures/search_pukhta_people_dead_hidden_request.json';
import pukhtaIncludeDead from '../fixtures/search_pukhta_people_include_dead.json';
import pukhtaIncludeDeadRequest from '../fixtures/search_pukhta_people_include_dead_request.json';

function readPreview(value: unknown): DeadPreview {
  return value as DeadPreview;
}
function readMarked(value: unknown): DeadMarked {
  return value as DeadMarked;
}
function readMarkInput(value: unknown): DeadMarkInput {
  return value as DeadMarkInput;
}
function readCard(value: unknown): PartyCard {
  return value as PartyCard;
}
function readTiles(value: unknown): Partial<DashboardTiles> {
  return value as Partial<DashboardTiles>;
}
function readPage(value: unknown): DeadAmountPage {
  return value as DeadAmountPage;
}
function readHits(value: unknown): AccountHit[] {
  return value as AccountHit[];
}
function readPeople(value: unknown): PukhtaPerson[] {
  return value as PukhtaPerson[];
}
function readRequest(value: unknown): Record<string, unknown> {
  return value as Record<string, unknown>;
}

/** What moving ارشد on 11-10-26 would do: he owes 18,684, his last entry is of 10-10-26. */
export const PREVIEW_ARSHAD = readPreview(previewArshad);
/** A person who owes nothing: the move is refused with `ERR_NOTHING_OWED`. */
export const PREVIEW_NOTHING_OWED = readPreview(previewNothingOwed);
/** The move saved: mark 1, 18,684 kept in the history. */
export const MARKED_ARSHAD = readMarked(markArshad);
/** His party card after the move: inactive, with the move, the date and the reason. */
export const CARD_ARSHAD_DEAD = readCard(cardArshad);
/** The tiles after the move (buyers 2,880, dead 18,684, total 21,564), after the 4,000, and before. */
export const TILES_AFTER_MOVE = readTiles(tilesMove);
export const TILES_AFTER_RECOVERY = readTiles(tilesRecovery);
export const TILES_END_OF_DAY_1 = readTiles(tilesDayOne);
/** R38 after the move, after the recovery of 4,000 (still owed 14,684) and "all" after the undo. */
export const R38_AFTER_MOVE = readPage(pageAfterMove);
export const R38_AFTER_RECOVERY = readPage(pageAfterRecovery);
export const R38_ALL_AFTER_UNDO = readPage(pageAllAfterUndo);
/** What the receipt picker gets for "arshad": the dead person, inactive, tagged dead. */
export const SEARCH_INCLUDE_DEAD = readHits(searchIncludeDead);

/** The request the backend's own test sent to move ارشد: the date as typed, the reason and a note. */
export const MARK_ARSHAD_REQUEST = readMarkInput(markArshadRequest);
/** The request of the picker search of the receipt field: dead people asked for. */
export const SEARCH_INCLUDE_DEAD_REQUEST = readRequest(searchIncludeDeadRequest);

/** The one-time people the receipt screen gets for "nasir": Nasir, in the dead amount (`is_dead`). */
export const PEOPLE_INCLUDE_DEAD = readPeople(pukhtaIncludeDead);
export const PEOPLE_INCLUDE_DEAD_REQUEST = readRequest(pukhtaIncludeDeadRequest);
/** The same search for any other field: no dead people, so nothing is found. */
export const PEOPLE_DEAD_HIDDEN = readPeople(pukhtaDeadHidden);
export const PEOPLE_DEAD_HIDDEN_REQUEST = readRequest(pukhtaDeadHiddenRequest);
