// The rule list (route /index/rules; PRD 11, FR-7.17): filters by side, charge, party, item and
// active, the rows in a grid with the winning level of each, one button for a new rule and one
// for "Test a rule". Keys: F2 new, ↑ ↓ Page Up and Page Down between rows, Enter opens a row, Esc
// goes back. Four states: loading, error with a retry, empty with a button, rows.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { FlaskConical, Plus, Scale, SearchX } from 'lucide-react';
import { useMemo, useState } from 'react';
import { uiErrorOf } from '../../../shared/api';
import { ListGrid } from '../../../shared/grid';
import { useFocusFirstField } from '../../../shared/hooks';
import { SHORTCUTS, formNavigationProps, useEscape, useShortcut } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  Checkbox,
  EmptyState,
  ErrorState,
  Field,
  LoadingState,
  Picker,
  Select,
  type PickerValue,
} from '../../../shared/ui';
import { ALL_CHARGES, SIDES } from '../allowed';
import { RULE_COLUMNS } from '../columns';
import { useRuleList } from '../hooks';
import type { ChargeType, RuleFilter, RuleRow, RuleSide } from '../types';

const SHORT_LIST_ROWS = 30;
const ALL = 'all';

export function RulesListPage() {
  const navigate = useNavigate();
  const [side, setSide] = useState<RuleSide | typeof ALL>(ALL);
  const [charge, setCharge] = useState<ChargeType | typeof ALL>(ALL);
  const [party, setParty] = useState<PickerValue | null>(null);
  const [item, setItem] = useState<PickerValue | null>(null);
  const [showInactive, setShowInactive] = useState(false);
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
  const data = list.data;

  let body;
  if (data === undefined && list.isPending) {
    body = <LoadingState />;
  } else if (list.isError) {
    body = (
      <ErrorState
        title={<Localized id="rules-load-failed" />}
        error={uiErrorOf(list.error)}
        onRetry={() => {
          void list.refetch();
        }}
      />
    );
  } else if (data?.length === 0) {
    body = filtering ? (
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
  } else if (data !== undefined) {
    body = (
      <ListGrid<RuleRow>
        columns={RULE_COLUMNS}
        rows={data}
        getRowId={(row) => String(row.id)}
        labelId="rules-title"
        onOpen={(row) => {
          void navigate({ to: '/index/rules/$id', params: { id: String(row.id) } });
        }}
        heightMode={data.length <= SHORT_LIST_ROWS ? 'auto' : 'fill'}
        isInactive={(row) => !row.is_active}
      />
    );
  }

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
          {data === undefined || data.length === 0 ? null : newButton}
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
              scope="any"
              value={party}
              showCard={false}
              onChange={(hit) => {
                setParty(
                  hit === null
                    ? null
                    : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                );
              }}
            />
          )}
        </Field>
        <Field label={<Localized id="rules-filter-item" />}>
          {(props) => (
            <Picker
              {...props}
              scope="item"
              value={item}
              onChange={(hit) => {
                setItem(
                  hit === null
                    ? null
                    : { id: hit.id, code: hit.code, name_ur: hit.name_ur, name_en: hit.name_en },
                );
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
