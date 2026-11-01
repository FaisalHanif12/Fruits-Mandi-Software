// What the partner code needs from the user right now (plan 44, PRD FR-17.11): make the code (it is
// not set yet), enter it (it is set and the section is closed), or make a new one (forgotten). The
// status from the backend decides. When the section is open there is nothing to ask, and the flow
// shows nothing.

import { useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ErrorState, LoadingState } from '../../../shared/ui';
import { Localized } from '@fluent/react';
import { usePartnerCodeStatus } from '../code-hooks';
import { ResetCodeCard, SetCodeCard, UnlockCard } from './PartnerCodeCards';

interface PartnerCodeFlowProps {
  /** The section is open (the code was right, or it was just made). */
  onOpened?: () => void;
}

export function PartnerCodeFlow({ onOpened }: PartnerCodeFlowProps) {
  const status = usePartnerCodeStatus();
  const [forgot, setForgot] = useState(false);
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
  const { is_set: isSet, unlocked, wait_seconds: wait } = status.data;
  const done = onOpened === undefined ? {} : { onDone: onOpened };
  if (!isSet) return <SetCodeCard {...done} />;
  if (unlocked) return null;
  if (forgot) {
    return (
      <ResetCodeCard
        onBack={() => {
          setForgot(false);
        }}
        {...done}
      />
    );
  }
  return (
    <UnlockCard
      waitSeconds={wait}
      onForgot={() => {
        setForgot(true);
      }}
      {...(onOpened === undefined ? {} : { onOpened })}
    />
  );
}
