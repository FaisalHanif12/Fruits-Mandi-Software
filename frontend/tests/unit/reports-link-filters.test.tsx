// A button under an answer, or any link, opens a report with its filters already set (plan 54 Section 6,
// PRD FR-7.65, FR-7.74): the Owner presses F9 and nothing else. Each report of the print questions
// (R35 the party's day slip, R5 the buyer bills, R13 the khata, R14 the statement, R6 the seller's lots,
// R34 the day summary, R2 the sales register), the thirteen reports that follow them (R31, R12, R29,
// R20, R24, R10, R11, R18, R36, R33, R37, R21, R8) and the reports of plan 55 Section 11.1 (R3, R4, R7,
// R9, R15, R16, R17, R19, R22, R23, R25, R26, R27, R28, R30, R32, R38, and R8, R33, R36, R37 in their
// other forms) are opened here from an address that carries their
// party and dates, and the request the screen sends must hold exactly those, in Urdu and in English. The
// boxes must show them too, so what the Owner sees is what the print will have. R31 sits behind the
// partner code: its link opens the code box and shows no figure while the code is not entered.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportId } from '../../src/bindings/ReportId';
import * as partnersApi from '../../src/features/partners/api';
import * as reportsApi from '../../src/features/reports/api';
import { expectOnlyCollectors } from '../helpers/collector-box';
import { STAFF_ROSTER, fakeMasters } from '../helpers/masters';
import { parseReportSearch } from '../../src/features/reports/search';
import { LANGUAGES } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { fakeAccountSearch } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { PARTNER_CODE_STATUS_LOCKED } from '../helpers/edit-real';
import { usePartnerCodeDialog } from '../../src/shared/ui';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';
import { PAGE_ARSHAD } from '../helpers/dead';
import { CATALOG, fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
// Only the status of the partner code is answered here; every other partner call stays as it was.
vi.mock('../../src/features/partners/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/features/partners/api')>()),
  partnersCodeStatus: vi.fn(),
}));
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  fakeAccountSearch([]);
  vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_LOCKED);
});

afterEach(() => {
  // The code box is one for the whole app and keeps its state between tests.
  usePartnerCodeDialog.getState().hide();
  leaveAppWindow();
});

const lastRun = () => vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];

const MONTH = { from: '01-10-2026', to: '31-10-2026' };
const DAY = '10-10-2026';

/**
 * The link of each row of plan 54 Section 6.2, as an address, and what the screen must ask for. A
 * buyer's or a party's report takes `party`; the seller's lots (R6) take `seller`; the day slip (R35) takes
 * one day as `as_of`; the day summary (R34) has no party.
 */
const LINKS: readonly {
  id: ReportId;
  asked: string;
  search: string;
  filters: Record<string, unknown>;
}[] = [
  {
    id: 'R35',
    asked: "a buyer's day slip",
    search: `party=20&as_of=${DAY}`,
    filters: { party: { account_id: 20, person_key: null }, as_of: DAY },
  },
  {
    id: 'R5',
    asked: "a buyer's bills for a month",
    search: `party=20&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { party: { account_id: 20, person_key: null }, ...MONTH },
  },
  {
    id: 'R13',
    asked: "a party's khata for a month",
    search: `party=20&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { party: { account_id: 20, person_key: null }, ...MONTH },
  },
  {
    id: 'R14',
    asked: "a party's statement for a month",
    search: `party=20&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { party: { account_id: 20, person_key: null }, ...MONTH },
  },
  {
    id: 'R6',
    asked: "a seller's lots for a day",
    search: `seller=31&from=${DAY}&to=${DAY}`,
    filters: { seller_id: 31, from: DAY, to: DAY },
  },
  {
    id: 'R34',
    asked: 'the whole day, no party',
    search: `from=${DAY}&to=${DAY}`,
    filters: { from: DAY, to: DAY },
  },
  {
    id: 'R2',
    asked: 'the full sales register for a range',
    search: `from=${MONTH.from}&to=${MONTH.to}`,
    filters: { ...MONTH },
  },
  // The reports that follow (plan 54 Section 6.3): a partner, a period, an as-of day, a city, an adda.
  {
    id: 'R31',
    asked: "a partner's statement for a month",
    search: `partner=3&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { partner_id: 3, ...MONTH },
  },
  {
    id: 'R12',
    asked: 'the cash book for a month',
    search: `from=${MONTH.from}&to=${MONTH.to}`,
    filters: { ...MONTH },
  },
  {
    id: 'R29',
    asked: 'the expense report for a month, all partners',
    search: `from=${MONTH.from}&to=${MONTH.to}`,
    filters: { ...MONTH, partner_id: null },
  },
  {
    id: 'R29',
    asked: "the expense report for a month, one partner's",
    search: `partner=3&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { ...MONTH, partner_id: 3 },
  },
  {
    id: 'R20',
    asked: 'the buyers’ dues as of a day',
    search: `as_of=${DAY}`,
    filters: { as_of: DAY },
  },
  {
    id: 'R24',
    asked: 'the sellers’ payables as of a day',
    search: `as_of=${DAY}`,
    filters: { as_of: DAY },
  },
  {
    id: 'R10',
    asked: 'the stock for a period (it takes a period, not an as-of day)',
    search: `from=${MONTH.from}&to=${MONTH.to}`,
    filters: { ...MONTH },
  },
  {
    id: 'R11',
    asked: 'the day book of one day',
    search: `from=${DAY}&to=${DAY}`,
    filters: { from: DAY, to: DAY },
  },
  {
    id: 'R18',
    asked: 'the trial balance as of a day',
    search: `as_of=${DAY}`,
    filters: { as_of: DAY },
  },
  {
    id: 'R36',
    asked: "one garden's money for a month",
    search: `garden=5&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { garden_id: 5, ...MONTH },
  },
  {
    id: 'R33',
    asked: 'the trips for a month (a period and a layout; there is no one-trip box)',
    search: `from=${MONTH.from}&to=${MONTH.to}&trip_view=day`,
    filters: { ...MONTH, trip_view: 'day' },
  },
  {
    id: 'R37',
    asked: "one partner's fruit for a month",
    search: `partner=3&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { partner_id: 3, ...MONTH },
  },
  {
    id: 'R21',
    asked: 'the dues of one city as of a day',
    search: `city=7&as_of=${DAY}`,
    filters: { city_id: 7, as_of: DAY },
  },
  {
    id: 'R8',
    asked: "one adda's trucks for a month",
    search: `adda=9&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { adda_id: 9, ...MONTH },
  },
  // Plan 55 Section 11.1: the assistant opens every report of the catalogue with what the question named.
  // A period, and a buyer or a seller named (R3, R4, R7, R15, R23, R25, R28 take `party`).
  ...(['R3', 'R4', 'R7', 'R15', 'R23', 'R25', 'R28'] as const).flatMap((id) => [
    {
      id,
      asked: 'the period only',
      search: `from=${MONTH.from}&to=${MONTH.to}`,
      filters: { ...MONTH },
    },
    {
      id,
      asked: 'the period and a party named',
      search: `party=20&from=${MONTH.from}&to=${MONTH.to}`,
      filters: { party: { account_id: 20, person_key: null }, ...MONTH },
    },
  ]),
  // The fruit register for the shop's guests only: `guests` and no partner (R37 takes a partner or the guests).
  {
    id: 'R37',
    asked: "the guests' fruit for a month",
    search: `guests=true&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { guests_only: true, partner_id: null, ...MONTH },
  },
  // An item named (plan 55 W3): R7, R27 and R10 take `item` with the period.
  ...(['R7', 'R27', 'R10'] as const).map((id) => ({
    id,
    asked: 'an item named, with the period',
    search: `item=5&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { item_id: 5, ...MONTH },
  })),
  // The sales of one group of items, fruit or vegetables (plan 55): R2, R3, R4, R7, R10 and R27.
  ...(['R2', 'R3', 'R4', 'R7', 'R10', 'R27'] as const).map((id) => ({
    id,
    asked: 'the sales of fruit only',
    search: `item_group=fruit&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { item_group: 'fruit', ...MONTH },
  })),
  // The stock by item (R10): the layout is carried, and sent only when it is not by lot.
  {
    id: 'R10',
    asked: 'the stock by item',
    search: `stock_view=item&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { stock_view: 'item', ...MONTH },
  },
  // A period only.
  ...(['R27', 'R30'] as const).map((id) => ({
    id,
    asked: 'the period only',
    search: `from=${MONTH.from}&to=${MONTH.to}`,
    filters: { ...MONTH },
  })),
  // The books as of a day (today when none is named): `as_of` only.
  ...(['R16', 'R17', 'R22', 'R26', 'R32'] as const).map((id) => ({
    id,
    asked: 'the books as of a day',
    search: `as_of=${DAY}`,
    filters: { as_of: DAY },
  })),
  // One collector, a staff member, named in the question (plan 55 Section 11.1): R22, R23 and R35.
  {
    id: 'R22',
    asked: 'the dues of one collector as of a day',
    search: `collector=12&as_of=${DAY}`,
    filters: { collector_id: 12, as_of: DAY },
  },
  {
    id: 'R23',
    asked: 'the receipts of one collector for a month',
    search: `collector=12&from=${MONTH.from}&to=${MONTH.to}`,
    filters: { collector_id: 12, ...MONTH },
  },
  {
    id: 'R35',
    asked: "one collector's day sheet",
    search: `collector=12&as_of=${DAY}`,
    filters: { collector_id: 12, as_of: DAY },
  },
  // No filter at all (R38 has a screen of its own: its test is below).
  { id: 'R19', asked: 'no filter', search: '', filters: {} },
  // All trips and all gardens, when only the word for them was said: the period, and for R33 the layout.
  {
    id: 'R33',
    asked: 'all trips, by trip',
    search: `from=${MONTH.from}&to=${MONTH.to}&trip_view=trip`,
    filters: { ...MONTH, trip_view: 'trip' },
  },
  {
    id: 'R36',
    asked: 'all gardens for a month',
    search: `from=${MONTH.from}&to=${MONTH.to}`,
    filters: { ...MONTH },
  },
  {
    id: 'R8',
    asked: 'the trucks of the month (no adda named)',
    search: `from=${MONTH.from}&to=${MONTH.to}`,
    filters: { ...MONTH },
  },
];

describe('the address of a report reads every filter the print questions need', () => {
  it.each(LINKS)('$id ($asked)', ({ search, filters }) => {
    const query = Object.fromEntries(new URLSearchParams(search));
    const read = parseReportSearch(query);
    // Whatever the link carries is read, and nothing it carries is dropped.
    expect(Object.keys(read).sort()).toEqual(Object.keys(query).sort());
    if ('from' in filters) expect(read.from).toBe(filters['from']);
    if ('to' in filters) expect(read.to).toBe(filters['to']);
    if ('as_of' in filters) expect(read.as_of).toBe(filters['as_of']);
  });

  it('knows the filters of each of the seven reports (the catalogue says which boxes there are)', () => {
    const boxes = (id: ReportId) => CATALOG.find((entry) => entry.id === id)?.filters ?? [];
    expect(boxes('R35')).toEqual(['as_of', 'party', 'collector']);
    for (const id of ['R5', 'R13', 'R14', 'R2'] as const) {
      expect(boxes(id)).toContain('date_range');
      expect(boxes(id)).toContain('party');
    }
    expect(boxes('R6')).toContain('seller');
    expect(boxes('R6')).toContain('date_range');
    expect(boxes('R34')).toEqual(['date_range']);
  });

  it('knows the boxes of the reports that follow: what each takes is what its link may carry', () => {
    const boxes = (id: ReportId) => CATALOG.find((entry) => entry.id === id)?.filters ?? [];
    expect(boxes('R31')).toEqual(['date_range', 'partner']);
    expect(boxes('R12')).toEqual(['date_range']);
    expect(boxes('R29')).toEqual(['date_range', 'partner']);
    expect(boxes('R20')).toEqual(['as_of', 'party', 'city']);
    expect(boxes('R24')).toEqual(['as_of', 'party']);
    // R10 is the stock over a period: it has no as-of box.
    expect(boxes('R10')).toEqual(['date_range', 'seller', 'item', 'item_group', 'stock_view']);
    expect(boxes('R11')).toEqual(['date_range', 'party']);
    expect(boxes('R18')).toEqual(['as_of']);
    expect(boxes('R36')).toContain('garden');
    // R33 takes a layout, not one trip: nothing in the catalogue or the filters names a trip.
    expect(boxes('R33')).toEqual(['date_range', 'item', 'trip_view']);
    expect(boxes('R37')).toEqual(expect.arrayContaining(['date_range', 'partner', 'guests']));
    expect(boxes('R21')).toEqual(['as_of', 'party', 'city']);
    expect(boxes('R8')).toEqual(['date_range', 'adda']);
  });

  it('knows the boxes of the reports of plan 55 Section 11.1: each takes what its link carries', () => {
    const boxes = (id: ReportId) => CATALOG.find((entry) => entry.id === id)?.filters ?? [];
    // A period, and `party` for a buyer or a seller who was named.
    for (const id of ['R3', 'R4', 'R7', 'R15', 'R23', 'R25', 'R28'] as const) {
      expect(boxes(id), id).toEqual(expect.arrayContaining(['date_range', 'party']));
    }
    for (const id of ['R27', 'R30'] as const) expect(boxes(id), id).toContain('date_range');
    // The books as of a day.
    for (const id of ['R16', 'R17', 'R22', 'R26', 'R32'] as const) {
      expect(boxes(id), id).toContain('as_of');
    }
    // No box at all: a link to these carries nothing.
    expect(boxes('R19')).toEqual([]);
    // R38 has only its status box; the question gives no filter for it.
    expect(boxes('R38')).toEqual(['status']);
    // All trips and all gardens: the period (and the layout of the trips); one partner's fruit.
    expect(boxes('R33')).toEqual(expect.arrayContaining(['date_range', 'trip_view']));
    expect(boxes('R36')).toContain('date_range');
    expect(boxes('R37')).toEqual(expect.arrayContaining(['date_range', 'partner']));
    expect(boxes('R8')).toContain('adda');
    // The group of the items (fruit or vegetables) on the sales reports.
    for (const id of ['R2', 'R3', 'R4', 'R7', 'R10', 'R27'] as const) {
      expect(boxes(id), id).toContain('item_group');
    }
    // The collector box of plan 55 Section 11.1.
    expect(boxes('R22')).toContain('collector');
    expect(boxes('R23')).toContain('collector');
    expect(boxes('R35')).toContain('collector');
  });
});

describe.each(LANGUAGES)(
  'a report opened from a link starts with its filters set (%s)',
  (language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    const m = (id: string) => message(language, id);

    it.each(LINKS)(
      '$id ($asked): the request holds the party and the dates, and so do the boxes',
      async (link) => {
        const entry = CATALOG.find((candidate) => candidate.id === link.id);
        fakeReports();
        await renderApp(`/reports/${link.id}?${link.search}`);
        await screen.findByRole('heading', {
          level: 1,
          name: (name) => visible(name).startsWith(m(entry?.title_id ?? '')),
        });
        await waitFor(() => {
          expect(lastRun()?.id).toBe(link.id);
        }, SLOW_MS);
        expect(lastRun()?.filters).toMatchObject(link.filters);
        // The boxes show the same dates, so the print is what the Owner sees.
        const wanted = [link.filters['from'], link.filters['to'], link.filters['as_of']].filter(
          (value): value is string => typeof value === 'string',
        );
        for (const date of wanted) {
          expect(
            screen.getAllByRole<HTMLInputElement>('textbox').some((box) => box.value === date),
          ).toBe(true);
        }
      },
    );

    it('R31 behind the partner code: the link shows the box that asks for the code, no figure and no failure, and the button opens the code box', async () => {
      const locked = refusal([{ code: 'ERR_PARTNER_LOCKED', field: null, values: {} }]);
      fakeReports({ refuse: (input) => (input.id === 'R31' ? locked : null) });
      const user = userEvent.setup();
      await renderApp(`/reports/R31?partner=3&from=${MONTH.from}&to=${MONTH.to}`);
      await waitFor(() => {
        expect(lastRun()?.id).toBe('R31');
      }, SLOW_MS);
      // The box that asks for the code stands in the place of the statement.
      await screen.findByText(
        (content) => visible(content) === m('partners-code-box-text'),
        undefined,
        SLOW_MS,
      );
      expect(visible(document.body.textContent)).not.toContain(m('err-partner-locked'));
      expect(screen.queryAllByRole('row')).toHaveLength(0);
      // Nothing is open yet: the Owner asks for the box, and it is the one box of the app.
      expect(screen.queryByRole('dialog')).toBeNull();
      await user.click(
        screen.getByRole('button', { name: (name) => visible(name) === m('partners-code-enter') }),
      );
      expect(
        await screen.findByRole('dialog', {
          name: (name) => visible(name) === m('partners-code-title'),
        }),
      ).toBeInTheDocument();
    });

    it('a report with a collector box draws it, with its label, and the address fills it', async () => {
      fakeReports();
      await renderApp(`/reports/R23?collector=12&from=${MONTH.from}&to=${MONTH.to}`);
      const box = await screen.findByRole<HTMLInputElement>(
        'combobox',
        { name: (name) => visible(name) === m('report-filter-collector') },
        SLOW_MS,
      );
      expect(box).toBeInTheDocument();
      await waitFor(() => {
        expect(lastRun()?.filters).toMatchObject({ collector_id: 12 });
      }, SLOW_MS);
    });

    it('the collector box of R22 lists the collectors only: the collectors own search scope, never all staff', async () => {
      fakeReports();
      fakeMasters({ staff: STAFF_ROSTER });
      const user = userEvent.setup();
      await renderApp(`/reports/R22?as_of=${DAY}`);
      const box = await screen.findByRole<HTMLInputElement>(
        'combobox',
        { name: (name) => visible(name).includes(m('report-filter-collector')) },
        SLOW_MS,
      );
      await expectOnlyCollectors(user, box, language);
    });

    it('the item group box shows the group the address names, and "all" or none sends nothing', async () => {
      fakeReports();
      await renderApp(`/reports/R7?item_group=vegetable&from=${MONTH.from}&to=${MONTH.to}`);
      await waitFor(() => {
        expect(lastRun()?.filters).toMatchObject({ item_group: 'vegetable' });
      }, SLOW_MS);
      const box = await screen.findByRole('combobox', {
        name: (name) => visible(name).includes(m('common-item-group')),
      });
      expect(visible(box.textContent)).toContain(m('masters-item-group-vegetable'));
    });

    it.each(['item_group=all', 'item_group=meat', ''])(
      'no group is sent for "%s"',
      async (extra) => {
        fakeReports();
        await renderApp(`/reports/R7?${extra}&from=${MONTH.from}&to=${MONTH.to}`);
        await waitFor(() => {
          expect(lastRun()?.id).toBe('R7');
        }, SLOW_MS);
        expect(lastRun()?.filters).not.toHaveProperty('item_group');
      },
    );

    it('the stock layout box shows the layout the address names, and "lot", an unknown word or none sends nothing', async () => {
      fakeReports();
      await renderApp(`/reports/R10?stock_view=item&from=${MONTH.from}&to=${MONTH.to}`);
      await waitFor(() => {
        expect(lastRun()?.filters).toMatchObject({ stock_view: 'item' });
      }, SLOW_MS);
      const box = await screen.findByRole('combobox', {
        name: (name) => visible(name).includes(m('report-filter-stock-view')),
      });
      expect(visible(box.textContent)).toContain(m('report-filter-stock-view-item'));
    });

    it.each(['stock_view=lot', 'stock_view=pile', ''])(
      'no stock layout is sent for "%s"',
      async (extra) => {
        fakeReports();
        await renderApp(`/reports/R10?${extra}&from=${MONTH.from}&to=${MONTH.to}`);
        await waitFor(() => {
          expect(lastRun()?.id).toBe('R10');
        }, SLOW_MS);
        expect(lastRun()?.filters).not.toHaveProperty('stock_view');
      },
    );

    it('a report that takes no stock layout never sends one, even when the address names one', async () => {
      fakeReports();
      await renderApp(`/reports/R12?stock_view=item&from=${MONTH.from}&to=${MONTH.to}`);
      await waitFor(() => {
        expect(lastRun()?.id).toBe('R12');
      }, SLOW_MS);
      expect(lastRun()?.filters).not.toHaveProperty('stock_view');
    });

    it('a report that takes no group never sends one, even when the address names one', async () => {
      fakeReports();
      await renderApp(`/reports/R12?item_group=fruit&from=${MONTH.from}&to=${MONTH.to}`);
      await waitFor(() => {
        expect(lastRun()?.id).toBe('R12');
      }, SLOW_MS);
      expect(lastRun()?.filters).not.toHaveProperty('item_group');
    });

    it('a report that takes no collector never sends one, even when the address names one', async () => {
      fakeReports();
      await renderApp(`/reports/R12?collector=12&from=${MONTH.from}&to=${MONTH.to}`);
      await waitFor(() => {
        expect(lastRun()?.id).toBe('R12');
      }, SLOW_MS);
      expect(lastRun()?.filters).not.toHaveProperty('collector_id');
    });

    it('R37 with a partner and the guests together: the partner wins and the guests choice is never sent', async () => {
      fakeReports();
      await renderApp(`/reports/R37?partner=3&guests=true&from=${MONTH.from}&to=${MONTH.to}`);
      await waitFor(() => {
        expect(lastRun()?.id).toBe('R37');
      }, SLOW_MS);
      expect(lastRun()?.filters).toMatchObject({ partner_id: 3 });
      expect(lastRun()?.filters).not.toHaveProperty('guests_only');
    });

    it('R38 has a screen of its own: the link with no filter lands on it, and the report run is not asked', async () => {
      fakeReports();
      vi.mocked(reportsApi.reportsDeadAmount).mockResolvedValue(PAGE_ARSHAD);
      await renderApp('/reports/R38');
      await screen.findByRole(
        'heading',
        { level: 1, name: (name) => visible(name).startsWith(m('dead-report-title')) },
        SLOW_MS,
      );
      await waitFor(() => {
        expect(reportsApi.reportsDeadAmount).toHaveBeenCalled();
      }, SLOW_MS);
      expect(reportsApi.reportsRun).not.toHaveBeenCalled();
    });

    it('R13: the party box shows who he is, with his name, once the answer has come', async () => {
      fakeReports();
      await renderApp('/reports/R13?party=20&from=01-10-2026&to=31-10-2026');
      const box = await screen.findByRole<HTMLInputElement>(
        'combobox',
        { name: (name) => visible(name) === m('report-filter-party') },
        SLOW_MS,
      );
      await waitFor(() => {
        expect(visible(box.value)).toMatch(language === 'ur' ? /اکرم/ : /Akram/);
      }, SLOW_MS);
    });
  },
);
