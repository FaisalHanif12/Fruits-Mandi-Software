// The things the user does to a lot in the stock list and to a saved بکری (plan 16): write goods
// off, undo a write-off, cancel a بکری, undo a save. Each is one call; afterwards everything that
// shows what it changed is read again (PRD 6.2): the stock list, the lists of بکری, the balances.

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { bikriCancel, bikriUndo, bikriWriteOff, bikriWriteOffUndo } from '../api';
import { readAgainAfterPosting } from './useSaveBikri';

export function useWriteOff() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ lotId, qty, reason }: { lotId: number; qty: string; reason: string }) =>
      bikriWriteOff(lotId, qty, reason),
    onSuccess: () => readAgainAfterPosting(queryClient),
  });
}

export function useWriteOffUndo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ moveId, reason }: { moveId: number; reason: string }) =>
      bikriWriteOffUndo(moveId, reason),
    onSuccess: () => readAgainAfterPosting(queryClient),
  });
}

export function useCancelBikri() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: { id: number; version: number; reason: string; confirm: boolean }) =>
      bikriCancel(input.id, input.version, input.reason, input.confirm),
    onSuccess: () => readAgainAfterPosting(queryClient),
  });
}

export function useUndoBikri() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: { id: number; confirm: boolean }) => bikriUndo(input.id, input.confirm),
    onSuccess: () => readAgainAfterPosting(queryClient),
  });
}
