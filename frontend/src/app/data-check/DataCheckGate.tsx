// Holds the app back until the start-up data check passes (plan 03 Section 4.3, NFR-REL-7).
// While it runs: the loading state. When it fails: the whole window says so, with the reference
// number and a "Try again" button. When it passes: the screens. The shell places it in plan 04.

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import { uiErrorOf } from '../../shared/api';
import { DataCheckFailedState, ErrorState, LoadingState } from '../../shared/ui';
import { useDataStatus } from './useDataStatus';

interface DataCheckGateProps {
  children: ReactNode;
}

export function DataCheckGate({ children }: DataCheckGateProps) {
  const query = useDataStatus();
  const retry = () => {
    void query.refetch();
  };
  if (query.isError) {
    return (
      <ErrorState
        title={<Localized id="common-data-check-title" />}
        error={uiErrorOf(query.error)}
        onRetry={retry}
      />
    );
  }
  if (query.isPending || query.data.state === 'checking') return <LoadingState />;
  if (query.data.state === 'failed') {
    return (
      <DataCheckFailedState
        reference={query.data.reference ?? ''}
        onRetry={retry}
        retrying={query.isFetching}
      />
    );
  }
  return <>{children}</>;
}
