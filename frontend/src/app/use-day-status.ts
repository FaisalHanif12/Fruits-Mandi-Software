import { useQuery } from '@tanstack/react-query';
import { dayStatus } from './api';

/** Today's date and whether the day is open, for the status bar and the dashboard. */
export function useDayStatus() {
  return useQuery({ queryKey: ['day_status'], queryFn: dayStatus });
}
