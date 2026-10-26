// What the report, analytics and list screens remember (round 9, PRD FR-7.45): the filters the Owner
// last used are put back when a screen opens, each change is kept after a pause, clearing forgets, an
// address with filters in it wins, and a value that cannot be read any more is left out. Nothing about
// money is kept. The reading helpers are proved on their own, then the screens against a fake backend.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  analyticsFromRemembered,
  rememberedAnalytics,
} from '../../src/features/analytics/remember';
import { EMPTY_ANALYTICS_FILTERS } from '../../src/features/analytics/filters';
import { EMPTY_AUDIT_FILTERS } from '../../src/features/audit/filters';
import { auditFromRemembered, rememberedAudit } from '../../src/features/audit/remember';
import * as bikriApi from '../../src/features/bikri/api';
import {
  NO_STOCK_CHOICES,
  NO_TODAY_CHOICES,
  stockChoicesFrom,
  stockChoicesText,
  todayChoicesFrom,
  todayChoicesText,
} from '../../src/features/bikri/remember';
import {
  NO_STATEMENT_CHOICES,
  statementChoicesFrom,
  statementChoicesText,
} from '../../src/features/partners/remember';
import {
  NO_REGISTER_CHOICES,
  registerChoicesFrom,
  registerChoicesText,
} from '../../src/features/pukhta_khata/remember';
import {
  NO_ACCOUNT_CHOICES,
  accountChoicesFrom,
  accountChoicesText,
} from '../../src/features/masters/remember';
import {
  NO_RULE_CHOICES,
  ruleChoicesFrom,
  ruleChoicesText,
} from '../../src/features/rules/remember';
import {
  NO_TRIP_CHOICES,
  tripChoicesFrom,
  tripChoicesText,
} from '../../src/features/trips/remember';
import { EMPTY_FILTERS } from '../../src/features/reports/filters';
import {
  filtersFromRemembered,
  reportScreen,
  rememberedFilters,
} from '../../src/features/reports/remember';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as screenApi from '../../src/shared/screen-state/api';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { fakePlan16 } from '../helpers/bikri-plan16';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeReports } from '../helpers/reports';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const get = vi.mocked(screenApi.prefsScreenGet);
const set = vi.mocked(screenApi.prefsScreenSet);

const AKRAM = { id: 20, name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax', code: '0020' };

describe('what a report keeps of its filters', () => {
  it('keeps nothing for a report with nothing chosen, so a cleared report forgets', () => {
    expect(rememberedFilters(EMPTY_FILTERS)).toBeNull();
  });

  it('keeps what was chosen, and reads it back the same, never the one-time person or money', () => {
    const state = {
      ...EMPTY_FILTERS,
      period: 'this_month' as const,
      party: AKRAM,
      detail: true,
      voucherType: 'payment' as const,
      tripView: 'partner' as const,
      partner: 2,
      personText: 'shah',
    };
    const kept = rememberedFilters(state);
    expect(kept).toEqual({
      period: 'this_month',
      party: AKRAM,
      detail: true,
      voucherType: 'payment',
      tripView: 'partner',
      partner: 2,
    });
    expect(JSON.stringify(kept)).not.toContain('personText');
    expect(filtersFromRemembered(JSON.parse(JSON.stringify(kept)))).toMatchObject({
      period: 'this_month',
      party: AKRAM,
      detail: true,
      voucherType: 'payment',
      tripView: 'partner',
      partner: 2,
      person: null,
      personText: '',
    });
  });

  it('leaves out what cannot be read any more and starts those boxes empty', () => {
    const read = filtersFromRemembered({
      period: 'next_century',
      from: 12,
      party: { id: 'x' },
      paid: 'maybe',
      voucherType: 'gift',
      tripView: 'galaxy',
      partner: -4,
      detail: 'yes',
    });
    expect(read).toEqual(EMPTY_FILTERS);
    expect(filtersFromRemembered('text')).toBeNull();
    expect(filtersFromRemembered(null)).toBeNull();
  });

  it('names a report’s screen the way the backend example does: report.r20', () => {
    expect(reportScreen('R20')).toBe('report.r20');
    expect(reportScreen('R13')).toBe('report.r13');
  });
});

describe('what analytics keeps', () => {
  it('keeps the area’s filters as one value and reads them back', () => {
    const state = {
      ...EMPTY_ANALYTICS_FILTERS,
      period: 'this_year' as const,
      item: { id: 1, name_ur: 'کیلا', name_en: 'Banana' },
      side: 'seller' as const,
      compare: 'previous' as const,
      personText: 'x',
    };
    const kept = rememberedAnalytics(state);
    expect(kept).toEqual({
      period: 'this_year',
      item: { id: 1, name_ur: 'کیلا', name_en: 'Banana' },
      side: 'seller',
      compare: 'previous',
    });
    expect(analyticsFromRemembered(kept)).toMatchObject({
      period: 'this_year',
      side: 'seller',
      compare: 'previous',
      item: { id: 1 },
    });
    expect(rememberedAnalytics(EMPTY_ANALYTICS_FILTERS)).toBeNull();
    expect(analyticsFromRemembered({ period: 'never', side: 3 })).toEqual(EMPTY_ANALYTICS_FILTERS);
  });
});

describe('what the audit log keeps', () => {
  it('keeps the filters it was left with, as one value, and reads them back', () => {
    expect(rememberedAudit(EMPTY_AUDIT_FILTERS)).toBeNull();
    const state = {
      ...EMPTY_AUDIT_FILTERS,
      action: 'day_close',
      entity: 'day',
      from: '01-10-2026',
      text: 'reopen',
    };
    const kept = rememberedAudit(state);
    expect(kept).toEqual({
      action: 'day_close',
      entity: 'day',
      from: '01-10-2026',
      text: 'reopen',
    });
    expect(auditFromRemembered(kept)).toEqual(state);
    expect(auditFromRemembered({ action: 5, from: 'x'.repeat(40) })).toEqual(EMPTY_AUDIT_FILTERS);
    expect(auditFromRemembered('text')).toBeNull();
  });
});

describe('what the list screens keep', () => {
  it('today’s بکری: the seller, the item and the status, not the day', () => {
    expect(todayChoicesText(NO_TODAY_CHOICES)).toBeNull();
    const kept = todayChoicesText({ seller: AKRAM, item: null, status: 'cancelled' });
    expect(kept).toEqual({ seller: AKRAM, status: 'cancelled' });
    expect(todayChoicesFrom(kept)).toEqual({ seller: AKRAM, item: null, status: 'cancelled' });
    expect(todayChoicesFrom({ status: 'sold' })).toEqual(NO_TODAY_CHOICES);
    expect(todayChoicesFrom(null)).toEqual(NO_TODAY_CHOICES);
  });

  it('باقی مال: the days, the seller, the item and the written-off tick', () => {
    expect(stockChoicesText(NO_STOCK_CHOICES)).toBeNull();
    const choices = {
      from: '01-10-2026',
      to: '',
      seller: null,
      item: AKRAM,
      includeWrittenOff: true,
    };
    expect(stockChoicesFrom(stockChoicesText(choices))).toEqual(choices);
    expect(stockChoicesFrom({ from: 5, includeWrittenOff: 'yes' })).toEqual(NO_STOCK_CHOICES);
  });

  it('the پختہ کھاتہ register: days, side, name, phone and paid or due', () => {
    expect(registerChoicesText(NO_REGISTER_CHOICES)).toBeNull();
    const choices = {
      from: '',
      to: '31-10-2026',
      side: 'buyer' as const,
      name: 'شاہد',
      phone: '0300',
      state: 'due' as const,
    };
    expect(registerChoicesFrom(registerChoicesText(choices))).toEqual(choices);
    expect(registerChoicesFrom({ side: 'both', state: 'late', name: 7 })).toEqual(
      NO_REGISTER_CHOICES,
    );
  });

  it('the trips: days, status and search', () => {
    expect(tripChoicesText(NO_TRIP_CHOICES)).toBeNull();
    const choices = { from: '01-10-2026', to: '', status: 'open' as const, search: 'TRP' };
    expect(tripChoicesFrom(tripChoicesText(choices))).toEqual(choices);
    expect(tripChoicesFrom({ status: 'lost', from: 1 })).toEqual(NO_TRIP_CHOICES);
  });

  it('the commission rules: side, charge, party, item and the inactive tick', () => {
    expect(ruleChoicesText(NO_RULE_CHOICES)).toBeNull();
    const choices = {
      side: 'buyer' as const,
      charge: 'laga' as const,
      party: AKRAM,
      item: null,
      showInactive: true,
    };
    expect(ruleChoicesFrom(ruleChoicesText(choices))).toEqual(choices);
    expect(ruleChoicesFrom({ side: 'both', charge: 'tax', party: 'x' })).toEqual(NO_RULE_CHOICES);
  });

  it('the accounts: the kind and the inactive tick, never the typed search', () => {
    expect(accountChoicesText(NO_ACCOUNT_CHOICES)).toBeNull();
    const choices = { type: 'seller' as const, showInactive: true };
    expect(accountChoicesText(choices)).toEqual({ type: 'seller', showInactive: true });
    expect(accountChoicesFrom(accountChoicesText(choices))).toEqual(choices);
    expect(accountChoicesFrom({ type: 'friend', text: 'akram' })).toEqual(NO_ACCOUNT_CHOICES);
  });

  it('the partner statement: the partner and the days', () => {
    expect(statementChoicesText(NO_STATEMENT_CHOICES)).toBeNull();
    const choices = { partnerId: '2', from: '01-10-2026', to: '31-10-2026' };
    expect(statementChoicesFrom(statementChoicesText(choices))).toEqual(choices);
    expect(statementChoicesFrom({ partner: 'two', from: [] })).toEqual(NO_STATEMENT_CHOICES);
  });
});

describe.each(LANGUAGES)(
  'the screens put the filters back and keep the changes (%s)',
  (language: Language) => {
    beforeEach(() => {
      vi.resetAllMocks();
      healthyBackend();
      fakePartners([]);
      vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
      resetClientState(language);
      signedIn(sessionUser({ language }));
      set.mockImplementation(({ screen: name, value }) =>
        Promise.resolve({ screen: name, value: value === '' ? null : value }),
      );
    });

    afterEach(() => {
      leaveAppWindow();
    });

    /** What the screens kept: the guide counts as seen, and each screen has what a test says. */
    function kept(values: Record<string, string>) {
      get.mockImplementation(({ screen: name }) =>
        Promise.resolve({
          screen: name,
          value: name === 'app.guide' ? '{"done":true}' : (values[name] ?? null),
        }),
      );
    }

    const m = (id: string, vars: Record<string, string | number> = {}) =>
      message(language, id, vars);

    it('a report opens as it was left: R15 asks for the payments the Owner had picked', async () => {
      kept({ 'report.r15': '{"voucherType":"payment"}' });
      fakeReports();
      await renderApp('/reports/R15');
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters.voucher_type).toBe(
          'payment',
        );
      });
      expect(get).toHaveBeenCalledWith({ screen: 'report.r15' });
    });

    it('an address with filters in it wins over what was kept', async () => {
      kept({ 'report.r13': '{"detail":true}' });
      fakeReports();
      await renderApp('/reports/R13?party=20');
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters.party).toEqual({
          account_id: 20,
          person_key: null,
        });
      });
      expect(vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters.detail).toBe(false);
    });

    it('a kept value that cannot be read starts the report as new', async () => {
      kept({ 'report.r15': '{"voucherType":"gift","from":99}' });
      fakeReports();
      await renderApp('/reports/R15');
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsRun)).toHaveBeenCalled();
      });
      expect(
        vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters.voucher_type,
      ).toBeUndefined();
    });

    it('a change is kept after a pause, and clearing it forgets', async () => {
      kept({});
      fakeReports();
      await renderApp('/reports/R15');
      const from = await screen.findByLabelText(m('report-filter-from'));
      const user = userEvent.setup();
      await user.type(from, '01-10-2026');
      await waitFor(() => {
        expect(set).toHaveBeenCalledWith({
          screen: 'report.r15',
          value: '{"from":"01-10-2026"}',
        });
      });
      await user.clear(from);
      await waitFor(() => {
        expect(set).toHaveBeenLastCalledWith({ screen: 'report.r15', value: '' });
      });
    });

    it('باقی مال opens with the tick it was left with', async () => {
      kept({ 'list.bikri-stock': '{"includeWrittenOff":true}' });
      fakeBikri();
      fakePlan16();
      await renderApp('/bikri/stock');
      await waitFor(() => {
        expect(bikriApi.bikriStockList).toHaveBeenCalledWith(
          expect.objectContaining({ include_written_off: true }),
        );
      });
      const tick = await screen.findByRole('checkbox', {
        name: (name) => visible(name) === m('bikri-stock-include-written-off'),
      });
      expect(tick).toBeChecked();
    });

    it('باقی مال keeps a change of its filters', async () => {
      kept({});
      fakeBikri();
      fakePlan16();
      await renderApp('/bikri/stock');
      const tick = await screen.findByRole('checkbox', {
        name: (name) => visible(name) === m('bikri-stock-include-written-off'),
      });
      await userEvent.setup().click(tick);
      await waitFor(() => {
        expect(set).toHaveBeenCalledWith({
          screen: 'list.bikri-stock',
          value: '{"includeWrittenOff":true}',
        });
      });
    });
  },
);
