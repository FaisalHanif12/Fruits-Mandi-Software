// Typed wrappers around the باغات commands (plan 17c; module `gardens`, Tech Stack 5.2).
// Components never call invoke() (Code Standards F4). The only file that knows the wire shape.
// Every command takes one `{ input }` object, except the two that take nothing.

import { callCommand } from '../../shared/api';
import type {
  GardenCloseInput,
  GardenClosePreview,
  GardenClosed,
  GardenCreatePreview,
  GardenCreated,
  GardenDefaults,
  GardenDetail,
  GardenEntryCancelInput,
  GardenEntryCancelled,
  GardenEntryInput,
  GardenEntryPreview,
  GardenEntrySaved,
  GardenGetInput,
  GardenInput,
  GardenReopenInput,
  GardenReopened,
  GardenRow,
  GardenSetPartnersInput,
  GardensList,
  GardensListInput,
  GardensSummary,
} from './types';

/** The form of a new garden, filled from the firm's shares or the last garden (PRD A56). */
export function gardensDefaults(): Promise<GardenDefaults> {
  return callCommand<GardenDefaults>('gardens_defaults');
}

/** What making the garden would do: the first installment's parts and the cash after (C23). */
export function gardensCreatePreview(input: GardenInput): Promise<GardenCreatePreview> {
  return callCommand<GardenCreatePreview>('gardens_create_preview', { input });
}

/** Makes a garden: the shares total 100% and each partner has a source (PRD FR-12.29). */
export function gardensCreate(input: GardenInput): Promise<GardenCreated> {
  return callCommand<GardenCreated>('gardens_create', { input });
}

/** New shares for a garden that has no entry yet. */
export function gardensSetPartners(input: GardenSetPartnersInput): Promise<GardenRow> {
  return callCommand<GardenRow>('gardens_set_partners', { input });
}

/** One page of gardens and the totals over the whole filter (PRD FR-19.11). */
export function gardensList(input: GardensListInput): Promise<GardensList> {
  return callCommand<GardensList>('gardens_list', { input });
}

/** One garden with a page of its history and each partner's money. */
export function gardensGet(input: GardenGetInput): Promise<GardenDetail> {
  return callCommand<GardenDetail>('gardens_get', { input });
}

/** The data of the dashboard tile باغات (PRD 9.1). */
export function gardensSummary(): Promise<GardensSummary> {
  return callCommand<GardensSummary>('gardens_summary');
}

/** Works out an installment, a cost or a sale in another shop as typed (C23, FR-12.37). */
export function gardensEntryPreview(input: GardenEntryInput): Promise<GardenEntryPreview> {
  return callCommand<GardenEntryPreview>('gardens_entry_preview', { input });
}

export function gardensEntrySave(input: GardenEntryInput): Promise<GardenEntrySaved> {
  return callCommand<GardenEntrySaved>('gardens_entry_save', { input });
}

export function gardensEntryCancel(input: GardenEntryCancelInput): Promise<GardenEntryCancelled> {
  return callCommand<GardenEntryCancelled>('gardens_entry_cancel', { input });
}

/** What closing the garden would do: each partner's money back (C26). */
export function gardensClosePreview(input: GardenCloseInput): Promise<GardenClosePreview> {
  return callCommand<GardenClosePreview>('gardens_close_preview', { input });
}

export function gardensClose(input: GardenCloseInput): Promise<GardenClosed> {
  return callCommand<GardenClosed>('gardens_close', { input });
}

/** Undoes the closing by reversal while its day is open (PRD FR-12.38). */
export function gardensReopen(input: GardenReopenInput): Promise<GardenReopened> {
  return callCommand<GardenReopened>('gardens_reopen', { input });
}
