// The partner code as hooks (plan 44, PRD FR-17.11, Code Standards F6). The status says whether a
// code is set, whether the section is open now, and how many seconds to wait after wrong tries.
// Opening or closing the section changes what every partner figure in the app shows, so every
// answer the screens hold is read again after it.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type { PartnerCodeChangeInput } from '../../bindings/PartnerCodeChangeInput';
import type { PartnerCodeResetInput } from '../../bindings/PartnerCodeResetInput';
import type { PartnerCodeSetInput } from '../../bindings/PartnerCodeSetInput';
import {
  partnersCodeChange,
  partnersCodeLock,
  partnersCodeReset,
  partnersCodeSet,
  partnersCodeStatus,
  partnersCodeUnlock,
} from './api';

export const PARTNER_CODE_KEY = ['partners', 'code'] as const;

/** Where the partner code stands now. Read again after every change, never kept. */
export function usePartnerCodeStatus() {
  return useQuery({
    queryKey: PARTNER_CODE_KEY,
    queryFn: partnersCodeStatus,
    staleTime: 0,
  });
}

/** After the section opens or closes: every figure on screen is asked again. */
function useRefreshEverything() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries();
}

export function useUnlockPartners() {
  const refresh = useRefreshEverything();
  return useMutation({
    mutationFn: (code: string) => partnersCodeUnlock(code),
    onSettled: () => refresh(),
  });
}

export function useLockPartners() {
  const refresh = useRefreshEverything();
  return useMutation({
    mutationFn: () => partnersCodeLock(),
    onSettled: () => refresh(),
  });
}

export function useSetPartnerCode() {
  const refresh = useRefreshEverything();
  return useMutation({
    mutationFn: (input: PartnerCodeSetInput) => partnersCodeSet(input),
    onSettled: () => refresh(),
  });
}

export function useChangePartnerCode() {
  const refresh = useRefreshEverything();
  return useMutation({
    mutationFn: (input: PartnerCodeChangeInput) => partnersCodeChange(input),
    onSettled: () => refresh(),
  });
}

export function useResetPartnerCode() {
  const refresh = useRefreshEverything();
  return useMutation({
    mutationFn: (input: PartnerCodeResetInput) => partnersCodeReset(input),
    onSettled: () => refresh(),
  });
}
