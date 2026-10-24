// The line under a list that loads page by page: how many rows are on screen of how many the whole
// filter has (the backend's count), and a button for the next page (PRD FR-19.11, Code Standards
// F34). Used by `PagedList`, and by a screen that draws its rows itself.

import { Localized } from '@fluent/react';
import { Button } from '../ui';

interface PagedFooterProps {
  /** How many rows are on screen. */
  shown: number;
  /** How many rows the whole filter has. */
  total: number;
  hasMore: boolean;
  loadingMore: boolean;
  onLoadMore: () => void;
}

export function PagedFooter({ shown, total, hasMore, loadingMore, onLoadMore }: PagedFooterProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-2">
      <Localized id="common-list-count" vars={{ shown: String(shown), total: String(total) }}>
        <p className="type-body text-ink-muted" />
      </Localized>
      {hasMore ? (
        <Button disabled={loadingMore} busy={loadingMore} onClick={onLoadMore}>
          <Localized id="common-list-more" />
        </Button>
      ) : null}
    </div>
  );
}
