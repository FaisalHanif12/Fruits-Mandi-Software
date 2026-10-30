// The filter boxes of an analytics view (PRD 18.1, FR-18.1): the period with its shortcuts and the two
// dates, the grouping of views D and G, the partner (F, G, I, J), the garden (I and others), the layout of the trips (J), the item,
// seller, buyer (and one person inside a one-time buyer account), city, اڈا, the side (both, seller,
// buyer), the trip, the goods (all, consigned, trip, garden) and the comparison. They combine freely. Each
// view shows the boxes the catalogue says it takes. The boxes are plain fields of the shared kit:
// arrows move between them, a picker searches both scripts and never converts English letters
// (Code Standards F40, F41).
//
// Something refused by the backend shows under its own box (PRD FR-7.6). A comparison needs both
// dates (ERR_REQUIRED on `filters.from` and `filters.to`); with a shortcut chosen those two boxes are
// not drawn, so the refusal shows under the comparison box instead. In a compact window the boxes
// that are used less (buyer, city, اڈا, side, trip, goods, comparison) wait behind "More filters", and open by
// themselves when one of them holds a value or a refusal (Code Standards F22).

import { Localized, useLocalization } from '@fluent/react';
import type { ReactNode, Ref } from 'react';
import type { UiError } from '../../../bindings/UiError';
import { useRememberedFlag } from '../../../shared/hooks';
import { formNavigationProps } from '../../../shared/keyboard';
import {
  AccountPicker,
  Button,
  DateInput,
  Field,
  Num,
  Picker,
  Select,
  fieldError,
  type PickerValue,
} from '../../../shared/ui';
import { GardenFilterSelect } from '../../gardens';
import { isPukhtaCode } from '../../pukhta_khata';
import { ReportPartnerSelect, ReportPersonPicker } from '../../reports';
import {
  COMPARE_CHOICES,
  GOODS_CHOICES,
  GROUP_BY_CHOICES,
  PERIOD_CHOICES,
  SIDE_CHOICES,
  TRIP_VIEW_CHOICES,
  periodMessageId,
  tripViewMessageId,
  type AnalyticsFilterState,
  type PeriodChoice,
} from '../filters';
import type {
  AnalyticsCompare,
  AnalyticsFilterKind,
  AnalyticsGoods,
  AnalyticsGroupBy,
  AnalyticsSide,
  AnalyticsTripView,
  AnalyticsView,
} from '../types';
import { AnalyticsTripSelect } from './AnalyticsTripSelect';

interface AnalyticsFilterBarProps {
  /** The view: the fruit taken (K) has no trip goods, so its goods box does not offer them. */
  view: AnalyticsView;
  takes: readonly AnalyticsFilterKind[];
  state: AnalyticsFilterState;
  onChange: (change: Partial<AnalyticsFilterState>) => void;
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

/** The fields of the folded boxes, so a refusal on one of them opens the fold. */
const FOLDED_FIELDS = [
  'filters.buyer',
  'filters.city_id',
  'filters.adda_id',
  'filters.side',
  'filters.trip_id',
  'filters.garden_id',
  'filters.goods',
  'filters.compare',
];

export function AnalyticsFilterBar({
  view,
  takes,
  state,
  onChange,
  error,
  containerRef,
}: AnalyticsFilterBarProps) {
  const { l10n } = useLocalization();
  const has = (kind: AnalyticsFilterKind) => takes.includes(kind);
  const custom = state.period === 'custom';
  // The gardens view (I) is about gardens, so its garden box stays in view; on the other views that
  // take it, the box is one of those that wait behind "More filters".
  const aboutGardens = has('garden') && !has('seller');

  const primary: ReactNode[] = [];
  const folded: ReactNode[] = [];

  if (has('period')) {
    primary.push(
      <Field
        key="period"
        label={<Localized id="report-filter-period" />}
        error={fieldError(error, 'filters.period')}
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
      primary.push(
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
  if (has('group_by')) {
    primary.push(
      <Field
        key="group_by"
        label={<Localized id="report-filter-group-by" />}
        error={fieldError(error, 'filters.group_by')}
      >
        {(control) => (
          <Select<AnalyticsGroupBy>
            {...control}
            value={state.groupBy}
            onChange={(groupBy) => {
              onChange({ groupBy });
            }}
            options={GROUP_BY_CHOICES.map((choice) => ({
              value: choice,
              label: l10n.getString(`analytics-group-by-${choice}`),
            }))}
          />
        )}
      </Field>,
    );
  }
  if (has('partner')) {
    // Views F, G and J: one partner or all of them (his money, his part of a trip).
    primary.push(
      <ReportPartnerSelect
        key="partner"
        partner={state.partner}
        required={false}
        error={fieldError(error, 'filters.partner_id')}
        onChange={(partner) => {
          onChange({ partner });
        }}
      />,
    );
  }
  if (has('trip_view')) {
    // View J: one row for each trip, purchase, item or day. It changes the rows, not the figures.
    primary.push(
      <Field
        key="trip_view"
        label={<Localized id="report-filter-trip-view" />}
        error={fieldError(error, 'filters.trip_view')}
      >
        {(control) => (
          <Select<AnalyticsTripView>
            {...control}
            value={state.tripView}
            onChange={(tripView) => {
              onChange({ tripView });
            }}
            options={TRIP_VIEW_CHOICES.map((choice) => ({
              value: choice,
              label: l10n.getString(tripViewMessageId(choice)),
            }))}
          />
        )}
      </Field>,
    );
  }
  if (has('trip') && has('trip_view')) {
    // The trips view is about trips, so its trip box stays in view; elsewhere it is one of the
    // boxes that wait behind "More filters".
    primary.push(
      <AnalyticsTripSelect
        key="trip"
        trip={state.trip}
        error={fieldError(error, 'filters.trip_id')}
        onChange={(trip) => {
          onChange({ trip });
        }}
      />,
    );
  }
  if (aboutGardens) {
    primary.push(
      <GardenFilterSelect
        key="garden"
        garden={state.garden}
        labelId="analytics-filter-garden"
        error={fieldError(error, 'filters.garden_id')}
        onChange={(garden) => {
          onChange({ garden });
        }}
      />,
    );
  }
  if (has('item')) {
    primary.push(
      <Field
        key="item"
        label={<Localized id="report-filter-item" />}
        error={fieldError(error, 'filters.item_id')}
      >
        {(control) => (
          <Picker
            {...control}
            canAdd={false}
            scope="item"
            value={state.item}
            onChange={(hit) => {
              onChange({ item: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  if (has('seller')) {
    primary.push(
      <Field
        key="seller"
        label={<Localized id="report-filter-seller" />}
        error={fieldError(error, 'filters.seller_id')}
      >
        {(control) => (
          <AccountPicker
            {...control}
            canAdd={false}
            scope="seller"
            showCard={false}
            value={state.seller}
            onChange={(hit) => {
              onChange({ seller: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  if (has('buyer')) {
    folded.push(
      <Field
        key="buyer"
        label={<Localized id="analytics-filter-buyer" />}
        error={fieldError(error, 'filters.buyer.account_id') ?? fieldError(error, 'filters.buyer')}
      >
        {(control) => (
          <AccountPicker
            {...control}
            canAdd={false}
            scope="buyer"
            showCard={false}
            value={state.buyer}
            onChange={(hit) => {
              // Another buyer has other people, so the person of the last one is let go.
              onChange({ buyer: valueOf(hit), person: null, personText: '' });
            }}
          />
        )}
      </Field>,
    );
    // A one-time buyer account keeps its lines per person: one of them can be picked (FR-13.5).
    if (state.buyer !== null && isPukhtaCode(state.buyer.code)) {
      const accountId = state.buyer.id;
      folded.push(
        <Field
          key="person"
          label={<Localized id="report-filter-person" />}
          error={fieldError(error, 'filters.buyer.person_key')}
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
        </Field>,
      );
    }
  }
  if (has('city')) {
    folded.push(
      <Field
        key="city"
        label={<Localized id="report-filter-city" />}
        error={fieldError(error, 'filters.city_id')}
      >
        {(control) => (
          <Picker
            {...control}
            canAdd={false}
            scope="city"
            value={state.city}
            onChange={(hit) => {
              onChange({ city: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  if (has('adda')) {
    folded.push(
      <Field
        key="adda"
        label={<Localized id="report-filter-adda" />}
        error={fieldError(error, 'filters.adda_id')}
      >
        {(control) => (
          <Picker
            {...control}
            canAdd={false}
            scope="adda"
            value={state.adda}
            onChange={(hit) => {
              onChange({ adda: valueOf(hit) });
            }}
          />
        )}
      </Field>,
    );
  }
  if (has('side')) {
    folded.push(
      <Field key="side" label={<Localized id="analytics-filter-side" />}>
        {(control) => (
          <Select<AnalyticsSide>
            {...control}
            value={state.side}
            onChange={(side) => {
              onChange({ side });
            }}
            options={SIDE_CHOICES.map((choice) => ({
              value: choice,
              label: l10n.getString(`analytics-side-${choice}`),
            }))}
          />
        )}
      </Field>,
    );
  }
  if (has('trip') && !has('trip_view')) {
    folded.push(
      <AnalyticsTripSelect
        key="trip"
        trip={state.trip}
        error={fieldError(error, 'filters.trip_id')}
        onChange={(trip) => {
          onChange({ trip });
        }}
      />,
    );
  }
  if (has('garden') && !aboutGardens) {
    folded.push(
      <GardenFilterSelect
        key="garden"
        garden={state.garden}
        labelId="analytics-filter-garden"
        error={fieldError(error, 'filters.garden_id')}
        onChange={(garden) => {
          onChange({ garden });
        }}
      />,
    );
  }
  if (has('goods')) {
    folded.push(
      <Field
        key="goods"
        label={<Localized id="analytics-filter-goods" />}
        error={fieldError(error, 'filters.goods')}
      >
        {(control) => (
          <Select<AnalyticsGoods>
            {...control}
            value={state.goods}
            onChange={(goods) => {
              onChange({ goods });
            }}
            options={GOODS_CHOICES.filter((choice) => view !== 'K' || choice !== 'trip').map(
              (choice) => ({
                value: choice,
                label: l10n.getString(`analytics-goods-${choice}`),
              }),
            )}
          />
        )}
      </Field>,
    );
  }
  if (has('compare')) {
    // A comparison needs both dates. With a shortcut the two date boxes are not drawn, so their
    // refusal shows here.
    const datesRefused = custom
      ? null
      : (fieldError(error, 'filters.from') ?? fieldError(error, 'filters.to'));
    folded.push(
      <Field
        key="compare"
        label={<Localized id="analytics-filter-compare" />}
        error={fieldError(error, 'filters.compare') ?? datesRefused}
      >
        {(control) => (
          <Select<AnalyticsCompare>
            {...control}
            value={state.compare}
            onChange={(compare) => {
              onChange({ compare });
            }}
            options={COMPARE_CHOICES.map((choice) => ({
              value: choice,
              label: l10n.getString(`analytics-compare-${choice.replaceAll('_', '-')}`),
            }))}
          />
        )}
      </Field>,
    );
  }

  const filledCount = [
    state.buyer !== null,
    state.city !== null,
    state.adda !== null,
    state.side !== 'both',
    state.trip !== null && !has('trip_view'),
    state.garden !== null && !aboutGardens,
    state.goods !== 'all',
    state.compare !== 'off',
  ].filter(Boolean).length;
  const filled =
    state.buyer !== null ||
    state.city !== null ||
    state.adda !== null ||
    state.side !== 'both' ||
    (state.trip !== null && !has('trip_view')) ||
    (state.garden !== null && !aboutGardens) ||
    state.goods !== 'all' ||
    state.compare !== 'off';
  // `fieldError` gives nothing (undefined) for a field with no refusal.
  const refused = FOLDED_FIELDS.some((field) => {
    const found = fieldError(error, field);
    return found !== undefined && found !== null;
  });
  const forced = filled || refused || (!custom && state.compare !== 'off' && error !== null);
  // The less used boxes wait behind "More filters" in every window: the period, the grouping and the
  // item are what is changed every day, and the chart and the table stay in view (plan 38, G3). The
  // choice is kept for the screen, and a box that holds a value or a refusal keeps the part open.
  const [more, setMore] = useRememberedFlag('analytics.more_filters', false);
  const open = more || forced;

  if (primary.length + folded.length === 0) return null;
  return (
    <div
      ref={containerRef}
      role="group"
      aria-label={l10n.getString('analytics-filters-label')}
      {...formNavigationProps}
      className="grid grid-cols-6 items-end gap-3"
    >
      {primary}
      {folded.length > 0 ? (
        <div key="more" className="flex items-end gap-2">
          <Button
            disabled={forced}
            onClick={() => {
              setMore(!more);
            }}
          >
            <Localized id={open ? 'report-filters-fewer' : 'report-filters-more'} />
          </Button>
          {forced ? (
            <Localized
              id="common-filters-set"
              vars={{ count: String(filledCount) }}
              elems={{ num: <Num /> }}
            >
              <span className="type-body text-ink-muted" />
            </Localized>
          ) : null}
        </div>
      ) : null}
      {open ? folded : null}
    </div>
  );
}
