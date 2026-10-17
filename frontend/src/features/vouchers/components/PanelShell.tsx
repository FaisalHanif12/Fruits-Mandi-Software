// The frame of a helper panel under the voucher form (open bills, payables, advances): a card with
// a title, and the four states of a read (PRD FR-7.58 to FR-7.60, Code Standards F8): loading, an
// error with a retry, nothing to show (with words), and the data, drawn by the panel.

import type { UseQueryResult } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { Card, ErrorState, LoadingState } from '../../../shared/ui';

interface PanelShellProps<T> {
  query: UseQueryResult<T>;
  /** The title of the card. */
  title: ReactNode;
  /** The words shown when the read failed. */
  failed: ReactNode;
  /** Draws the data. */
  children: (data: T) => ReactNode;
}

export function PanelShell<T>({ query, title, failed, children }: PanelShellProps<T>) {
  return (
    <Card compact title={title}>
      {query.isPending ? (
        <LoadingState rows={2} />
      ) : query.isError ? (
        <ErrorState
          title={failed}
          error={uiErrorOf(query.error)}
          onRetry={() => {
            void query.refetch();
          }}
        />
      ) : (
        children(query.data)
      )}
    </Card>
  );
}
