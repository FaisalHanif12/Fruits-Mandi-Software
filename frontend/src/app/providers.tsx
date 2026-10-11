// App-wide providers (Tech Stack 5.3): backend data cache, then language and direction, and the
// one toast area of the app (PRD FR-7.7).

import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useState, type ReactNode } from 'react';
import { AUTH_STATUS_KEY } from '../features/auth/hooks';
import { uiErrorOf } from '../shared/api';
import { I18nProvider } from '../shared/i18n';
import { Toaster } from '../shared/ui';

/** A new data cache. Backend errors are final answers, so queries do not retry by themselves. */
export function createQueryClient(): QueryClient {
  // A command that answers "nobody is signed in" or "the app is locked" means what the screen
  // shows is out of date (the Owner made the user inactive, the idle lock came on). Who is signed
  // in is read again, and the gate shows the sign-in screen or the lock (backend contract, plan 04).
  const sessionChanged = (error: unknown) => {
    const codes = uiErrorOf(error).errors.map((item) => item.code);
    if (codes.includes('ERR_NOT_LOGGED_IN') || codes.includes('ERR_SESSION_LOCKED')) {
      void client.invalidateQueries({ queryKey: AUTH_STATUS_KEY });
    }
  };
  const client: QueryClient = new QueryClient({
    queryCache: new QueryCache({ onError: sessionChanged }),
    mutationCache: new MutationCache({ onError: sessionChanged }),
    defaultOptions: {
      queries: { retry: false, refetchOnWindowFocus: false },
      mutations: { retry: false },
    },
  });
  return client;
}

interface AppProvidersProps {
  children: ReactNode;
  queryClient?: QueryClient;
}

export function AppProviders({ children, queryClient }: AppProvidersProps) {
  const [client] = useState(() => queryClient ?? createQueryClient());
  return (
    <QueryClientProvider client={client}>
      <I18nProvider>
        {children}
        <Toaster />
      </I18nProvider>
    </QueryClientProvider>
  );
}
