// The history of one entry and the stamps of several (plan 44, PRD FR-7.71), as query hooks.

import { useQuery } from '@tanstack/react-query';
import type { EditEntity } from '../../bindings/EditEntity';
import { auditEditHistory, auditEditStamp } from './api';

export const EDIT_KEY = ['edit'] as const;

/** Every version of one entry. Read only when the dialog is open. */
export function useEditHistory(entity: EditEntity, id: number, enabled: boolean) {
  return useQuery({
    queryKey: [...EDIT_KEY, 'history', entity, id],
    queryFn: () => auditEditHistory(entity, id),
    enabled,
  });
}

/** The last edit of each entry of a list that does not carry its own `edited` stamp. */
export function useEditStamps(entity: EditEntity, ids: readonly number[]) {
  return useQuery({
    queryKey: [...EDIT_KEY, 'stamps', entity, ...ids],
    queryFn: () => auditEditStamp(entity, ids),
    enabled: ids.length > 0,
  });
}

/** The last edit of one entry (an account, a rule), or null when it was never edited. */
export function useEntryStamp(entity: EditEntity, id: number | null) {
  const stamps = useEditStamps(entity, id === null ? [] : [id]);
  return stamps.data?.find((row) => row.id === id)?.stamp ?? null;
}
