// Settings as hooks (Code Standards F6). A failed call shows as an error state with its message.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import type { FirmSettings } from '../../bindings/FirmSettings';
import type { SetDefaultLanguageInput } from '../../bindings/SetDefaultLanguageInput';
import type { SetSecurityInput } from '../../bindings/SetSecurityInput';
import type { RemovePairInput } from '../../bindings/RemovePairInput';
import { translitPairsList, translitPairsRemove } from '../../shared/urdu/api';
import {
  settingsGet,
  settingsSetDefaultLanguage,
  settingsSetFirm,
  settingsSetSecurity,
} from './api';

export const SETTINGS_KEY = ['settings'] as const;
export function useSettings() {
  return useQuery({ queryKey: SETTINGS_KEY, queryFn: settingsGet, staleTime: Infinity });
}

export function useSetFirm() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: FirmSettings) => settingsSetFirm(input),
    onSuccess: async () => {
      // The firm names are also on the sign-in screen, so everything is read again.
      await queryClient.invalidateQueries();
    },
  });
}

export function useSetSecurity() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SetSecurityInput) => settingsSetSecurity(input),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SETTINGS_KEY }),
  });
}

export function useSetDefaultLanguage() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SetDefaultLanguageInput) => settingsSetDefaultLanguage(input),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SETTINGS_KEY }),
  });
}

export const TRANSLIT_PAIRS_KEY = ['translit_pairs'] as const;

/** The words the app has learned (PRD FR-7.26). Owner only. */
export function useLearnedPairs() {
  return useQuery({ queryKey: TRANSLIT_PAIRS_KEY, queryFn: translitPairsList });
}

export function useRemoveLearnedPair() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: RemovePairInput) => translitPairsRemove(input),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: TRANSLIT_PAIRS_KEY }),
  });
}
