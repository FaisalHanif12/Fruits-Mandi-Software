// The filter boxes of a report (PRD FR-19.1, FR-19.2, FR-19.10): the period with its shortcuts, the
// two dates, the day of a balance list, and the party, seller, زمیندار, item, city and اڈا pickers.
// Each report shows only the boxes the catalogue says it takes. The boxes are plain fields of the
// shared kit: arrows move between them (shared/keyboard), a picker searches both scripts and never
// converts English letters (Code Standards F40, F41). The party card is not drawn here: these are
// filters, not an entry, and the table is the answer.

import { Localized, useLocalization } from '@fluent/react';
import { useState, type ReactNode, type Ref } from 'react';
import type { ReportCatalogEntry } from '../../../bindings/ReportCatalogEntry';
import type { ReportFilterKind } from '../../../bindings/ReportFilterKind';
import type { ReportGroupBy } from '../../../bindings/ReportGroupBy';
import type { UiError } from '../../../bindings/UiError';
import { useLayoutStep } from '../../../shared/hooks/useLayoutStep';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  Checkbox,
  ControlAccountPicker,
  DateInput,
  Field,
  Picker,
  Select,
  fieldError,
  type PickerValue,
} from '../../../shared/ui';
import { isPukhtaCode } from '../../pukhta_khata';
import { partyBoxOf } from '../catalog';
import {
  GROUP_BY_CHOICES,
  PAID_CHOICES,
  PERIOD_CHOICES,
  periodMessageId,
  type PaidChoice,
  type PeriodChoice,
  type ReportFilterState,
} from '../filters';
import { ReportPartnerSelect } from './ReportPartnerSelect';
import { ReportPersonPicker } from './ReportPersonPicker';

/** A report with more boxes than this folds its record pickers behind "More filters" when compact. */
const FOLD_AFTER = 5;

/** The control of the people who can borrow from the shop (PRD 16.5, FR-10.4): the party of R32. */
const LENT_CONTROL_CODE = '0103';

interface ReportFilterBarProps {
  entry: ReportCatalogEntry;
  state: ReportFilterState;
  onChange: (change: Partial<ReportFilterState>) => void;
  /** The backend's refusal of the last run, so a wrong date shows under its box. */
  error: UiError | null;
  containerRef?: Ref<HTMLDivElement>;
}

/** A record the picker chose, as the value the screen keeps (both names, so a language switch is free). */
function valueOf(
  hit: { id: number; name_ur: string; name_en: string; code?: string | undefined } | null,
): PickerValue | null {
  return hit === null
    ? null
    : { id: hit.id, name_ur: hit.name_ur, name_en: hit.name_en, code: hit.code };
}

export function ReportFilterBar({
  entry,
  state,
  onChange,
  error,
  containerRef,
}: ReportFilterBarProps) {
  const { l10n } = useLocalization();
  const takes = (kind: ReportFilterKind) => entry.filters.includes(kind);
  const custom = state.period === 'custom';

  const boxes: ReactNode[] = [];
  // The pickers of records (seller, زمیندار, item, city, اڈا). A report with many boxes folds most of
  // them behind "More filters" in a compact window, where the rows would push the table out of view
  // (Code Standards F22). From the normal width up every box shows. A folded box is not drawn at
  // all, so the arrow keys never try to enter a box that cannot be seen.
  const pickers: ReactNode[] = [];
  if (takes('date_range')) {
    boxes.push(
      <Field
        key="period"
        label={<Localized id="report-filter-period" />}
        error={fieldError(error, 'filters.shortcut')}
      >
        {(control) => (
          <Select<PeriodChoice>
            {...control}
            value={state.period}
            onChange={(period) => {
              onChange({ period });
            }}
            options={PERIOD_CHOICES.map((choice) => ({
              value: choice,
              label: l10n.getString(periodMessageId(choice)),
            }))}
          />
        )}
      </Field>,
    );
    if (custom) {
      boxes.push(
        <Field
          key="from"
          label={<Localized id="report-filter-from" />}
          error={fieldError(error, 'filters.from')}
        >
          {(control) => (
            <DateInput
              {...control}
              value={state.from}
              onChange={(from) => {
                onChange({ from });
              }}
            />
          )}
        </Field>,
        <Field
          key="to"
          label={<Localized id="report-filter-to" />}
          error={fieldError(error, 'filters.to')}
        >
          {(control) => (
            <DateInput
              {...control}
              value={state.to}
              onChange={(to) => {
                onChange({ to });
              }}
            />
          )}
        </Field>,
      );
    }
  }
  if (takes('as_of')) {
    boxes.push(
      <Field
        key="as_of"
        label={<Localized id="report-filter-as-of" />}
        error={fieldError(error, 'filters.as_of')}
      >
        {(control) => (
          <DateInput
            {...control}
            value={state.asOf}
            onChange={(asOf) => {
              onChange({ asOf });
            }}
          />
        )}
      </Field>,
    );
  }
  if (takes('party')) {
    const box = partyBoxOf(entry.id);
    const personBox = (accountId: number | null) => (
      <Field
        key="person"
        label={<Localized id="report-filter-person" />}
        error={
          fieldError(error, 'filters.party.person_key') ??
          fieldError(error, 'filters.party.account_id') ??
          fieldError(error, 'filters.party')
        }
      >
        {(control) => (
          <ReportPersonPicker
            {...control}
            person={state.person}
            text={state.personText}
            accountId={accountId}
            onTextChange={(personText) => {
              onChange({ person: null, personText });
            }}
            onPick={(person) => {
              onChange({ person, personText: '' });
            }}
          />
        )}
      </Field>
    );
    if (box === 'one_time') {
      // The register of one-time people takes no other account: the box is the person himself.
      boxes.push(personBox(null));
    } else {
      boxes.push(
        <Field
          key="party"
          label={
            <Localized id={box === 'lent' ? 'report-filter-borrower' : 'report-filter-party'} />
          }
          error={
            fieldError(error, 'filters.party') ?? fieldError(error, 'filters.party.account_id')
          }
        >
          {(control) =>
            box === 'lent' ? (
              // The people who can borrow are the accounts of control 0103 (PRD 16.5, R32).
              <ControlAccountPicker
                {...control}
                controlCode={LENT_CONTROL_CODE}
                showCard={false}
                value={state.party}
                onChange={(hit) => {
                  onChange({ party: valueOf(hit), person: null, personText: '' });
                }}
              />
            ) : (
              <AccountPicker
                {...control}
                scope={box}
                showCard={false}
                value={state.party}
                onChange={(hit) => {
                  // Another account has other people, so the person of the last one is let go.
                  onChange({ party: valueOf(hit), person: null, personText: '' });
                }}
              />
            )
          }
        </Field>,
      );
      // A one-time account keeps its lines per person: one of them can be picked (FR-13.5).
      if (state.party !== null && isPukhtaCode(state.party.code)) {
        boxes.push(personBox(state.party.id));
      }
    }
  }
  for (const kind of ['seller', 'zamindar'] as const) {
    if (!takes(kind)) continue;
    pickers.push(
      <Field
        key={kind}
        label={<Localized id={`report-filter-${kind}`} />}
        error={fieldError(error, `filters.${kind}_id`)}
      >
        {(control) => (
          <AccountPicker
            {...control}
            scope="seller"
            showCard={false}
            value={state[kind]}
            onChange={(hit) => {
              onChange({ [kind]: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  for (const kind of ['item', 'city', 'adda'] as const) {
    if (!takes(kind)) continue;
    pickers.push(
      <Field
        key={kind}
        label={<Localized id={`report-filter-${kind}`} />}
        error={fieldError(error, `filters.${kind}_id`)}
      >
        {(control) => (
          <Picker
            {...control}
            scope={kind}
            value={state[kind]}
            onChange={(hit) => {
              onChange({ [kind]: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  if (takes('partner')) {
    // R31 cannot run without a partner (the catalogue says so); R29 takes one or all.
    boxes.push(
      <ReportPartnerSelect
        key="partner"
        partner={state.partner}
        required={entry.party_required}
        error={fieldError(error, 'filters.partner_id')}
        onChange={(partner) => {
          onChange({ partner });
        }}
      />,
    );
  }
  if (takes('detail')) {
    boxes.push(
      <div key="detail" className="flex items-end">
        <Checkbox
          checked={state.detail}
          onChange={(detail) => {
            onChange({ detail });
          }}
          label={<Localized id="report-filter-detail" />}
        />
      </div>,
    );
  }

  if (takes('status')) {
    boxes.push(
      <Field key="status" label={<Localized id="report-filter-status" />}>
        {(control) => (
          <Select<PaidChoice>
            {...control}
            value={state.paid}
            onChange={(paid) => {
              onChange({ paid });
            }}
            options={PAID_CHOICES.map((choice) => ({
              value: choice,
              label: l10n.getString(`report-status-${choice}`),
            }))}
          />
        )}
      </Field>,
    );
  }
  if (takes('group_by')) {
    boxes.push(
      <Field key="group_by" label={<Localized id="report-filter-group-by" />}>
        {(control) => (
          <Select<ReportGroupBy>
            {...control}
            value={state.groupBy}
            onChange={(groupBy) => {
              onChange({ groupBy });
            }}
            options={GROUP_BY_CHOICES.map((choice) => ({
              value: choice,
              label: l10n.getString(`report-group-by-${choice}`),
            }))}
          />
        )}
      </Field>,
    );
  }

  // A report with more than FOLD_AFTER boxes keeps its first picker in view when it has no party box
  // (the seller of R6) and folds the rest.
  const fold = boxes.length + pickers.length > FOLD_AFTER;
  const kept = fold ? (takes('party') ? 0 : 1) : pickers.length;
  const folded = pickers.slice(kept);
  const anyFolded = folded.length > 0;
  const folderFilled =
    state.seller !== null ||
    state.zamindar !== null ||
    state.item !== null ||
    state.city !== null ||
    state.adda !== null;
  // Open from the start when the address already filled one of the folded boxes.
  const [more, setMore] = useState(folderFilled);
  const shownMore = more || folderFilled;
  const compact = useLayoutStep() === 'compact';

  const visibleBoxes = [...boxes, ...pickers.slice(0, kept)];
  if (visibleBoxes.length === 0 && !anyFolded) return null;
  return (
    <div
      ref={containerRef}
      role="group"
      aria-label={l10n.getString('report-filters-label')}
      {...formNavigationProps}
      className="grid grid-cols-5 items-end gap-4"
    >
      {visibleBoxes}
      {anyFolded && compact ? (
        <div key="more" className="flex items-end">
          <Button
            disabled={folderFilled}
            onClick={() => {
              setMore(!more);
            }}
          >
            <Localized id={shownMore ? 'report-filters-fewer' : 'report-filters-more'} />
          </Button>
        </div>
      ) : null}
      {anyFolded && (!compact || shownMore) ? folded : null}
    </div>
  );
}
