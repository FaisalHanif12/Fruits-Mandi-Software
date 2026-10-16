// The hidden print window's route (plan 14 contract 7): `#/print?bikri=...&doc=...&language=...
// &job=...`. It calls `reports_bikri_print_data` and NO other command (its capability allows no
// other), draws the document in the print language, waits for the fonts and the layout, and tells
// the backend with `print-ready-<job>` and the page height. A page that cannot be read says
// `print-error-<job>`. The Tauri bridge is stood in for here: `invoke` records every command and
// `emit` records every event, so the test sees exactly what the backend would.

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { parsePrintParams, jobOf } from '../../src/features/bikri/print/params';
import { useLanguageStore } from '../../src/shared/i18n';
import { leaveAppWindow, renderApp, resetClientState } from '../helpers/app';
import type { UiError } from '../../src/bindings/UiError';
import { failWith } from '../helpers/print';
import { printDataFor } from '../helpers/print-data';
import { refusal } from '../helpers/ui-error';
import { message, visible } from '../helpers/render';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn(() => Promise.resolve()) }));

interface Call {
  command: string;
  args: unknown;
}
let calls: Call[] = [];

/** What `answer` returns for a command that fails: Tauri rejects with the raw error value. */
class Refused {
  constructor(readonly error: UiError) {}
}

/** Every command goes to `answer`; the call is kept. */
function bridge(answer: (command: string, args: unknown) => unknown) {
  calls = [];
  vi.mocked(invoke).mockImplementation((command, args) => {
    calls.push({ command, args });
    const result = answer(command, args);
    return result instanceof Refused ? failWith(result.error) : Promise.resolve(result);
  });
}

const dataBridge = () => {
  bridge((command, args) => {
    if (command !== 'reports_bikri_print_data') {
      return new Refused(refusal([{ code: 'ERR_NOT_IMPLEMENTED' }]));
    }
    const { input } = args as { input: { doc: 'seller_bill'; language: 'ur' | 'en' } };
    return printDataFor(input.doc, { language: input.language });
  });
};

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
const url = (query: string) => `/print?${query}`;
const GOOD = 'bikri=41&doc=seller_bill&language=en&job=job-1';

describe('the print route', () => {
  it('reads the print data with the one command it may call, with exactly the asked input', async () => {
    dataBridge();
    await renderApp(url(GOOD));
    await waitFor(() => {
      expect(events().some((event) => event.name === 'print-ready-job-1')).toBe(true);
    });
    expect(calls).toEqual([
      {
        command: 'reports_bikri_print_data',
        args: { input: { bikri_id: 41, doc: 'seller_bill', language: 'en', only_buyer_id: null } },
      },
    ]);
  });

  it('passes the one buyer of a slip on', async () => {
    dataBridge();
    await renderApp(url('bikri=41&doc=buyer_slips&language=ur&job=j2&buyer=19'));
    await waitFor(() => {
      expect(calls).toHaveLength(1);
    });
    expect(calls[0]?.args).toEqual({
      input: { bikri_id: 41, doc: 'buyer_slips', language: 'ur', only_buyer_id: 19 },
    });
  });

  it('draws the document in the print language, and says print-ready once with the page height', async () => {
    dataBridge();
    await renderApp(url(GOOD));
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        'Nineteen thousand three hundred sixteen rupees only',
      );
    });
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    const [ready] = events();
    expect(ready?.name).toBe('print-ready-job-1');
    expect(Object.keys(ready?.payload as object)).toEqual(['height_px']);
    expect(typeof (ready?.payload as { height_px: unknown }).height_px).toBe('number');
    // No print-error, and still only the one ready event a moment later.
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(events()).toHaveLength(1);
  });

  it('reports the height of the document itself, rounded up, for the 80 mm roll', async () => {
    dataBridge();
    // jsdom has no layout, so the document says it is 1234.2 pixels tall.
    const measure = vi
      .spyOn(Element.prototype, 'getBoundingClientRect')
      .mockImplementation(function (this: Element) {
        return { height: this.classList.contains('print-doc') ? 1234.2 : 0 } as DOMRect;
      });
    try {
      await renderApp(url(GOOD));
      await waitFor(() => {
        expect(events()).toHaveLength(1);
      });
    } finally {
      measure.mockRestore();
    }
    expect(events()[0]).toEqual({ name: 'print-ready-job-1', payload: { height_px: 1235 } });
  });

  it('keeps the print language on the window, and leaves the app language alone', async () => {
    dataBridge();
    // The app is in Urdu; this print is in English.
    await renderApp(url(GOOD));
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(document.documentElement.lang).toBe('en');
    expect(document.documentElement.dir).toBe('ltr');
    expect(useLanguageStore.getState().language).toBe('ur');
    expect(document.documentElement.style.getPropertyValue('--print-footer')).toContain(
      message('en', 'print-page-label'),
    );
  });

  it('is outside the app shell: no menu, no sign-in screen', async () => {
    dataBridge();
    await renderApp(url(GOOD));
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(screen.queryByRole('navigation')).toBeNull();
    expect(screen.queryByRole('button')).toBeNull();
  });

  it('says print-error with the codes when the data cannot be read, and never print-ready', async () => {
    bridge(() => new Refused(refusal([{ code: 'ERR_NOT_FOUND' }])));
    await renderApp(url(GOOD));
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(events()[0]).toEqual({
      name: 'print-error-job-1',
      payload: { codes: ['ERR_NOT_FOUND'] },
    });
    // The window itself shows plain words and a retry, never a raw code.
    expect(visible(document.body.textContent)).not.toContain('ERR_NOT_FOUND');
  });

  it('says print-error and calls nothing for an address that is not a print address', async () => {
    dataBridge();
    await renderApp(url('bikri=abc&doc=seller_bill&language=en&job=job-9'));
    await waitFor(() => {
      expect(events()).toHaveLength(1);
    });
    expect(events()[0]).toEqual({
      name: 'print-error-job-9',
      payload: { codes: ['ERR_INTERNAL'] },
    });
    expect(calls).toHaveLength(0);
  });

  it('with no job it cannot tell anybody, and calls nothing', async () => {
    dataBridge();
    await renderApp(url('bikri=41&doc=seller_bill&language=en'));
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(events()).toHaveLength(0);
    expect(calls).toHaveLength(0);
  });
});

describe('the address of the print window', () => {
  it('reads a good address', () => {
    expect(parsePrintParams('?bikri=41&doc=pukhta_slip&language=ur&job=a_b-1')).toEqual({
      job: 'a_b-1',
      input: { bikri_id: 41, doc: 'pukhta_slip', language: 'ur', only_buyer_id: null },
    });
    expect(parsePrintParams(`?${GOOD}&buyer=7`)?.input.only_buyer_id).toBe(7);
  });

  it.each([
    ['no بکری', 'doc=seller_bill&language=en&job=j'],
    ['a zero بکری', 'bikri=0&doc=seller_bill&language=en&job=j'],
    ['a decimal بکری', 'bikri=4.5&doc=seller_bill&language=en&job=j'],
    ['an unknown document', 'bikri=1&doc=report&language=en&job=j'],
    ['an unknown language', 'bikri=1&doc=seller_bill&language=fr&job=j'],
    ['a buyer that is not a number', 'bikri=1&doc=buyer_slips&language=en&job=j&buyer=x'],
    ['a job with a slash', 'bikri=1&doc=seller_bill&language=en&job=a/b'],
    ['an empty job', 'bikri=1&doc=seller_bill&language=en&job='],
  ])('refuses %s', (_name, query) => {
    expect(parsePrintParams(`?${query}`)).toBeNull();
  });

  it('keeps the job text exactly, even when it looks like a number', () => {
    expect(jobOf('?job=1e3&bikri=1')).toBe('1e3');
    expect(jobOf('?job=007')).toBe('007');
  });
});
