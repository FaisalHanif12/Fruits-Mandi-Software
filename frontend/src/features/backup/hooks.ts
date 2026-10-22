// The backup commands as hooks (Code Standards F6). The state of the backups is read about once a
// minute, every two seconds while a copy or a restore runs, and again after each action that
// changes it (plan 18: there is no progress event).

import {
  keepPreviousData,
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query';
import {
  backupKeyConfirm,
  backupKeySetup,
  backupList,
  backupNow,
  backupRestore,
  backupRestorePreview,
  backupSettingsGet,
  backupSettingsSet,
  backupStatus,
  backupTestRestore,
} from './api';
import type {
  BackupKind,
  BackupNowInput,
  BackupRef,
  BackupSettingsInput,
  BackupStatus,
  RestoreInput,
  TestRestoreInput,
} from './types';

export const BACKUP_STATUS_KEY = ['backup_status'] as const;
const LIST_KEY = ['backup_list'] as const;
const SETTINGS_KEY = ['backup_settings'] as const;

/** How often the state is read: the status bar needs a fresh time about once a minute. */
export const STATUS_POLL_MS = 60_000;
/** While a copy or a restore runs, the screen follows it closely. */
export const STATUS_POLL_RUNNING_MS = 2_000;
/** A page of the list (Code Standards B31). */
export const LIST_PAGE_SIZE = 100;

/** How long to wait before the state is read again: sooner while a copy or a restore runs. */
export function statusPollMs(status: BackupStatus | undefined): number {
  return status?.running === true ? STATUS_POLL_RUNNING_MS : STATUS_POLL_MS;
}

/** The state of the backups. The status bar and the backup screen share this one query. */
export function useBackupStatus() {
  return useQuery({
    queryKey: BACKUP_STATUS_KEY,
    queryFn: backupStatus,
    refetchInterval: (query) => statusPollMs(query.state.data),
  });
}

/** The copies, page by page. `kind` null is all places. */
export function useBackupList(kind: BackupKind | null) {
  return useInfiniteQuery({
    queryKey: [...LIST_KEY, kind],
    queryFn: ({ pageParam }) => backupList({ kind, offset: pageParam, limit: LIST_PAGE_SIZE }),
    initialPageParam: 0,
    getNextPageParam: (last, pages) => {
      const loaded = pages.reduce((count, page) => count + page.rows.length, 0);
      return last.rows.length > 0 && loaded < last.total_count ? loaded : undefined;
    },
    placeholderData: keepPreviousData,
  });
}

function useRefreshBackups() {
  const queryClient = useQueryClient();
  return async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: BACKUP_STATUS_KEY }),
      queryClient.invalidateQueries({ queryKey: LIST_KEY }),
    ]);
  };
}

export function useBackupNow() {
  const refresh = useRefreshBackups();
  return useMutation({
    mutationFn: (input: BackupNowInput) => backupNow(input),
    onSettled: refresh,
  });
}

/**
 * What a restore of one copy would do. Asked only while `ref` is set. The key is sent as typed, or
 * null. Nothing is kept: a preview is always read fresh.
 */
export function useRestorePreview(ref: BackupRef | null, key: string | null) {
  return useQuery({
    queryKey: ['backup_restore_preview', ref?.kind, ref?.name, key],
    queryFn: () => {
      if (ref === null) throw new Error('No copy chosen.');
      return backupRestorePreview({ backup: ref, key });
    },
    enabled: ref !== null,
    staleTime: 0,
    gcTime: 0,
  });
}

/**
 * Restores a copy. The Owner is signed out by the backend, and every cached answer belongs to the
 * old data, so everything is read again: who is signed in comes first, and the gate then shows the
 * sign-in screen.
 */
export function useRestore() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: RestoreInput) => backupRestore(input),
    onSuccess: () => queryClient.invalidateQueries(),
  });
}

export function useTestRestore() {
  const refresh = useRefreshBackups();
  return useMutation({
    mutationFn: (input: TestRestoreInput) => backupTestRestore(input),
    onSettled: refresh,
  });
}

export function useBackupSettings() {
  return useQuery({ queryKey: SETTINGS_KEY, queryFn: backupSettingsGet });
}

export function useSaveBackupSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: BackupSettingsInput) => backupSettingsSet(input),
    onSuccess: async (view) => {
      queryClient.setQueryData(SETTINGS_KEY, view);
      await queryClient.invalidateQueries({ queryKey: BACKUP_STATUS_KEY });
    },
  });
}

export function useMakeBackupKey() {
  const refresh = useRefreshBackups();
  return useMutation({ mutationFn: backupKeySetup, onSettled: refresh });
}

export function useConfirmBackupKey() {
  const refresh = useRefreshBackups();
  return useMutation({
    mutationFn: (key: string) => backupKeyConfirm({ key }),
    onSuccess: refresh,
  });
}
