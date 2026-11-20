// The print of a partner's statement for one month (PRD FR-17.12, FR-17.13) in the print language of
// the Owner's choice (plan 81): every heading, the opening line, the totals label and the facts of
// the header are words in that language, never a raw message ID. Umar saw
// `print-partner-month-col-advances` on an Urdu print: the backend sends the heading by its message
// ID and the advances column had no message. The test draws the print from the backend's recorded
// answer (tests/fixtures/partners_statement_month_print_data_*.json), with all ten column headings.
//
// A message ID that no Fluent file has also fails any test that draws it (tests/setup.ts), so this
// file fails by itself on a missing word.

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { leaveAppWindow, renderApp, resetClientState } from '../helpers/app';
import { partnerMonthPrintData } from '../helpers/partner-month-print';
import { message, visible } from '../helpers/render';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn(() => Promise.resolve()) }));

const INPUT = { partner_id: 2, month: '2026-10', language: 'ur', paper: 'a4' } as const;

/** Text that is only a message ID: lower-case words joined by dashes. */
const RAW_ID = /^[a-z]+(-[a-z0-9]+)+$/;

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(emit).mockResolvedValue(undefined);
});

afterEach(() => {
  leaveAppWindow();
  document.documentElement.removeAttribute('style');
});

describe.each(LANGUAGES)('the partner month print in %s', (language) => {
  const open = async () => {
    vi.mocked(invoke).mockImplementation((command) =>
      Promise.resolve(
        command === 'partners_statement_month_print_data'
          ? partnerMonthPrintData(language)
          : undefined,
      ),
    );
    resetClientState('ur');
    const input = { ...INPUT, language };
    await renderApp(
      `/print?print=partner-month&input=${encodeURIComponent(JSON.stringify(input))}&job=job-1`,
    );
    await waitFor(() => {
      expect(vi.mocked(emit).mock.calls.some(([name]) => name === 'print-ready-job-1')).toBe(true);
    });
  };

  it('writes every heading in words, the advances column included, and no raw message ID shows', async () => {
    await open();
    const headings = [...document.querySelectorAll('.print-doc table thead th')].map((cell) =>
      visible(cell.textContent),
    );
    // Ten columns, each with the words of its message.
    const columns = partnerMonthPrintData(language).columns;
    expect(columns).toHaveLength(10);
    expect(headings).toHaveLength(columns.length);
    for (const column of columns) expect(headings).toContain(message(language, column.title_id));
    expect(columns.map((column) => column.title_id)).toContain('print-partner-month-col-advances');
    expect(headings).toContain(message(language, 'partners-month-col-advances'));
    // Nothing on the page is a message ID as it is written in the Fluent files.
    const raw = [...document.querySelectorAll('.print-doc *')]
      .filter((element) => element.children.length === 0)
      .map((element) => visible(element.textContent).trim())
      .filter((text) => RAW_ID.test(text));
    expect(raw).toEqual([]);
  });

  it('the title, the opening line and the totals row are words too', async () => {
    await open();
    const text = visible(document.querySelector('.print-doc')?.textContent);
    expect(text).toContain(message(language, 'print-partner-month-title'));
    expect(text).toContain(message(language, 'print-partner-month-opening'));
    expect(text).toContain(message(language, 'print-partner-month-total'));
  });
});
