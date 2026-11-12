// The hidden print window of a party's statement (plan 74 T3, PRD FR-12.47): the address the backend opens,
// `#/print?statement=<account id>&from=..&to=..&language=<ur|en>&paper=<a4|a5>&job=<job>`. It calls
// `party_ledgers_statement_print_data` and `reports_print_header` and NO other command (the capability of the
// window allows no other), draws the statement in the print language of the address on the paper of the
// address, waits for the fonts and the layout, and tells the backend with `print-ready-<job>`. A page that
// cannot be read says `print-error-<job>`; a period with no entry shows the words for nothing to print and does
// not say ready. The Tauri bridge is stood in for: `invoke` records every command and `emit` every event.

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  isStatementAddress,
  parseStatementPrintParams,
} from '../../src/features/party_ledgers/print/statement-params';
import type { StatementPrintDataInput } from '../../src/bindings/StatementPrintDataInput';
import { useLanguageStore } from '../../src/shared/i18n';
import { leaveAppWindow, renderApp, resetClientState } from '../helpers/app';
import { failWith } from '../helpers/print';
import { message, visible } from '../helpers/render';
import { statementAnswer, statementFrame } from '../helpers/statement-print';
import { refusal } from '../helpers/ui-error';
import type { UiError } from '../../src/bindings/UiError';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn(() => Promise.resolve()) }));

interface Call {
  command: string;
  args: unknown;
}
let calls: Call[] = [];

class Refused {
  constructor(readonly error: UiError) {}
}

function bridge(answer: (command: string, args: unknown) => unknown) {
  calls = [];
  vi.mocked(invoke).mockImplementation((command, args) => {
    calls.push({ command, args });
    const result = answer(command, args);
    return result instanceof Refused ? failWith(result.error) : Promise.resolve(result);
  });
}

/** The backend as it answers the statement: the data of the double and the header of the data. */
function backend(command: string, args: unknown): unknown {
  const input = (args as { input: Record<string, unknown> }).input;
  if (command === 'party_ledgers_statement_print_data') {
    return statementAnswer(input as unknown as StatementPrintDataInput);
  }
  if (command === 'reports_print_header') {
    return statementFrame({
      ...statementAnswer({ account_id: 20, from: '14-10-2026', to: '12-11-2026', language: 'en' }),
      language: input['language'] as 'ur' | 'en',
      title_id: input['title_id'] as string,
      filters: input['facts'] as never,
    });
  }
  return new Refused(refusal([{ code: 'ERR_NOT_IMPLEMENTED' }]));
}

const events = () => vi.mocked(emit).mock.calls.map(([name, payload]) => ({ name, payload }));
const GOOD = 'statement=20&from=01-11-2026&to=06-11-2026&language=en&paper=a4&job=job-1';

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(emit).mockResolvedValue(undefined);
  resetClientState('ur');
});

afterEach(() => {
  leaveAppWindow();
  document.documentElement.removeAttribute('style');
});

describe('the address of the hidden print window for the statement', () => {
  it('reads a good address: the party, the dates as typed, the language, the paper and the job', () => {
    expect(parseStatementPrintParams(`?${GOOD}`)).toEqual({
      job: 'job-1',
      input: { account_id: 20, from: '01-11-2026', to: '06-11-2026', language: 'en', paper: 'a4' },
    });
    expect(isStatementAddress(`?${GOOD}`)).toBe(true);
    expect(isStatementAddress('?voucher=5&doc=receipt_slip&language=ur&job=a')).toBe(false);
  });

  it('A4 is the paper when the address names none, A5 is read, a date shortcut is passed on as typed', () => {
    expect(
      parseStatementPrintParams('?statement=20&from=10&to=10-9&language=ur&job=j')?.input,
    ).toEqual({ account_id: 20, from: '10', to: '10-9', language: 'ur', paper: 'a4' });
    expect(
      parseStatementPrintParams(
        '?statement=20&from=01-11-2026&to=06-11-2026&language=ur&paper=a5&job=j',
      )?.input.paper,
    ).toBe('a5');
  });

  it.each([
    ['no party', 'from=01-11-2026&to=06-11-2026&language=en&job=j'],
    ['a zero party', 'statement=0&from=01-11-2026&to=06-11-2026&language=en&job=j'],
    ['a decimal party', 'statement=2.5&from=01-11-2026&to=06-11-2026&language=en&job=j'],
    ['no first date', 'statement=20&to=06-11-2026&language=en&job=j'],
    ['a date with letters', 'statement=20&from=today&to=06-11-2026&language=en&job=j'],
    ['a date with a script in it', 'statement=20&from=01-11-2026&to=<b>&language=en&job=j'],
    ['an unknown language', 'statement=20&from=01-11-2026&to=06-11-2026&language=fr&job=j'],
    ['an unknown paper', 'statement=20&from=01-11-2026&to=06-11-2026&language=en&paper=a3&job=j'],
    ['a job with a slash', 'statement=20&from=01-11-2026&to=06-11-2026&language=en&job=a/b'],
    ['no job', 'statement=20&from=01-11-2026&to=06-11-2026&language=en'],
  ])('refuses %s', (_name, query) => {
    expect(parseStatementPrintParams(`?${query}`)).toBeNull();
  });
});

describe('the print route for the statement', () => {
  it('reads the statement and the header with the two commands it may call, draws the page in the print language and says print-ready once', async () => {
    bridge(backend);
    await renderApp(`/print?${GOOD}`);
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain('51,360');
    });
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(calls.map((call) => call.command).sort()).toEqual([
      'party_ledgers_statement_print_data',
      'reports_print_header',
    ]);
    expect(
      calls.find((call) => call.command === 'party_ledgers_statement_print_data')?.args,
    ).toEqual({
      input: { account_id: 20, from: '01-11-2026', to: '06-11-2026', language: 'en' },
    });
    const text = visible(document.body.textContent);
    expect(text).toContain('Akram Toll Tax');
    expect(text).toContain(message('en', 'print-statement-total-closing'));
    expect(events()[0]?.name).toBe('print-ready-job-1');
    // The print language is the window's own; the app language did not change; no menus are drawn.
    expect(document.documentElement.lang).toBe('en');
    expect(useLanguageStore.getState().language).toBe('ur');
    expect(document.querySelector('nav')).toBeNull();
    expect(document.querySelector('.print-doc')?.getAttribute('data-paper')).toBe('a4');
  });

  it('lays the page out for the paper of the address', async () => {
    bridge(backend);
    await renderApp(
      '/print?statement=20&from=01-11-2026&to=06-11-2026&language=ur&paper=a5&job=j2',
    );
    await waitFor(() => {
      expect(document.querySelector('.print-doc')?.getAttribute('data-paper')).toBe('a5');
    });
    expect(document.documentElement.lang).toBe('ur');
  });

  it('says print-error with the codes when the statement cannot be read (a date the backend refuses)', async () => {
    bridge(() => new Refused(refusal([{ code: 'ERR_DATE_INVALID', field: 'from' }])));
    await renderApp(`/print?${GOOD}`);
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(events()[0]).toEqual({
      name: 'print-error-job-1',
      payload: { codes: ['ERR_DATE_INVALID'] },
    });
  });

  it('a period with no entry shows the words for nothing to print and draws no statement', async () => {
    bridge(backend);
    await renderApp('/print?statement=20&from=01-01-2000&to=31-12-2000&language=en&job=job-3');
    // The shared view draws its empty state in the app language: nothing is printed from it.
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(message('ur', 'print-preview-empty'));
    });
    expect(document.querySelector('.print-doc')).toBeNull();
    expect(document.querySelector('table.print-statement')).toBeNull();
  });

  it('a period with no entry but a balance carried through it is drawn as a one-line statement and says print-ready', async () => {
    bridge(backend);
    await renderApp('/print?statement=20&from=20-11-2026&to=30-11-2026&language=en&job=job-4');
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(events()[0]?.name).toBe('print-ready-job-4');
    const text = visible(document.body.textContent);
    expect(text).toContain(message('en', 'print-statement-opening'));
    expect(text).toContain('One thousand rupees only');
    expect(document.querySelector('tbody.print-statement-day')).toBeNull();
    expect(document.querySelector('.print-doc')).not.toBeNull();
  });

  it('says print-error and calls nothing for an address that is not a statement address', async () => {
    bridge(backend);
    await renderApp('/print?statement=abc&from=01-11-2026&to=06-11-2026&language=en&job=job-9');
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(events()[0]).toEqual({
      name: 'print-error-job-9',
      payload: { codes: ['ERR_INTERNAL'] },
    });
    expect(calls).toHaveLength(0);
  });
});
