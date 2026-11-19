// The plan 22 commands as hooks (Code Standards F6): Day settings, number grouping, the books check,
// updates and the support file.

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import type { DaySettings } from '../../bindings/DaySettings';
import type { Grouping } from '../../bindings/Grouping';
import type { Settings } from '../../bindings/Settings';
import type { SupportInfoInput } from '../../bindings/SupportInfoInput';
import type { UpdateInstallInput } from '../../bindings/UpdateInstallInput';
import { useGroupingStore } from '../../shared/format';
import {
  healthLast,
  healthRun,
  settingsDayGet,
  settingsDaySet,
  settingsSetNumberGrouping,
  supportInfoSave,
  updatesCheck,
  updatesInstall,
} from './maintenance-api';
import { SETTINGS_KEY } from './hooks';

export const DAY_KEY = ['settings', 'day'] as const;
export const HEALTH_KEY = ['health', 'last'] as const;
export const UPDATE_CHECK_KEY = ['updates', 'check'] as const;

// The backup screen keeps its own copy of the closing time. These are its query keys (features/backup
// keeps them private); saving the time here must refresh that screen too.
const BACKUP_SETTINGS_KEY = ['backup_settings'] as const;
const BACKUP_STATUS_KEY = ['backup_status'] as const;

export function useDaySettings() {
  return useQuery({ queryKey: DAY_KEY, queryFn: settingsDayGet, staleTime: Infinity });
}

export function useSetDaySettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: DaySettings) => settingsDaySet(input),
    onSuccess: (saved) => {
      queryClient.setQueryData(DAY_KEY, saved);
      // The closing time is the same value as on the backup screen.
      void queryClient.invalidateQueries({ queryKey: BACKUP_SETTINGS_KEY });
      void queryClient.invalidateQueries({ queryKey: BACKUP_STATUS_KEY });
    },
  });
}

/**
 * Makes the saved number grouping the one in use. The grouping is read from `settings_get` at
 * sign-in, so every amount follows it from the first screen (PRD 21, Language).
 */
export function useApplyNumberGrouping(grouping: Grouping | undefined): void {
  const setGrouping = useGroupingStore((state) => state.setGrouping);
  useEffect(() => {
    if (grouping !== undefined) setGrouping(grouping);
  }, [grouping, setGrouping]);
}

export function useSetNumberGrouping() {
  const queryClient = useQueryClient();
  const setGrouping = useGroupingStore((state) => state.setGrouping);
  return useMutation({
    mutationFn: (grouping: Grouping) => settingsSetNumberGrouping({ grouping }),
    onSuccess: (saved) => {
      // Every amount on screen follows at once; the saved settings keep it for the next start.
      setGrouping(saved);
      queryClient.setQueryData<Settings>(SETTINGS_KEY, (old) =>
        old === undefined ? old : { ...old, number_grouping: saved },
      );
    },
  });
}

/** The result of the last books check, or null before any check (NFR-OBS-5). */
export function useLastHealth() {
  return useQuery({ queryKey: HEALTH_KEY, queryFn: healthLast });
}

export function useRunHealth() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => healthRun(),
    onSuccess: (result) => {
      queryClient.setQueryData(HEALTH_KEY, result);
    },
  });
}

/**
 * Whether a newer version is on offer. It goes to the network, so it is asked when the screen opens
 * and again only on the user's press; it is never kept (`gcTime` 0), so coming back asks again.
 */
export function useUpdateCheck() {
  return useQuery({
    queryKey: UPDATE_CHECK_KEY,
    queryFn: updatesCheck,
    staleTime: Infinity,
    gcTime: 0,
    retry: false,
  });
}

/** The start-up check of the app's own: asked once while the app runs, only to show a note. */
export const STARTUP_UPDATE_KEY = ['updates', 'startup'] as const;

/**
 * Whether a newer version is on offer, asked once when the app starts (plan 19, NFR-INS-3). It is only
 * a note on the dashboard and in the bell; installing stays in Settings. A check that cannot reach the
 * network is not shown as a problem: there is simply no note.
 */
export function useStartupUpdateCheck(enabled: boolean) {
  return useQuery({
    queryKey: STARTUP_UPDATE_KEY,
    queryFn: updatesCheck,
    enabled,
    staleTime: Infinity,
    gcTime: Infinity,
    retry: false,
  });
}

export function useInstallUpdate() {
  return useMutation({ mutationFn: (input: UpdateInstallInput) => updatesInstall(input) });
}

export function useSaveSupportInfo() {
  return useMutation({ mutationFn: (input: SupportInfoInput) => supportInfoSave(input) });
}
