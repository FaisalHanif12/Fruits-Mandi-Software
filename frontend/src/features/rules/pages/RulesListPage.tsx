// The rule list (route /index/rules; PRD 11, FR-7.17): filters by side, charge, party, item and
// active, the rows in a grid with the winning level of each, one button for a new rule and one
// for "Test a rule". Keys: F2 new, ↑ ↓ Page Up and Page Down between rows, Enter opens a row, Esc
// goes back. Four states: loading, error with a retry, empty with a button, rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { FlaskConical, Plus, Scale, SearchX } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { pickedOf } from '../../../shared/accounts';
import { uiErrorOf } from '../../../shared/api';
import { PagedList } from '../../../shared/grid';
import { useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import { Remembered, type MemoryProps } from '../../../shared/screen-state';
import {
  AccountPicker,
  Button,
  Checkbox,
  EmptyState,
  Field,
  Picker,
  Select,
  type PickerValue,
} from '../../../shared/ui';
import { ALL_CHARGES, SIDES } from '../allowed';
import { RULE_COLUMNS } from '../columns';
import { useRuleList } from '../hooks';
import { RULES_SCREEN, ruleChoicesFrom, ruleChoicesText } from '../remember';
import type { ChargeType, RuleFilter, RuleRow, RuleSide } from '../types';

const ALL = 'all';

/** The rule list starts as the Owner left its filters (PRD FR-7.45). */
export function RulesListPage() {
  return <Remembered screen={RULES_SCREEN}>{(memory) => <RulesBody {...memory} />}</Remembered>;
}

function RulesBody({ saved, remember }: MemoryProps) {
  const navigate = useNavigate();
  const start = useMemo(() => ruleChoicesFrom(saved), [saved]);
  const [side, setSide] = useState<RuleSide | typeof ALL>(start.side);
  const [charge, setCharge] = useState<ChargeType | typeof ALL>(start.charge);
  const [party, setParty] = useState<PickerValue | null>(start.party);
  const [item, setItem] = useState<PickerValue | null>(start.item);
  const [showInactive, setShowInactive] = useState(start.showInactive);
  useEffect(() => {
    remember(ruleChoicesText({ side, charge, party, item, showInactive }));
  }, [side, charge, party, item, showInactive, remember]);
  const filter = useMemo<RuleFilter>(
    () => ({
      side: side === ALL ? null : side,
      charge_type: charge === ALL ? null : charge,
      account_id: party?.id ?? null,
      item_id: item?.id ?? null,
      include_inactive: showInactive,
    }),
    [side, charge, party, item, showInactive],
  );
  const list = useRuleList(filter);
  const containerRef = useFocusFirstField<HTMLDivElement>();
  const filtering =
    filter.side !== null ||
    filter.charge_type !== null ||
    filter.account_id !== null ||
    filter.item_id !== null ||
    filter.include_inactive;

  const openNew = () => {
    void navigate({ to: '/index/rules/new' });
  };
  useShortcut(SHORTCUTS.newEntry, openNew);
  useEscape('leave', () => {
    void navigate({ to: '/index' });
  });

  const newButton = (
    <Button variant="primary" shortcut={SHORTCUTS.newEntry} onClick={openNew}>
      <Plus aria-hidden="true" className="size-icon-md" />
      <Localized id="rules-new" />
    </Button>
  );
  const first = list.data?.pages[0];
  // A refusal shows as the state, not as the rows of the filter before it.
  const rows = list.isError ? undefined : list.data?.pages.flatMap((page) => page.rows);
  const empty = filtering ? (
    <EmptyState
      icon={SearchX}
      title={<Localized id="rules-no-match-title" />}
      help={<Localized id="rules-no-match-help" />}
      action={
        <Button
          onClick={() => {
            setSide(ALL);
            setCharge(ALL);
            setParty(null);
            setItem(null);
            setShowInactive(true);
          }}
        >
          <Localized id="rules-no-match-clear" />
        </Button>
      }
    />
  ) : (
    <EmptyState
      icon={Scale}
      title={<Localized id="rules-empty" />}
      help={<Localized id="rules-empty-help" vars={{ key: SHORTCUTS.newEntry.label }} />}
      action={newButton}
    />
  );
  const body = (
    <PagedList<RuleRow>
      columns={RULE_COLUMNS}
      rows={rows}
      totalCount={first?.total_count}
      getRowId={(row) => String(row.id)}
      labelId="rules-title"
      failedId="rules-load-failed"
      error={list.isError ? uiErrorOf(list.error) : null}
      onRetry={() => {
        void list.refetch();
      }}
      hasMore={list.hasNextPage}
      loadingMore={list.isFetchingNextPage}
      onLoadMore={() => {
        void list.fetchNextPage();
      }}
      empty={empty}
      onOpen={(row) => {
        void navigate({ to: '/index/rules/$id', params: { id: String(row.id) } });
      }}
      isInactive={(row) => !row.is_active}
    />
  );

  return (
    <div className="flex h-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex flex-col gap-1">
          <Localized id="rules-title">
            <h1 className="type-title" />
          </Localized>
          <Localized id="rules-help">
            <p className="type-body text-ink-muted" />
          </Localized>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            onClick={() => {
              void navigate({ to: '/index/rules/test' });
            }}
          >
            <FlaskConical aria-hidden="true" className="size-icon-md" />
            <Localized id="rules-test-open" />
          </Button>
          {rows === undefined || rows.length === 0 ? null : newButton}
        </div>
      </div>
      <div
        ref={containerRef}
        {...formNavigationProps}
        className="grid grid-cols-1 items-end gap-4 normal:grid-cols-4"
      >
        <Field label={<Localized id="rules-filter-side" />}>
          {(props) => (
            <Select
              {...props}
              value={side}
              onChange={setSide}
              options={[
                { value: ALL, label: <Localized id="rules-all" /> },
                ...SIDES.map((option) => ({
                  value: option,
                  label: <Localized id={`rules-side-${option}`} />,
                })),
              ]}
            />
          )}
        </Field>
        <Field label={<Localized id="rules-filter-charge" />}>
          {(props) => (
            <Select
              {...props}
              value={charge}
              onChange={setCharge}
              options={[
                { value: ALL, label: <Localized id="rules-all" /> },
                ...ALL_CHARGES.map((option) => ({
                  value: option,
                  label: <Localized id={`rules-charge-${option}`} />,
                })),
              ]}
            />
          )}
        </Field>
        <Field label={<Localized id="rules-filter-party" />}>
          {(props) => (
            <AccountPicker
              {...props}
              // A filter looks for what is already there; it has nothing to add (plan 32, R8).
              canAdd={false}
              scope="any"
              value={party}
              onChange={(hit) => {
                setParty(hit === null ? null : pickedOf(hit));
              }}
            />
          )}
        </Field>
        <Field label={<Localized id="rules-filter-item" />}>
          {(props) => (
            <Picker
              {...props}
              // A filter looks for what is already there; it has nothing to add (plan 32, R8).
              canAdd={false}
              scope="item"
              value={item}
              onChange={(hit) => {
                setItem(hit === null ? null : pickedOf(hit));
              }}
            />
          )}
        </Field>
        <Checkbox
          checked={showInactive}
          onChange={setShowInactive}
          label={<Localized id="rules-show-inactive" />}
        />
      </div>
      <div className="flex min-h-0 flex-1 flex-col">{body}</div>
    </div>
  );
}
