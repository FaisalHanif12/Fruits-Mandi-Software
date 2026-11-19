// The words the app has learned from the user's choices (PRD FR-7.26, plan 05): when he picks
// another spelling than the first, the pair is remembered and offered first next time. The Owner
// sees them here and can remove one. Removing never changes a saved name. Four states: it loads,
// it fails with a message and a retry, it is empty (with what to do), or it shows the table.

import { Localized, useLocalization } from '@fluent/react';
import { Languages } from 'lucide-react';
import { useState } from 'react';
import type { PairRow } from '../../../bindings/PairRow';
import { uiErrorOf } from '../../../shared/api';
import { utcToLocalDisplay } from '../../../shared/format';
import { Pager } from '../../../shared/grid';
import {
  Button,
  Code,
  ConfirmDialog,
  EmptyState,
  ErrorNotice,
  ErrorState,
  LoadingState,
  Num,
  Table,
  showToast,
  type TableColumn,
} from '../../../shared/ui';
import { useLearnedPairs, useRemoveLearnedPair } from '../hooks';

const KIND_IDS = {
  name: 'translit-words-kind-name',
  place: 'translit-words-kind-place',
  item: 'translit-words-kind-item',
  word: 'translit-words-kind-word',
} as const;

export function LearnedWords() {
  const { l10n } = useLocalization();
  const pairs = useLearnedPairs();
  const remove = useRemoveLearnedPair();
  const [asking, setAsking] = useState<PairRow | null>(null);

  const body = () => {
    if (pairs.isError) {
      return (
        <ErrorState
          title={<Localized id="translit-words-load-failed" />}
          error={uiErrorOf(pairs.error)}
          onRetry={() => {
            void pairs.refetch();
          }}
        />
      );
    }
    if (pairs.isPending || pairs.data === undefined) return <LoadingState />;
    const rows = pairs.data.pages.flatMap((page) => page.rows);
    if (rows.length === 0) {
      return (
        <EmptyState
          icon={Languages}
          title={<Localized id="translit-words-empty-title" />}
          help={<Localized id="translit-words-empty-help" />}
        />
      );
    }
    const columns: TableColumn<PairRow>[] = [
      {
        id: 'roman',
        header: <Localized id="translit-words-col-roman" />,
        cell: (pair) => <Code>{pair.roman}</Code>,
      },
      {
        id: 'urdu',
        header: <Localized id="translit-words-col-urdu" />,
        cell: (pair) => <span lang="ur">{pair.urdu}</span>,
      },
      {
        id: 'kind',
        header: <Localized id="translit-words-col-kind" />,
        cell: (pair) => <Localized id={KIND_IDS[pair.kind]} />,
      },
      {
        id: 'count',
        header: <Localized id="translit-words-col-count" />,
        kind: 'number',
        size: 'qty',
        cell: (pair) => <Num>{String(pair.use_count)}</Num>,
      },
      {
        id: 'last',
        header: <Localized id="translit-words-col-last" />,
        cell: (pair) =>
          pair.last_used_at === null ? null : <Num>{utcToLocalDisplay(pair.last_used_at)}</Num>,
      },
      {
        id: 'remove',
        header: <Localized id="translit-words-remove" />,
        kind: 'action',
        cell: (pair) => (
          <Button
            onClick={() => {
              setAsking(pair);
            }}
          >
            <Localized id="translit-words-remove" />
          </Button>
        ),
      },
    ];
    return (
      <>
        <div className="rounded-3 border border-border bg-surface">
          <Table
            label={l10n.getString('translit-words-table')}
            columns={columns}
            rows={rows}
            rowKey={(pair) => String(pair.id)}
          />
        </div>
        <Pager pager={pairs.pager} />
      </>
    );
  };

  const removeError = remove.error === null ? null : uiErrorOf(remove.error);
  return (
    <section className="flex flex-col gap-3">
      <Localized id="translit-words-title">
        <h2 className="type-section" />
      </Localized>
      <Localized id="translit-words-help">
        <p className="type-body text-ink-muted" />
      </Localized>
      {removeError === null ? null : <ErrorNotice error={removeError} />}
      {body()}
      <ConfirmDialog
        open={asking !== null}
        onOpenChange={(open) => {
          if (!open) setAsking(null);
        }}
        tone="danger"
        title={<Localized id="translit-words-confirm-title" />}
        description={
          <Localized
            id="translit-words-confirm-description"
            vars={{ roman: asking?.roman ?? '', urdu: asking?.urdu ?? '' }}
          />
        }
        confirmLabel={<Localized id="translit-words-remove" />}
        onConfirm={() => {
          if (asking === null) return;
          remove.mutate(
            { id: asking.id },
            {
              onSuccess: () => {
                showToast('success', l10n.getString('translit-words-removed'));
              },
            },
          );
        }}
      />
    </section>
  );
}
