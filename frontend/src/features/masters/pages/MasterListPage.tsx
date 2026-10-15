// The list of one master (route /index/<kind>), the same screen for all ten (plan 06): a filter box
// that searches both scripts, a "show inactive" tick, the rows in a grid, and one button for a new
// one. Keys (PRD FR-7.17): F2 new, ↑ ↓ and Page Up / Page Down between rows, Enter opens the row,
// Esc goes back to the menu. Four states: loading, error with a retry, empty with a button, rows.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate, useParams } from '@tanstack/react-router';
import { Plus, SearchX } from 'lucide-react';
import { useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { useFocusFirstField } from '../../../shared/hooks';
import { ListGrid } from '../../../shared/grid';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  Checkbox,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  TextInput,
} from '../../../shared/ui';
import { columnsOf, configOfPath, type MasterConfig } from '../configs';
import { useMasterFilter, useMasterList } from '../hooks';
import { useMasterFocus } from '../store';
import type { MasterRecord } from '../types';
import { UnknownMasterPage } from './UnknownMasterPage';

/**
 * A list of this many rows or fewer is as tall as its rows and draws them all. A longer one fills
 * the height of the screen and draws only the rows in view (NFR-SPD-18).
 */
const SHORT_LIST_ROWS = 30;

export function MasterListPage() {
  const params = useParams({ strict: false });
  const config = configOfPath(params.kind ?? '');
  if (config === undefined) return <UnknownMasterPage />;
  return <MasterList key={config.kind} config={config} />;
}

function MasterList({ config }: { config: MasterConfig }) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const list = useMasterList(config.kind);
  const [filterText, setFilterText] = useState('');
  const [showInactive, setShowInactive] = useState(false);
  const filter = useMasterFilter(config.kind, filterText);
  const remember = useMasterFocus((state) => state.remember);
  const focusRowId = useMasterFocus((state) => state.rows[config.kind]);
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

  const all = list.data;
  const rows = useMemo(() => {
    if (all === undefined) return [];
    const visible = showInactive ? all : all.filter((record) => record.is_active);
    if (filter.ids === null) return visible;
    const byId = new Map(visible.map((record) => [record.id, record]));
    return filter.ids.flatMap((id) => byId.get(id) ?? []);
  }, [all, showInactive, filter.ids]);

  const newButton = (
    <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={openNew}>
      <Plus aria-hidden="true" className="size-icon-md" />
      <Localized id={config.newId} />
    </Button>
  );

  let body;
  if (list.isPending) {
    body = <LoadingState />;
  } else if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id="masters-load-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (filter.isError) {
    body = (
      <ErrorState
        title={<Localized id="masters-filter-failed" />}
        error={uiErrorOf(filter.error)}
        onRetry={() => {
          void filter.refetch();
        }}
      />
    );
  } else if (list.data.length === 0) {
    body = (
      <EmptyState
        icon={config.icon}
        title={<Localized id={config.emptyId} />}
        help={<Localized id="masters-empty-help" vars={{ key: SHORTCUTS.newEntry.label }} />}
        action={newButton}
      />
    );
  } else if (rows.length === 0 && !filter.isPending) {
    body = (
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
  } else {
    body = (
      <ListGrid<MasterRecord>
        columns={columns}
        rows={rows}
        getRowId={(record) => String(record.id)}
        labelId={config.titleId}
        onOpen={open}
        heightMode={rows.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
        isInactive={(record) => !record.is_active}
        focusRowId={focusRowId === undefined ? null : String(focusRowId)}
        onFocusRow={(rowId) => {
          remember(config.kind, Number(rowId));
        }}
      />
    );
  }

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
        {list.data === undefined || list.data.length === 0 ? null : newButton}
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
