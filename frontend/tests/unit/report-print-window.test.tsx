// The hidden print window for a report, its slips and an analytics view (round 14, 17e, PRD FR-19.3,
// FR-18.4): `#/print?report=R5&language=ur&paper=a4[&slips=1]&filters=<JSON>&job=<job>` and
// `#/print?analytics=K&...`. The address is read as plain text and checked once; the page calls the
// print-data command and the print header and NO other command (its capability allows no other),
// draws the document in the print language on the shared header and footer, and tells the backend
// with `print-ready-<job>`. An address that is not right is `print-error-<job>`, never a guess. The
// Tauri bridge is stood in for here, as in print-route.test.tsx.

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportFilters } from '../../src/bindings/ReportFilters';
import {
  ANALYTICS_VIEW_IDS,
  parseAnalyticsPrintParams,
} from '../../src/features/analytics/print/params';
import { REPORT_IDS, parseReportPrintParams } from '../../src/features/reports/print/params';
import { leaveAppWindow, renderApp, resetClientState } from '../helpers/app';
import { CATALOG } from '../helpers/reports';
import { ANALYTICS_CATALOG } from '../helpers/analytics-real';
import {
  ANALYTICS_PRINT_DATA_K_EN,
  ANALYTICS_RUN_K_FRUIT_REQUEST,
} from '../helpers/analytics-fruit-real';
import { printFrameFor } from '../helpers/print-frame-real';
import { BUYER_BILLS_EN, PRINT_R5_EN, R37_FRUIT_REQUEST } from '../helpers/reports';
import { visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn(() => Promise.resolve()) }));

interface Call {
  command: string;
  args: unknown;
}
let calls: Call[] = [];

/** Every command goes to the stand-in; the call is kept. */
function bridge() {
  calls = [];
  vi.mocked(invoke).mockImplementation((command, args) => {
    calls.push({ command, args });
    const input = (args as { input?: { language?: 'ur' | 'en'; title_id?: string; facts?: [] } })
      .input;
    switch (command) {
      case 'reports_print_data':
        return Promise.resolve(PRINT_R5_EN);
      case 'reports_buyer_bills_print_data':
        return Promise.resolve(BUYER_BILLS_EN);
      case 'analytics_print_data':
        return Promise.resolve(ANALYTICS_PRINT_DATA_K_EN);
      case 'reports_print_header':
        return Promise.resolve(
          printFrameFor({
            language: input?.language ?? 'en',
            title_id: input?.title_id ?? '',
            facts: input?.facts ?? [],
          }),
        );
      default:
        return Promise.reject(new Error(`the print window may not call ${command}`));
    }
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
const filtersText = (filters: ReportFilters) => encodeURIComponent(JSON.stringify(filters));
const FILTERS = filtersText(R37_FRUIT_REQUEST.filters);

describe('the address of the print window for a report', () => {
  it('reads the report, the language, the paper, the slips and the filters', () => {
    const params = parseReportPrintParams(
      `?report=R5&language=ur&paper=a5&slips=1&filters=${FILTERS}&job=job-1`,
    );
    expect(params).toEqual({
      job: 'job-1',
      slips: true,
      // The print window asks for the report in the order of the page (plan 38: no sort, no month rows).
      input: {
        id: 'R5',
        language: 'ur',
        paper: 'a5',
        filters: R37_FRUIT_REQUEST.filters,
        sort: null,
        group_by_month: false,
      },
    });
  });

  it('takes no paper for A4 and no slips for a plain report', () => {
    const params = parseReportPrintParams(`?report=R20&language=en&filters=${FILTERS}&job=j2`);
    expect(params?.input.paper).toBeNull();
    expect(params?.slips).toBe(false);
  });

  it.each([
    ['a job that is not a name', `?report=R5&language=ur&filters=${FILTERS}&job=a%20b`],
    ['no job', `?report=R5&language=ur&filters=${FILTERS}`],
    ['an unknown report', `?report=R99&language=ur&filters=${FILTERS}&job=j`],
    ['an unknown language', `?report=R5&language=fr&filters=${FILTERS}&job=j`],
    ['an unknown paper', `?report=R5&language=ur&paper=a3&filters=${FILTERS}&job=j`],
    ['no filters', '?report=R5&language=ur&job=j'],
    ['filters that are not JSON', '?report=R5&language=ur&filters=%7Bnope&job=j'],
    ['filters that are not an object', '?report=R5&language=ur&filters=%5B1%5D&job=j'],
    ['slips that are not 1', `?report=R5&language=ur&slips=yes&filters=${FILTERS}&job=j`],
  ])('is not valid with %s', (_name, search) => {
    expect(parseReportPrintParams(search)).toBeNull();
  });

  it('knows every report of the catalogue, and no other', () => {
    expect([...REPORT_IDS].sort()).toEqual(CATALOG.map((entry) => entry.id).sort());
  });
});

describe('the address of the print window for an analytics view', () => {
  const VIEW_FILTERS = encodeURIComponent(JSON.stringify(ANALYTICS_RUN_K_FRUIT_REQUEST.filters));

  it('reads the view, the language, the paper and the filters', () => {
    expect(
      parseAnalyticsPrintParams(`?analytics=K&language=en&paper=a4&filters=${VIEW_FILTERS}&job=j3`),
    ).toEqual({
      job: 'j3',
      input: {
        view: 'K',
        language: 'en',
        paper: 'a4',
        filters: ANALYTICS_RUN_K_FRUIT_REQUEST.filters,
        sort: null,
      },
    });
  });

  it.each([
    ['an unknown view', `?analytics=Z&language=en&filters=${VIEW_FILTERS}&job=j`],
    ['an unknown language', `?analytics=K&language=fr&filters=${VIEW_FILTERS}&job=j`],
    ['no filters', '?analytics=K&language=en&job=j'],
    ['no job', `?analytics=K&language=en&filters=${VIEW_FILTERS}`],
  ])('is not valid with %s', (_name, search) => {
    expect(parseAnalyticsPrintParams(search)).toBeNull();
  });

  it('knows every view of the catalogue, and no other', () => {
    expect([...ANALYTICS_VIEW_IDS].sort()).toEqual(
      ANALYTICS_CATALOG.map((entry) => entry.id).sort(),
    );
  });
});

describe('the print window', () => {
  it('draws a report on the shared header, calls the two commands it may and says print-ready once', async () => {
    bridge();
    await renderApp(`/print?report=R5&language=en&paper=a4&filters=${FILTERS}&job=job-1`);
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-ready-job-1')).toBe(true);
    });
    expect(calls.map((call) => call.command)).toEqual([
      'reports_print_data',
      'reports_print_header',
    ]);
    // The header asked for is the one of this print: its title and its filters, in its language.
    expect(calls[1]?.args).toMatchObject({
      input: { language: 'en', title_id: PRINT_R5_EN.title_id, facts: PRINT_R5_EN.filters },
    });
    expect(visible(document.querySelector('.print-header')?.textContent)).toContain(
      'Maher Adnan and Brothers',
    );
    expect(document.querySelector('.print-doc')?.getAttribute('data-window')).toBe('true');
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(events()).toHaveLength(1);
  });

  it('draws the slips of all the credit buyers with slips=1, one sheet for each', async () => {
    bridge();
    await renderApp(`/print?report=R5&language=en&paper=a4&slips=1&filters=${FILTERS}&job=job-2`);
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-ready-job-2')).toBe(true);
    });
    expect(calls.map((call) => call.command)).toEqual([
      'reports_buyer_bills_print_data',
      'reports_print_header',
    ]);
    expect(document.querySelectorAll('.print-sheet')).toHaveLength(BUYER_BILLS_EN.slips.length);
  });

  it('draws an analytics view and calls only its own print data and the header', async () => {
    bridge();
    const viewFilters = encodeURIComponent(JSON.stringify(ANALYTICS_RUN_K_FRUIT_REQUEST.filters));
    await renderApp(`/print?analytics=K&language=en&paper=a4&filters=${viewFilters}&job=job-3`);
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-ready-job-3')).toBe(true);
    });
    expect(calls.map((call) => call.command)).toEqual([
      'analytics_print_data',
      'reports_print_header',
    ]);
  });

  it('says print-error once for an address that is not right, and calls nothing', async () => {
    bridge();
    await renderApp('/print?report=R99&language=en&filters=%7B%7D&job=job-4');
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(events()[0]?.name).toBe('print-error-job-4');
    expect(calls).toEqual([]);
  });

  it('says print-error with the codes when the print data cannot be read', async () => {
    calls = [];
    vi.mocked(invoke).mockImplementation(() =>
      // The backend's refusal is a plain object on the wire; an Error that carries the same fields reads the same.
      Promise.reject(
        Object.assign(
          new Error('refused'),
          refusal([{ code: 'ERR_DATE_INVALID', field: 'filters.from' }]),
        ),
      ),
    );
    await renderApp(`/print?report=R5&language=en&filters=${FILTERS}&job=job-5`);
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-error-job-5')).toBe(true);
    });
    expect(events()[0]?.payload).toEqual({ codes: ['ERR_DATE_INVALID'] });
  });
});
