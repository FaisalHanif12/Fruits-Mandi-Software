// The list of one master (route /index/<kind>), the same screen for all nine (plan 06): a filter box
// that searches both scripts, a "show inactive" tick, the rows in a grid, and one button for a new
// one. Keys (PRD FR-7.17): F2 new, ↑ ↓ and Page Up / Page Down between rows, Enter opens the row,
// Esc goes back to the menu. Four states: loading, error with a retry, empty with a button, rows.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useParams } from '@tanstack/react-router';
import { Plus, SearchX } from 'lucide-react';
import { useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useFocusFirstField, useLoadRest } from '../../../shared/hooks';
import { PagedList } from '../../../shared/grid';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import { Button, Checkbox, EmptyState, ErrorState, Field, TextInput } from '../../../shared/ui';
import { columnsOf, configOfPath, type MasterConfig } from '../configs';
import { useMasterFilter, useMasterList } from '../hooks';
import { useMasterFocus } from '../store';
import type { MasterRecord } from '../types';
import { UnknownMasterPage } from './UnknownMasterPage';

/** How many pages (100 masters each) are read to find the row last opened; then the focus starts on the first row. */
const REMEMBERED_ROW_PAGES = 20;

export function MasterListPage() {
  const params = useParams({ strict: false });
  const config = configOfPath(params.kind ?? '');
  if (config === undefined) return <UnknownMasterPage />;
  return <MasterList key={config.kind} config={config} />;
}

function MasterList({ config }: { config: MasterConfig }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const [filterText, setFilterText] = useState('');
  const [showInactive, setShowInactive] = useState(false);
  // The list holds the inactive masters too; the tick only shows or hides them, with no new call.
  const list = useMasterList(config.kind, true);
  const filter = useMasterFilter(config.kind, filterText);
  const remember = useMasterFocus((state) => state.remember);
  const focusRowId = useMasterFocus((state) => state.rows[config.kind]);
  // The grid puts the focus back on the row last opened or saved when it is drawn, so that row must
  // be loaded first: while it is on a later page, the pages are read until it is (no filter typed).
  const findingRow =
    focusRowId !== undefined &&
    filter.ids === null &&
    list.hasNextPage &&
    (list.data?.pages.length ?? 0) < REMEMBERED_ROW_PAGES &&
    !list.data?.pages.some((page) => page.rows.some((record) => record.id === focusRowId));
  // A typed filter is matched against the rows, so while it is typed the rest of the pages load.
  useLoadRest(list, filter.ids !== null || findingRow);
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const columns = useMemo(() => columnsOf(config), [config]);

  const openNew = () => {
    void navigate({ to: '/index/$kind/new', params: { kind: config.path } });
  };
  const open = (record: MasterRecord) => {
    remember(config.kind, record.id);
    void navigate({ to: '/index/$kind/$id', params: { kind: config.path, id: String(record.id) } });
  };
  useShortcut(SHORTCUTS.newEntry, openNew);
  useEscape('leave', () => {
    void navigate({ to: '/index' });
  });

  const first = list.data?.pages[0];
  const rows = useMemo(() => {
    const all = list.data?.pages.flatMap((page) => page.rows);
    if (all === undefined) return undefined;
    const loaded = showInactive ? all : all.filter((record) => record.is_active);
    if (filter.ids === null) return loaded;
    const byId = new Map(loaded.map((record) => [record.id, record]));
    return filter.ids.flatMap((id) => byId.get(id) ?? []);
  }, [list.data, showInactive, filter.ids]);
  // While the filter is on its way, or the rest of the pages it needs, there is nothing to say yet.
  const settling = filter.isPending || (filter.ids !== null && list.hasNextPage);

  const newButton = (
    <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={openNew}>
      <Plus aria-hidden="true" className="size-icon-md" />
      <Localized id={config.newId} />
    </Button>
  );

  // No master at all (the list with the inactive ones is empty) or no match for what is typed.
  const nothingAtAll = first?.total_count === 0 && filter.ids === null;
  const empty = nothingAtAll ? (
    <EmptyState
      icon={config.icon}
      title={<Localized id={config.emptyId} />}
      help={<Localized id="masters-empty-help" vars={{ key: SHORTCUTS.newEntry.label }} />}
      action={newButton}
    />
  ) : (
    <EmptyState
      icon={SearchX}
      title={<Localized id="masters-no-match-title" />}
      help={<Localized id="masters-no-match-help" />}
      action={
        <Button
          onClick={() => {
            setFilterText('');
            setShowInactive(true);
          }}
        >
          <Localized id="masters-no-match-clear" />
        </Button>
      }
    />
  );
  const body = filter.isError ? (
    <ErrorState
      title={<Localized id="masters-filter-failed" />}
      error={uiErrorOf(filter.error)}
      onRetry={() => {
        void filter.refetch();
      }}
    />
  ) : (
    <PagedList<MasterRecord>
      columns={columns}
      // Nothing is drawn while the filter settles, so "no match" never flashes.
      rows={list.isError || findingRow || (settling && rows?.length === 0) ? undefined : rows}
      totalCount={
        filter.ids !== null
          ? rows?.length
          : showInactive
            ? first?.total_count
            : first?.totals.active_count
      }
      getRowId={(record) => String(record.id)}
      labelId={config.titleId}
      failedId="masters-load-failed"
      error={list.isError ? uiErrorOf(list.error) : null}
      onRetry={() => {
        void list.refetch();
      }}
      hasMore={list.hasNextPage && filter.ids === null}
      loadingMore={list.isFetchingNextPage}
      onLoadMore={() => {
        void list.fetchNextPage();
      }}
      empty={empty}
      onOpen={open}
      isInactive={(record) => !record.is_active}
      focusRowId={focusRowId === undefined ? null : String(focusRowId)}
      onFocusRow={(rowId) => {
        remember(config.kind, Number(rowId));
      }}
    />
  );

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id={config.titleId}>
            <h1 className="type-title" />
          </Localized>
          <Localized id={config.helpId}>
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        {/* An empty list has the same button in its empty state. */}
        {first === undefined || first.total_count === 0 ? null : newButton}
      </div>
      <div ref={containerRef} {...formNavigationProps} className="flex flex-wrap items-end gap-4">
        <div className="min-w-0 flex-1 basis-field">
          <Field label={<Localized id="masters-filter-label" />}>
            {(props) => (
              <TextInput
                {...props}
                value={filterText}
                onChange={setFilterText}
                placeholder={l10n.getString('masters-filter-placeholder')}
              />
            )}
          </Field>
        </div>
        <Checkbox
          checked={showInactive}
          onChange={setShowInactive}
          label={<Localized id="masters-show-inactive" />}
        />
      </div>
      <div className="min-h-0 flex-1">{body}</div>
    </div>
  );
}
