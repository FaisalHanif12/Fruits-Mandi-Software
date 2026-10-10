// Waiting for data (design system 6, Code Standards F8). The skeleton rows appear only after
// 300 ms, so a fast screen never flashes (StatesUr board). Screen readers hear "please wait" at
// once.

import { Localized } from '@fluent/react';
import { useEffect, useState } from 'react';

/** Delay before the skeleton shows (design system 6). */
export const LOADING_SKELETON_DELAY_MS = 300;
const DEFAULT_ROWS = 4;

/** Bar widths per row, so the skeleton looks like real rows of different lengths. */
const MIDDLE_WIDTHS = ['w-3/5', 'w-2/5', 'w-1/2', 'w-1/3'] as const;

interface LoadingStateProps {
  rows?: number;
}

export function LoadingState({ rows = DEFAULT_ROWS }: LoadingStateProps) {
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const timer = window.setTimeout(() => {
      setVisible(true);
    }, LOADING_SKELETON_DELAY_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, []);

  return (
    <div role="status" aria-busy="true" className="flex flex-col py-2">
      <Localized id="common-loading">
        <span className="sr-only" />
      </Localized>
      {visible
        ? Array.from({ length: rows }, (_, row) => (
            <div
              key={row}
              data-testid="skeleton-row"
              className="grid h-control grid-cols-4 items-center gap-4 border-t border-border px-4 first:border-t-0"
            >
              <div className="h-skeleton rounded-full bg-skeleton" />
              <div className="col-span-2">
                <div
                  className={`h-skeleton rounded-full bg-skeleton ${MIDDLE_WIDTHS[row % MIDDLE_WIDTHS.length] ?? 'w-1/2'}`}
                />
              </div>
              <div className="h-skeleton w-3/4 justify-self-end rounded-full bg-skeleton" />
            </div>
          ))
        : null}
    </div>
  );
}
