// Opens one posting of a saved بکری for edit (PRD FR-12.18, plan 16). One call loads the posting
// (`bikri_open_for_edit`), one reads the saved بکری for the names the draft holds only by id, and
// the records are found, then the draft is put into new stores and a copy of every loaded cell is
// kept (the baseline), so the screen can mark what the user changes.
//
// The session is made once and kept: it is NOT under the `['bikri']` query key, because a save
// reads those queries again and that must never remake the stores under the user's hands. It is
// thrown away when the screen closes (`gcTime: 0`), so the next edit loads the posting fresh.

import { useQuery } from '@tanstack/react-query';
import type { EditableBikri } from '../../../bindings/EditableBikri';
import { bikriEditPreview, bikriGet, bikriOpenForEdit } from '../api';
import { applyBackendDraft, recordsOfSaved, resolveDraftRecords } from '../draft-load';
import { createBikriDraft, type BikriDraftStores } from '../draft-store';
import { captureBaseline, type Baseline } from '../edit-changes';
import type { BikriPreview } from '../types';

export interface EditSession {
  editable: EditableBikri;
  stores: BikriDraftStores;
  baseline: Baseline;
  /**
   * What the backend works out for the draft as it was loaded: the numbers of THIS entry before
   * any change, from the same function as the save. The stored lot totals cannot be used for
   * "before": they hold every posting of the lot, a later sale too, and the preview holds only
   * this entry. Null if the backend could not say.
   */
  before: BikriPreview | null;
}

async function openSession(bikriId: number, entryId: number): Promise<EditSession> {
  const [editable, detail] = await Promise.all([
    bikriOpenForEdit(bikriId, entryId),
    bikriGet(bikriId),
  ]);
  // The lots of the saved بکری carry the names of the sellers, buyers, items and packings.
  const saved = recordsOfSaved([...detail.lots, ...editable.stored], detail.zamindar);
  const records = await resolveDraftRecords(editable.draft, {
    accounts: saved.accounts,
    masters: saved.masters,
  });
  const stores = createBikriDraft(editable.draft.header.entry_date);
  applyBackendDraft(stores, editable.draft, records);
  const before = await bikriEditPreview(entryId, editable.draft).catch(() => null);
  return { editable, stores, baseline: captureBaseline(stores), before };
}

export function useEditSession(bikriId: number, entryId: number) {
  return useQuery({
    queryKey: ['edit-session', bikriId, entryId],
    queryFn: () => openSession(bikriId, entryId),
    staleTime: Infinity,
    gcTime: 0,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    retry: false,
  });
}
