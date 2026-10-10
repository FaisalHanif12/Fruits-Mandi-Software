import { useQuery } from '@tanstack/react-query';
import { appInfo } from '../api';

/** The app version for the start screen. It never changes while the app runs. */
export function useAppInfo() {
  return useQuery({ queryKey: ['app_info'], queryFn: appInfo, staleTime: Infinity });
}
