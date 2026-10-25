// The hidden print window for the collection list (plan 17e F2, PRD FR-15.6): the address the
// backend opens, `#/print?collection=<YYYY-MM-DD>&doc=collection_list&language=<ur|en>[&collector=
// <id>][&city=<id>]&job=<job>`. It calls `reports_collection_print_data` and NO other command (the
// capability of the window allows no other), with the same day the address names (`DD-MM-YYYY` in
// the command), draws the backend's REAL list in the print language, waits for the fonts and the
// layout, and tells the backend with `print-ready-<job>` and the page height. A page that cannot be
// read says `print-error-<job>`. The Tauri bridge is stood in for: `invoke` records every command
// and `emit` every event, so the test sees exactly what the backend would.

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  isCollectionAddress,
  parseCollectionPrintParams,
} from '../../src/features/vouchers/print/collection-params';
import { useLanguageStore } from '../../src/shared/i18n';
import type { UiError } from '../../src/bindings/UiError';
import { leaveAppWindow, renderApp, resetClientState } from '../helpers/app';
import { failWith } from '../helpers/print';
import { visible } from '../helpers/render';
import * as real from '../helpers/tag-real';
import { refusal } from '../helpers/ui-error';

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

const events = () => vi.mocked(emit).mock.calls.map(([name, payload]) => ({ name, payload }));
const GOOD = 'collection=2026-10-10&doc=collection_list&language=en&job=job-1';

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(emit).mockResolvedValue(undefined);
  resetClientState('ur');
});

afterEach(() => {
  leaveAppWindow();
  document.documentElement.removeAttribute('style');
});

describe('the address of the hidden print window for the collection list', () => {
  it('reads a good address, the day as the command takes it', () => {
    expect(parseCollectionPrintParams(`?${GOOD}`)).toEqual({
      job: 'job-1',
      input: { date: '10-10-2026', collector_staff_id: null, city_id: null, language: 'en' },
    });
    expect(isCollectionAddress(`?${GOOD}`)).toBe(true);
    expect(isCollectionAddress('?voucher=5&doc=receipt_slip&language=ur&job=a')).toBe(false);
  });

  it('reads the collector and the city when the address has them', () => {
    expect(
      parseCollectionPrintParams(
        '?collection=2026-10-10&doc=collection_list&language=ur&collector=81&city=91&job=a_b-1',
      ),
    ).toEqual({
      job: 'a_b-1',
      input: { date: '10-10-2026', collector_staff_id: 81, city_id: 91, language: 'ur' },
    });
  });

  it.each([
    ['no day', 'doc=collection_list&language=en&job=j'],
    ['a day that is not a date', 'collection=2026-02-30&doc=collection_list&language=en&job=j'],
    ['a day written the other way', 'collection=10-10-2026&doc=collection_list&language=en&job=j'],
    ['another document', 'collection=2026-10-10&doc=receipt_slip&language=en&job=j'],
    ['an unknown language', 'collection=2026-10-10&doc=collection_list&language=fr&job=j'],
    ['a zero collector', 'collection=2026-10-10&doc=collection_list&language=en&collector=0&job=j'],
    ['a decimal city', 'collection=2026-10-10&doc=collection_list&language=en&city=4.5&job=j'],
    ['a job with a slash', 'collection=2026-10-10&doc=collection_list&language=en&job=a/b'],
    ['no job', 'collection=2026-10-10&doc=collection_list&language=en'],
  ])('refuses %s', (_name, query) => {
    expect(parseCollectionPrintParams(`?${query}`)).toBeNull();
  });
});

describe('the print route for the collection list', () => {
  it('reads the list with the one command it may call, draws the real list in the print language and says print-ready once', async () => {
    bridge((command) =>
      command === 'reports_collection_print_data'
        ? real.COLLECTION_EN
        : new Refused(refusal([{ code: 'ERR_NOT_IMPLEMENTED' }])),
    );
    await renderApp(`/print?${GOOD}`);
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain('21,564');
    });
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(calls).toEqual([
      {
        command: 'reports_collection_print_data',
        args: {
          input: { date: '10-10-2026', collector_staff_id: null, city_id: null, language: 'en' },
        },
      },
    ]);
    const text = visible(document.body.textContent);
    expect(text).toContain('Akram Toll Tax K');
    expect(text).toContain('Total of this list');
    expect(events()[0]?.name).toBe('print-ready-job-1');
    // The print language is the window's own; the app language did not change; no menus are drawn.
    expect(document.documentElement.lang).toBe('en');
    expect(useLanguageStore.getState().language).toBe('ur');
    expect(screen.queryByRole('navigation')).toBeNull();
  });

  it('sends the collector and the city of the address to the command', async () => {
    bridge(() => real.COLLECTION_UR);
    await renderApp(
      '/print?collection=2026-10-10&doc=collection_list&language=ur&collector=81&city=91&job=j2',
    );
    await waitFor(() => {
      expect(calls).toHaveLength(1);
    });
    expect(calls[0]?.args).toEqual({
      input: { date: '10-10-2026', collector_staff_id: 81, city_id: 91, language: 'ur' },
    });
    expect(document.documentElement.lang).toBe('ur');
  });

  it('says print-error with the codes when the list cannot be read (an unknown collector)', async () => {
    bridge(() => new Refused(refusal([{ code: 'ERR_NOT_FOUND', field: 'collector_staff_id' }])));
    await renderApp(`/print?${GOOD}&collector=9`);
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(events()[0]).toEqual({
      name: 'print-error-job-1',
      payload: { codes: ['ERR_NOT_FOUND'] },
    });
  });

  it('an empty day shows the words for nothing to print and does not say print-ready', async () => {
    bridge(() => real.COLLECTION_EMPTY);
    await renderApp(`/print?${GOOD}`);
    await waitFor(() => {
      expect(document.querySelector('.print-doc')).toBeNull();
      expect(visible(document.body.textContent)).not.toBe('');
    });
    await waitFor(() => {
      expect(screen.queryByRole('status')).toBeNull();
    });
    expect(events().some((event) => event.name.startsWith('print-ready'))).toBe(false);
  });

  it('says print-error and calls nothing for an address that is not a collection address', async () => {
    bridge(() => real.COLLECTION_UR);
    await renderApp('/print?collection=abc&doc=collection_list&language=en&job=job-9');
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
