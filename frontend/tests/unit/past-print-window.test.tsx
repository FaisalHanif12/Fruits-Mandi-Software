// The hidden print window for پچھلی بکری (plan 72 P8, PRD FR-19.3): `#/print?bikrilist=1&language=<ur|en>&paper=
// <a4|a5>&list=<JSON>&job=<job>`. The address is read as plain text and checked once; the page calls the
// print-data command and the print header and NO other command (its capability allows no other), draws
// the document in the print language on the shared header and footer, and tells the backend with
// `print-ready-<job>`. An address that is not right is `print-error-<job>`, never a guess. The Tauri
// bridge is stood in for here, as in report-print-window.test.tsx; the print data is the double.

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  isPastListAddress,
  parsePastPrintParams,
} from '../../src/features/bikri/print/past-print-params';
import type { BikriListPrintInput } from '../../src/bindings/BikriListPrintInput';
import type { PastPrintFilter } from '../../src/features/bikri/past-print-filter';
import { leaveAppWindow, renderApp, resetClientState } from '../helpers/app';
import { pastPrintDataFor } from '../helpers/past-print';
import { printFrameFor } from '../helpers/print-frame-real';
import { visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn(() => Promise.resolve()) }));

const FILTER: PastPrintFilter = {
  from: '03-10-2026',
  to: '09-10-2026',
  with_today: true,
  seller_id: null,
  item_id: null,
  status: null,
  show_finished: false,
  search: null,
  all: true,
};
const LIST = encodeURIComponent(JSON.stringify(FILTER));

interface Call {
  command: string;
  args: unknown;
}
let calls: Call[] = [];

/** Every command goes to the stand-in; the call is kept. Only the two the window may call answer. */
function bridge() {
  calls = [];
  vi.mocked(invoke).mockImplementation((command, args) => {
    calls.push({ command, args });
    const input = (args as { input?: Record<string, unknown> }).input ?? {};
    switch (command) {
      case 'bikri_list_print_data':
        return Promise.resolve(pastPrintDataFor(input as unknown as BikriListPrintInput));
      case 'reports_print_header':
        return Promise.resolve(
          printFrameFor({
            language: (input['language'] as 'ur' | 'en' | undefined) ?? 'en',
            title_id: (input['title_id'] as string | undefined) ?? '',
            facts: (input['facts'] as [] | undefined) ?? [],
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

describe('the address of the print window for the list', () => {
  it('knows its own address and no other', () => {
    expect(isPastListAddress(`?bikrilist=1&language=ur&list=${LIST}&job=j`)).toBe(true);
    expect(isPastListAddress('?report=R5&language=ur&job=j')).toBe(false);
  });

  it('reads the language, the paper and the filter, with the switch `all` in it', () => {
    expect(
      parsePastPrintParams(`?bikrilist=1&language=ur&paper=a5&list=${LIST}&job=job-1`),
    ).toEqual({
      job: 'job-1',
      input: { ...FILTER, language: 'ur', paper: 'a5' },
    });
  });

  it('takes A4 when no paper is named', () => {
    expect(parsePastPrintParams(`?bikrilist=1&language=en&list=${LIST}&job=j2`)?.input.paper).toBe(
      'a4',
    );
  });

  it.each([
    ['a job that is not a name', `?bikrilist=1&language=ur&list=${LIST}&job=a%20b`],
    ['no job', `?bikrilist=1&language=ur&list=${LIST}`],
    ['an unknown language', `?bikrilist=1&language=fr&list=${LIST}&job=j`],
    ['an unknown paper', `?bikrilist=1&language=ur&paper=a3&list=${LIST}&job=j`],
    ['no filter', '?bikrilist=1&language=ur&job=j'],
    ['a filter that is not JSON', '?bikrilist=1&language=ur&list=%7Bnope&job=j'],
    ['a filter that is not an object', '?bikrilist=1&language=ur&list=%5B1%5D&job=j'],
  ])('is not valid with %s', (_name, search) => {
    expect(parsePastPrintParams(search)).toBeNull();
  });
});

describe('the print window', () => {
  it('draws the list on the shared header, calls the two commands it may and says print-ready once', async () => {
    bridge();
    await renderApp(`/print?bikrilist=1&language=en&paper=a4&list=${LIST}&job=job-1`);
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-ready-job-1')).toBe(true);
    });
    expect(calls.map((call) => call.command)).toEqual([
      'bikri_list_print_data',
      'reports_print_header',
    ]);
    // The data is asked with the filter of the address, in the language and on the paper of the print.
    expect(calls[0]?.args).toEqual({ input: { ...FILTER, language: 'en', paper: 'a4' } });
    // The header asked for is the one of this print: its title and its filters, in its language.
    expect(calls[1]?.args).toMatchObject({
      input: { language: 'en', title_id: 'bikri-list-print-title' },
    });
    expect(visible(document.querySelector('.print-header')?.textContent)).toContain(
      'Maher Adnan and Brothers',
    );
    expect(document.querySelector('.print-doc')?.getAttribute('data-window')).toBe('true');
    // Eight heading rows: today and the seven days of the window.
    expect(document.querySelectorAll('.print-table tr[data-row="group"]')).toHaveLength(8);
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(events()).toHaveLength(1);
  });

  it('is drawn in Urdu when the address says so', async () => {
    bridge();
    await renderApp(`/print?bikrilist=1&language=ur&paper=a5&list=${LIST}&job=job-2`);
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-ready-job-2')).toBe(true);
    });
    expect(calls[0]?.args).toEqual({ input: { ...FILTER, language: 'ur', paper: 'a5' } });
    expect(document.querySelector('.print-doc')?.getAttribute('dir')).toBe('rtl');
  });

  it('says print-error once for an address that is not right, and calls nothing', async () => {
    bridge();
    await renderApp('/print?bikrilist=1&language=fr&list=%7B%7D&job=job-4');
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(events()[0]?.name).toBe('print-error-job-4');
    expect(calls).toEqual([]);
  });

  it('says print-error with the codes when the print data cannot be read', async () => {
    calls = [];
    vi.mocked(invoke).mockImplementation(() =>
      Promise.reject(
        Object.assign(
          new Error('refused'),
          refusal([{ code: 'ERR_DATE_INVALID', field: 'filters.from' }]),
        ),
      ),
    );
    await renderApp(`/print?bikrilist=1&language=en&list=${LIST}&job=job-5`);
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-error-job-5')).toBe(true);
    });
    expect(events().find((event) => event.name === 'print-error-job-5')?.payload).toEqual({
      codes: ['ERR_DATE_INVALID'],
    });
  });
});
