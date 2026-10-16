// The printed documents against the REAL print data of the backend (plan 14, T4). The backend wrote
// it in its own snapshots, from its real `bikri_get` answer for the PRD worked day (tests/fixtures/
// print/*.snap, copied, never edited). Each document is drawn here, in Urdu and in English, with the
// numbers of PRD 4.4 and 4.5, through the hidden print route and through the in-app preview. The
// canned data of the other tests must have exactly the shape of the real data.

import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { printDataFor } from '../helpers/print-data';
import { REAL_SNAPSHOTS, realFor, realPrintData, shapeOf } from '../helpers/print-real';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeBikri } from '../helpers/bikri';
import { named } from '../helpers/bikri-screen';
import { fakePartners } from '../helpers/partners';
import { fakePrinting } from '../helpers/print';
import { message, visible } from '../helpers/render';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ emit: vi.fn(() => Promise.resolve()) }));
vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(emit).mockResolvedValue(undefined);
});

afterEach(() => {
  leaveAppWindow();
  document.documentElement.removeAttribute('style');
});

/** What each real document must say: the figures of PRD 4.4 and 4.5, and the words. */
const EXPECTED: Record<string, { figures: string[]; words: string[]; sheets: number }> = {
  'seller_bill-event1': {
    figures: [
      '29,400',
      '3,000',
      '1,764',
      '250',
      '50',
      '5,000',
      '10,084',
      '19,316',
      '12,000',
      '17,400',
    ],
    words: ['انیس ہزار تین سو سولہ روپے', 'Nineteen thousand three hundred sixteen rupees only'],
    sheets: 1,
  },
  'buyer_slips-event1': {
    figures: ['12,880', '720', '160', '18,684', '1,044', '240', '17,400'],
    words: [],
    sheets: 2,
  },
  'pukhta_slip-event2': {
    figures: ['12,800', '12,000', '720', '80', '03000000000'],
    words: ['بارہ ہزار آٹھ سو روپے', 'Twelve thousand eight hundred rupees only'],
    sheets: 1,
  },
  'seller_bill-event2': {
    figures: ['12,000', '720', '1,000', '10,160', '1,840'],
    words: [],
    sheets: 1,
  },
};

describe.each(REAL_SNAPSHOTS)('the real $doc of $event in $language', (snapshot) => {
  const data = realPrintData(snapshot);
  const expected = EXPECTED[`${snapshot.doc}-${snapshot.event}`];

  it('is drawn with the PRD figures, exactly as the backend wrote them', () => {
    const { container } = render(<BikriPrintDocument data={data} />);
    const text = visible(container.textContent);
    for (const figure of expected?.figures ?? []) expect(text, figure).toContain(figure);
    for (const sentence of expected?.words ?? []) {
      if (/[؀-ۿ]/.test(sentence) === (snapshot.language === 'ur')) expect(text).toContain(sentence);
    }
    expect(container.querySelectorAll('.print-sheet')).toHaveLength(expected?.sheets ?? -1);
    // The language, the direction and the paper are the backend's.
    const root = container.querySelector<HTMLElement>('.print-doc');
    expect(root?.lang).toBe(data.language);
    expect(root?.dir).toBe(data.dir);
    expect(root?.dataset['paper']).toBe(data.paper);
    // Every label id the backend sent has words in the print language.
    for (const id of [
      data.title_id,
      ...data.facts.map((fact) => fact.label_id),
      ...data.pages.flatMap((page) => [
        page.party.role_id,
        ...page.amounts.map((amount) => amount.label_id),
        ...(page.payment_id === null ? [] : [page.payment_id]),
        ...(page.words === null ? [] : [page.words.label_id]),
      ]),
    ]) {
      expect(text).toContain(message(snapshot.language, id));
    }
  });

  it('has the shape of the canned data, so the other tests stand for the real answer', () => {
    const canned = printDataFor(snapshot.doc, { language: snapshot.language });
    expect(shapeOf(data)).toEqual(shapeOf(canned));
  });
});

describe('the real documents through the hidden print route', () => {
  beforeEach(() => {
    resetClientState('ur');
  });

  it.each(REAL_SNAPSHOTS)(
    '$doc of $event in $language: one command, then print-ready',
    async (snapshot) => {
      // The real command wrapper is used, so the route is proved to ask the bridge for one command.
      const actual = await vi.importActual<typeof import('../../src/features/reports/api')>(
        '../../src/features/reports/api',
      );
      vi.mocked(reportsApi.reportsBikriPrintData).mockImplementation(actual.reportsBikriPrintData);
      const calls: string[] = [];
      vi.mocked(invoke).mockImplementation((command) => {
        calls.push(command);
        return Promise.resolve(realPrintData(snapshot));
      });
      await renderApp(
        `/print?bikri=41&doc=${snapshot.doc}&language=${snapshot.language}&job=real-1`,
      );
      await waitFor(() => {
        expect(vi.mocked(emit)).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(emit).mock.calls[0]?.[0]).toBe('print-ready-real-1');
      expect(calls).toEqual(['reports_bikri_print_data']);
      const first = realPrintData(snapshot).pages[0];
      expect(visible(document.body.textContent)).toContain(first?.party.name);
    },
  );
});

describe.each(['ur', 'en'] as const)('the real documents in the print preview (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    healthyBackend();
    fakePartners();
    signedIn(sessionUser({ language }));
  });
  afterEach(() => {
    leaveAppWindow();
  });

  it('shows the real bill, slips and پختہ slip of بکری 1 in the box', async () => {
    fakePrinting({ data: (input) => realFor(input.doc, input.language) });
    fakeBikri();
    await renderApp('/bikri/book?id=41');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'bikri-saved-title', { number: 1 })),
    });
    press('F9');
    const box = await screen.findByRole('dialog');
    const user = userEvent.setup();
    await within(box).findByRole('checkbox', {
      name: named(message(language, 'print-doc-seller-bill')),
    });
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'print-box-preview')) }),
    );
    await waitFor(() => {
      expect(box.querySelectorAll('.print-doc')).toHaveLength(3);
    });
    const text = visible(box.textContent);
    for (const figure of ['29,400', '10,084', '19,316', '12,880', '18,684', '12,800']) {
      expect(text, figure).toContain(figure);
    }
    expect(box.querySelectorAll('.print-sheet').length).toBe(4);
  });
});
