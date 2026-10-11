import { useQuery } from '@tanstack/react-query';
import { appDataStatus } from '../api';

/** How long to wait before asking again while the start-up check runs (plan 03 Section 4.2b). */
export const DATA_STATUS_POLL_MS = 200;

/**
 * The state of the start-up data check. It asks again every 200 ms while the state is
 * `checking`, and stops once the answer is `ok` or `failed`. Retrying (refetch) asks once more.
 */
export function useDataStatus() {
  return useQuery({
    queryKey: ['app_data_status'],
    queryFn: appDataStatus,
    refetchInterval: (query) =>
      query.state.data?.state === 'checking' ? DATA_STATUS_POLL_MS : false,
  });
}
