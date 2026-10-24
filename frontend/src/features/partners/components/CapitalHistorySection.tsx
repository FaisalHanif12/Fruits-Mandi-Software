// The list under the سرمایہ form (PRD 17.4, FR-19.11): the chosen partner's capital history,
// oldest first, page by page, with his capital balance now and the backend's totals of what he put
// in and took back. Four states: loading, an error with a retry, nothing yet, and the rows. Without
// a partner it says to choose one.

import { Localized } from '@fluent/react';
import { Landmark } from 'lucide-react';
import { uiErrorOf } from '../../../shared/api';
import { Card, EmptyState, Fact, Num } from '../../../shared/ui';
import { SignedNum } from '../../cashbook';
import { PagedList } from '../../../shared/grid';
import { CAPITAL_COLUMNS } from '../capital-columns';
import { useCapitalHistory } from '../money-hooks';

export function CapitalHistorySection({ partnerId }: { partnerId: number | null }) {
  const history = useCapitalHistory(partnerId);
  const first = history.data?.pages[0];
  const rows = history.data?.pages.flatMap((page) => page.rows);
  return (
    <Card compact title={<Localized id="partners-capital-history-title" />}>
      {partnerId === null ? (
        <EmptyState
          icon={Landmark}
          title={<Localized id="partners-capital-pick" />}
          help={<Localized id="partners-capital-pick-help" />}
        />
      ) : (
        <>
          {first === undefined ? null : (
            <dl className="grid grid-cols-3 gap-3">
              <Fact label={<Localized id="partners-capital-balance" />}>
                <SignedNum paisa={first.balance_paisa} />
              </Fact>
              <Fact label={<Localized id="partners-capital-invested" />}>
                <Num paisa={first.totals.invested_paisa} />
              </Fact>
              <Fact label={<Localized id="partners-capital-taken-back" />}>
                <Num paisa={first.totals.taken_back_paisa} />
              </Fact>
            </dl>
          )}
          <PagedList
            columns={CAPITAL_COLUMNS}
            rows={rows}
            totalCount={first?.total_count}
            getRowId={(row) => [row.kind, row.voucher_id ?? 0, row.entry_date].join('-')}
            labelId="partners-capital-label"
            failedId="partners-capital-failed"
            error={history.isError ? uiErrorOf(history.error) : null}
            onRetry={() => {
              void history.refetch();
            }}
            hasMore={history.hasNextPage}
            loadingMore={history.isFetchingNextPage}
            onLoadMore={() => {
              void history.fetchNextPage();
            }}
            empty={
              <EmptyState
                icon={Landmark}
                title={<Localized id="partners-capital-empty" />}
                help={<Localized id="partners-capital-empty-help" />}
              />
            }
          />
        </>
      )}
    </Card>
  );
}
