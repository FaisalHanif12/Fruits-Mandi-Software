// The guard around every screen (plan 38 G8, Code Standards F8). A screen that cannot draw what the
// backend sent (a wrong-shaped answer, a bug) shows our own error state with a retry, in the app
// language, never the framework's red error page. The reference number is shown when the error has
// one (NFR-REL-6).

import { Localized } from '@fluent/react';
import type { ErrorComponentProps } from '@tanstack/react-router';
import { uiErrorOf } from '../shared/api';
import { ErrorState } from '../shared/ui';

export function RouteError({ error, reset }: ErrorComponentProps) {
  return (
    <ErrorState
      title={<Localized id="common-screen-failed" />}
      error={uiErrorOf(error)}
      onRetry={reset}
    />
  );
}
