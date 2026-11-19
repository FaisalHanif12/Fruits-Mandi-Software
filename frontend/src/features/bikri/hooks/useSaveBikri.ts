// The save of a بکری (PRD FR-12.8, FR-6.7, plan 13). One call, all or nothing: the backend writes
// the بکری, its journal, the partner split, the stock, the matching and the audit row in one
// transaction, and removes the draft. The screen sends the draft as typed and always sends
// `confirm_cash_below_zero`. After a save, everything that reads what the save wrote is read again
// (PRD 6.2): the balances on the party cards, the trial balance, the partners' shares, the lists.

import { useMutation, useQueryClient, type QueryClient } from '@tanstack/react-query';
import { uiErrorOf } from '../../../shared/api';
import { draftsDiscard } from '../../../shared/drafts';
import type { BikriSaved } from '../../../bindings/BikriSaved';
import type { MixedSaleSaved } from '../../../bindings/MixedSaleSaved';
import { bikriEditSave, bikriMixedSave, bikriSave } from '../api';
import { BIKRI_DRAFT_SCREEN } from '../draft-snapshot';
import type { BikriDraft } from '../types';

export interface SaveBikriInput {
  draft: BikriDraft;
  /** True after the user said yes to "is this entry correct?" (PRD FR-6.7). */
  confirmCashBelowZero: boolean;
}

/** The queries that show what a saved بکری changed (PRD 6.2). */
const READ_AGAIN: readonly (readonly string[])[] = [
  ['bikri'],
  ['accounts'],
  ['coa'],
  ['partners'],
  ['search'],
  ['day_status'],
  // A lot of trip goods takes its quantity off the purchase (PRD FR-12.26, FR-12.39).
  ['purchases'],
  ['trips'],
];

/** Reads again everything a posting changed (PRD 6.2). Used after a save, an edit, a cancel, an undo. */
export async function readAgainAfterPosting(queryClient: QueryClient): Promise<void> {
  await Promise.all(
    READ_AGAIN.map((queryKey) => queryClient.invalidateQueries({ queryKey: [...queryKey] })),
  );
}

export function useSaveBikri() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ draft, confirmCashBelowZero }: SaveBikriInput) =>
      bikriSave(draft, confirmCashBelowZero),
    onSuccess: async () => {
      // The backend removed the draft in its transaction. A draft sent a moment before the save
      // may have arrived after it, so the screen asks for it to be thrown away once more.
      await draftsDiscard(BIKRI_DRAFT_SCREEN).catch(() => null);
      await readAgainAfterPosting(queryClient);
    },
    // Two open new sales can hold the same ready goods: the second save is refused with what is
    // left (ERR_QTY_EXCEEDS_LOT, PRD FR-12.40). The goods ready to sell are asked for again, so the
    // next new بکری and the panel of the lot show what is really left. Nothing typed is changed.
    onError: (error) => {
      const refused = uiErrorOf(error).errors.some(
        (item) =>
          item.code === 'ERR_QTY_EXCEEDS_LOT' && (item.field ?? '').endsWith('qty_received'),
      );
      if (refused) void queryClient.invalidateQueries({ queryKey: ['bikri', 'ready-goods'] });
    },
  });
}

/**
 * The save of an edit of one posting (PRD FR-12.19). `reason` is read when the save is sent, so
 * the reason typed in the box that asks for it is the one that goes.
 */
export function useSaveEdit(entryId: number, version: number, reason: () => string | null) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ draft, confirmCashBelowZero }: SaveBikriInput) =>
      bikriEditSave(entryId, version, draft, reason(), confirmCashBelowZero),
    onSuccess: () => readAgainAfterPosting(queryClient),
  });
}

/**
 * What a mixed save gives back, in the shape of a plain save so the screen's messages, print and last-saved
 * link work on it as they do for any بکری: the new بکری when the screen made one, else the old بکری of the
 * first carried lot. `mixed` is what the backend said, which the undo and the day print use.
 */
export interface MixedSaved extends BikriSaved {
  mixed: MixedSaleSaved;
}

/** Is this the answer of a mixed save? (A plain save has no `mixed`.) */
export function isMixedSaved(saved: BikriSaved): saved is MixedSaved {
  return 'mixed' in saved;
}

/** The mixed answer as a `MixedSaved`. Nothing is worked out: the numbers are the backend's. */
export function asBikriSaved(answer: MixedSaleSaved): MixedSaved {
  const first = answer.carried[0];
  return {
    // The new بکری is the one to show and print; with none, the old بکری of the first carried lot.
    bikri_id: answer.new_bikri_id ?? first?.old_bikri_id ?? 0,
    number: answer.new_bikri_number ?? first?.old_number ?? 0,
    // The new entry is written last, after the carried ones.
    entry_id: answer.entry_ids.at(-1) ?? 0,
    preview: answer.preview,
    mixed: answer,
  };
}

/**
 * The save of the carried lots and the new lots together (plan 76 point 1, PRD FR-12.17, FR-12.48): one call,
 * one transaction, one cash question for all of it. After it everything that reads what it wrote is read
 * again, as after any save.
 */
export function useSaveMixed() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ draft, confirmCashBelowZero }: SaveBikriInput): Promise<MixedSaved> =>
      asBikriSaved(
        await bikriMixedSave(
          {
            date: draft.header.entry_date,
            carried: draft.carried ?? [],
            new_lots: draft.lots,
          },
          confirmCashBelowZero,
        ),
      ),
    onSuccess: async () => {
      await draftsDiscard(BIKRI_DRAFT_SCREEN).catch(() => null);
      await readAgainAfterPosting(queryClient);
    },
  });
}
