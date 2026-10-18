// The save of a بکری (PRD FR-12.8, FR-6.7, plan 13). One call, all or nothing: the backend writes
// the بکری, its journal, the partner split, the stock, the matching and the audit row in one
// transaction, and removes the draft. The screen sends the draft as typed and always sends
// `confirm_cash_below_zero`. After a save, everything that reads what the save wrote is read again
// (PRD 6.2): the balances on the party cards, the trial balance, the partners' shares, the lists.

import { useMutation, useQueryClient, type QueryClient } from '@tanstack/react-query';
import { draftsDiscard } from '../../../shared/drafts';
import { bikriEditSave, bikriLaterSaleSave, bikriSave } from '../api';
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
  ['pukhta'],
  ['day_status'],
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
  });
}

/** The save of a later sale of one lot (PRD FR-12.17). The new lines are the first lot's lines. */
export function useSaveLaterSale(lotId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ draft, confirmCashBelowZero }: SaveBikriInput) =>
      bikriLaterSaleSave(
        lotId,
        draft.header.entry_date,
        draft.lots[0]?.sales ?? [],
        confirmCashBelowZero,
      ),
    onSuccess: () => readAgainAfterPosting(queryClient),
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
