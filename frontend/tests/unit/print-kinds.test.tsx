// The hidden print window of the prints of plan 78 (`#/print?print=<kind>&input=<JSON>&job=<job>`): the address
// is read and checked, the one print-data command of the kind is called with exactly the asked input, the page is
// drawn in the print language, and the window tells the backend `print-ready-<job>`. An address that is not right
// is a failed print, told once as `print-error-<job>`. The Tauri bridge is stood in for here: `invoke` records
// every command and `emit` every event, so the test sees what the backend would.

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { PRINT_KINDS, isKindAddress, parseKindPrintParams } from '../../src/app/print-kinds/params';
import { leaveAppWindow, renderApp, resetClientState } from '../helpers/app';
import { printData as collectionPrintData } from '../helpers/collection-sheet';
import { adhariPrintData, daySlipsData, salePrintData } from '../helpers/day-slips';
import { statementRowsPrintAnswer } from '../helpers/party-statement-rows';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn(() => Promise.resolve()) }));

interface Call {
  command: string;
  args: unknown;
}
let calls: Call[] = [];

/** Every command goes to `answer`; the call is kept. */
function bridge(answer: (command: string, args: unknown) => unknown) {
  calls = [];
  vi.mocked(invoke).mockImplementation((command, args) => {
    calls.push({ command, args });
    return Promise.resolve(answer(command, args));
  });
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(emit).mockResolvedValue(undefined);
  resetClientState('ur');
});

afterEach(() => {
  leaveAppWindow();
  document.documentElement.removeAttribute('style');
});

const events = () => vi.mocked(emit).mock.calls.map(([name, payload]) => ({ name, payload }));
const url = (kind: string, input: unknown, job = 'job-1') =>
  `/print?print=${kind}&input=${encodeURIComponent(JSON.stringify(input))}&job=${job}`;

const SLIPS_INPUT = {
  date: '2026-10-10',
  bikri_id: null,
  bikri_ids: null,
  buyer_ids: null,
  sections: 'both',
  language: 'en',
  paper: 'a4',
} as const;
const STATEMENT_INPUT = {
  account_id: 20,
  from: '14-10-2026',
  to: '12-11-2026',
  items: false,
  include_history: false,
  language: 'en',
  paper: 'a4',
} as const;

/** One case for each kind with a fixture: its address, the one command it may call and what that answers. */
const CASES = [
  {
    kind: 'parchi',
    command: 'reports_buyer_parchi_print_data',
    input: { ...SLIPS_INPUT, sections: 'buyers' },
    answer: () => daySlipsData('en', 'parchi'),
    sees: '.print-parchi-slip',
  },
  {
    kind: 'seller-bills',
    command: 'reports_seller_bills_print_data',
    input: { ...SLIPS_INPUT, sections: 'sellers' },
    answer: () => daySlipsData('en', 'seller-bills'),
    sees: '[data-variant="seller-bills"]',
  },
  {
    kind: 'sale',
    command: 'reports_sale_print_data',
    input: SLIPS_INPUT,
    answer: () => salePrintData('en'),
    sees: '[data-variant="sale-print"]',
  },
  {
    kind: 'adhari',
    command: 'reports_adhari_sheet_print_data',
    input: { date: '2026-10-10', language: 'en', paper: 'a4' },
    answer: () => adhariPrintData('en'),
    sees: '.print-doc table',
  },
  {
    kind: 'collection-sheet',
    command: 'reports_collection_sheet_print_data',
    input: { date: '2026-10-10', language: 'en', paper: 'a4', search: null },
    answer: () => collectionPrintData('en'),
    sees: '.print-doc table',
  },
  {
    kind: 'statement-rows',
    command: 'party_ledgers_statement_rows_print_data',
    input: STATEMENT_INPUT,
    answer: () => statementRowsPrintAnswer(STATEMENT_INPUT),
    sees: '.print-doc table',
  },
  // Plan 79: the pages behind the dashboard tiles, each with its own print-data command.
  {
    kind: 'seller-dues',
    command: 'dashboard_seller_dues_print_data',
    input: {
      side: 'shop_owes',
      party_id: null,
      search: null,
      sort: null,
      language: 'en',
      paper: 'a4',
    },
    answer: () => collectionPrintData('en'),
    sees: '.print-doc table',
  },
  {
    kind: 'total-dues',
    command: 'dashboard_total_dues_print_data',
    input: { language: 'en', paper: 'a4' },
    answer: () => collectionPrintData('en'),
    sees: '.print-doc table',
  },
  {
    kind: 'lent-open',
    command: 'dashboard_lent_open_print_data',
    input: { search: null, sort: null, language: 'en', paper: 'a4' },
    answer: () => collectionPrintData('en'),
    sees: '.print-doc table',
  },
  {
    kind: 'advances-open',
    command: 'advances_report_print_data',
    input: {
      seller_id: null,
      as_of: '',
      search: null,
      only_open: true,
      language: 'en',
      paper: 'a4',
    },
    answer: () => collectionPrintData('en'),
    sees: '.print-doc table',
  },
  {
    kind: 'cash-sales-month',
    command: 'dashboard_cash_sales_month_print_data',
    input: { month: '2026-10', language: 'en', paper: 'a4' },
    answer: () => collectionPrintData('en'),
    sees: '.print-doc table',
  },
  {
    kind: 'cash-sales-day',
    command: 'dashboard_cash_sales_day_print_data',
    input: { date: '2026-10-10', language: 'en', paper: 'a4' },
    answer: () => collectionPrintData('en'),
    sees: '.print-doc table',
  },
  // Plan 83: the sales page behind the tile آج کی بکری.
  {
    kind: 'sales',
    command: 'dashboard_sales_print_data',
    input: {
      period: { kind: 'month', month: '2026-10' },
      goods: 'trip',
      language: 'en',
      paper: 'a4',
    },
    answer: () => collectionPrintData('en'),
    sees: '.print-doc table',
  },
] as const;

describe('the address of a print', () => {
  it('knows its kinds, and a value that is not a kind is still a print address (so it can fail loudly)', () => {
    expect(PRINT_KINDS).toEqual([
      'parchi',
      'seller-bills',
      'sale',
      'adhari',
      'collection-sheet',
      'statement-rows',
      'buyer-dues',
      'commission',
      'partner-month',
      'seller-dues',
      'total-dues',
      'lent-open',
      'advances-open',
      'cash-sales-month',
      'cash-sales-day',
      'sales',
      'total-worth',
    ]);
    expect(isKindAddress('print=parchi&job=a')).toBe(true);
    expect(isKindAddress('print=nothing&job=a')).toBe(true);
    expect(isKindAddress('report=R5&job=a')).toBe(false);
  });

  it.each(CASES)('reads the input of $kind as it was sent', ({ kind, input }) => {
    const query = `print=${kind}&input=${encodeURIComponent(JSON.stringify(input))}&job=job-9`;
    expect(parseKindPrintParams(query)).toEqual({ job: 'job-9', kind, input });
  });

  it.each([
    ['no job', `print=parchi&input=${encodeURIComponent('{}')}`],
    ['an unknown kind', `print=other&input=${encodeURIComponent('{}')}&job=a`],
    ['no input', 'print=parchi&job=a'],
    ['an input that is not JSON', 'print=parchi&input=%7Bnot&job=a'],
    ['an input that is a list', `print=parchi&input=${encodeURIComponent('[1]')}&job=a`],
    ['an input that is a number', `print=sale&input=7&job=a`],
  ])('refuses an address with %s', (_why, query) => {
    expect(parseKindPrintParams(query)).toBeNull();
  });
});

describe('the print window of a kind', () => {
  it.each(CASES)(
    '$kind calls $command and nothing else, with exactly the asked input, and says it is ready',
    async ({ kind, command, input, answer, sees }) => {
      bridge((name) => (name === command ? answer() : undefined));
      await renderApp(url(kind, input));
      await waitFor(() => {
        expect(events().some((event) => event.name === 'print-ready-job-1')).toBe(true);
      });
      expect(calls).toEqual([{ command, args: { input } }]);
      expect(document.querySelector(sees)).not.toBeNull();
      expect(events().some((event) => event.name === 'print-error-job-1')).toBe(false);
    },
  );

  it('the page is in the print language and direction of the data, not of the app', async () => {
    bridge((name) => (name === 'reports_sale_print_data' ? salePrintData('en') : undefined));
    await renderApp(url('sale', SLIPS_INPUT));
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-ready-job-1')).toBe(true);
    });
    // The app is in Urdu (right-to-left); the print is English, so its own root says so.
    const holder = document.querySelector('.print-report')?.closest('[lang]');
    expect(holder?.getAttribute('lang')).toBe('en');
    expect(holder?.getAttribute('dir')).toBe('ltr');
  });

  it('an address that is not right is a failed print, told once, and nothing is read', async () => {
    bridge(() => undefined);
    await renderApp(`/print?print=parchi&input=%7Bnot&job=job-2`);
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-error-job-2')).toBe(true);
    });
    expect(events().filter((event) => event.name === 'print-error-job-2')).toHaveLength(1);
    expect(events().find((event) => event.name === 'print-error-job-2')?.payload).toEqual({
      codes: ['ERR_INTERNAL'],
    });
    expect(calls).toEqual([]);
    expect(events().some((event) => event.name === 'print-ready-job-2')).toBe(false);
    // The page says so in words, with no code on it.
    expect(screen.queryByText(/ERR_/)).toBeNull();
  });
});
