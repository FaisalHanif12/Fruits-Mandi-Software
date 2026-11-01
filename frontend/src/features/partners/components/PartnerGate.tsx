// The door of the partner section (plan 44, PRD FR-17.11): the screens of شراکت دار open only with
// the 4-digit code. The first time the code is made, every time after it is asked. While the section
// is open the screen shows; hiding a screen is not enough, the backend refuses every partner command
// while it is closed (NFR-SEC-14).

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ErrorState, LoadingState } from '../../../shared/ui';
import { usePartnerCodeStatus } from '../code-hooks';
import { PartnerCodeFlow } from './PartnerCodeFlow';

export function PartnerGate({ children }: { children: ReactNode }) {
  const status = usePartnerCodeStatus();
  if (status.isPending) return <LoadingState />;
  if (status.isError) {
    return (
      <ErrorState
        title={<Localized id="partners-code-failed" />}
        error={uiErrorOf(status.error)}
        onRetry={() => {
          void status.refetch();
        }}
      />
    );
  }
  // A code that is not made yet is made first, with no way round (PRD FR-17.11): the backend leaves
  // the section open until then, so the screen must ask.
  if (status.data.is_set && status.data.unlocked) return <>{children}</>;
  return (
    <div className="flex max-w-page flex-col gap-4">
      <PartnerCodeFlow />
    </div>
  );
}
