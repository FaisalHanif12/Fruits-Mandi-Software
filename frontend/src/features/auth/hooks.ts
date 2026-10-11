// Sign-in state and actions as hooks (Code Standards F6). The backend is the one source of truth
// for who is signed in; the screens read it with auth_status and refresh it after each action.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useRef } from 'react';
import type { FirstRunInput } from '../../bindings/FirstRunInput';
import type { LoginInput } from '../../bindings/LoginInput';
import type { RecoverInput } from '../../bindings/RecoverInput';
import type { SessionUser } from '../../bindings/SessionUser';
import type { UnlockInput } from '../../bindings/UnlockInput';
import { useLanguageStore } from '../../shared/i18n';
import { useTextSizeStore } from '../../app/layout/text-size';
import { prefsSet } from '../settings/api';
import {
  authLock,
  authLogin,
  authLogout,
  authRecover,
  authStatus,
  authUnlock,
  setupFirstRun,
} from './api';

export const AUTH_STATUS_KEY = ['auth_status'] as const;

export function useAuthStatus() {
  return useQuery({ queryKey: AUTH_STATUS_KEY, queryFn: authStatus, staleTime: Infinity });
}

/** Everything that depends on who is signed in is read again. */
function useRefreshSession() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries();
}

/**
 * The language and the text size of a user who has just signed in (PRD FR-7.30, FR-7.11): his
 * own choices, saved on the computer, replace what the sign-in screen showed.
 */
function applyUserPreferences(user: SessionUser) {
  useLanguageStore.getState().setLanguage(user.language);
  useTextSizeStore.getState().setChosen(user.text_size);
}

export function useLogin() {
  const refresh = useRefreshSession();
  return useMutation({
    mutationFn: (input: LoginInput) => authLogin(input),
    onSuccess: async (user) => {
      applyUserPreferences(user);
      await refresh();
    },
  });
}

export function useUnlock() {
  const refresh = useRefreshSession();
  return useMutation({ mutationFn: (input: UnlockInput) => authUnlock(input), onSuccess: refresh });
}

export function useLock() {
  const refresh = useRefreshSession();
  return useMutation({ mutationFn: () => authLock(), onSuccess: refresh });
}

export function useLogout() {
  const refresh = useRefreshSession();
  return useMutation({ mutationFn: () => authLogout(), onSuccess: refresh });
}

export function useFirstRun() {
  return useMutation({ mutationFn: (input: FirstRunInput) => setupFirstRun(input) });
}

export function useRecover() {
  return useMutation({ mutationFn: (input: RecoverInput) => authRecover(input) });
}

/** Refreshes who is signed in after the first-run setup is finished. */
export function useRefreshAfterSetup() {
  return useRefreshSession();
}

/**
 * Saves the user's language and text size when he changes them, so the next start is as he left
 * it (PRD FR-7.30, FR-7.45). The values that arrived with the session are not saved again.
 */
export function useSavePreferences(user: SessionUser) {
  const language = useLanguageStore((state) => state.language);
  const textSize = useTextSizeStore((state) => state.chosen);
  const savedLanguage = useRef(user.language);
  const savedTextSize = useRef(user.text_size);
  useEffect(() => {
    if (language === savedLanguage.current) return;
    savedLanguage.current = language;
    void prefsSet({ key: 'language', value: language });
  }, [language]);
  useEffect(() => {
    if (textSize === savedTextSize.current) return;
    savedTextSize.current = textSize;
    void prefsSet({ key: 'text_size', value: String(textSize) });
  }, [textSize]);
}
