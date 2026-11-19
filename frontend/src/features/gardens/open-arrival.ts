// Opens the garden screen of a goods arrived entry (PRD FR-12.40): the attention list names an
// entry for the garden goods that wait, and a row of the goods left report (R10) is one. Both give
// the id of the entry only; the backend says which garden it is in (`gardens_arrival_get`). If the
// entry cannot be read, the list of goods left opens instead, so the click still leads somewhere.
// Where no router is around (a component test, a gallery page) nothing opens and nothing fails.

import { useRouter } from '@tanstack/react-router';
import { gardensArrivalGet } from './api';

export function useOpenArrival(): (arrivalId: number) => void {
  const router = useRouter({ warn: false }) as ReturnType<typeof useRouter> | undefined;
  return (arrivalId) => {
    if (router === undefined) return;
    gardensArrivalGet(arrivalId)
      .then((row) => router.navigate({ to: '/gardens/$id', params: { id: String(row.garden.id) } }))
      .catch(() => router.navigate({ to: '/bikri/stock' }));
  };
}
