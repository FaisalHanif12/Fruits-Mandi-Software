// Print preview speed (NFR-SPD-14, plan 14 acceptance 5): a 15-page document is drawn and ready
// to print in under 2 seconds. jsdom does no layout, so this times what the app itself does for
// the pages (building 15 sheets of 12 lines and waiting for the fonts and the layout). The real
// browser layout is timed by tests/e2e/print-shots.mjs. The runner's threshold is the NFR's own
// 2 seconds: the work here is far below it, so a slow runner still passes and a real slowdown fails.

import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { LANGUAGES } from '../../src/i18n';
import { fontFamiliesOf, waitForPrintReady } from '../../src/shared/print';
import { longPrintData } from '../helpers/print-data';

const NFR_SPD_14_MS = 2_000;
const PAGES = 15;
const LINES_PER_PAGE = 12;

describe.each(LANGUAGES)('a %s print of 15 pages', (language) => {
  it('is drawn and ready in under 2 seconds', async () => {
    const data = longPrintData(PAGES, LINES_PER_PAGE, { language });
    const started = performance.now();
    const { container } = render(<BikriPrintDocument data={data} applyRoot />);
    await waitForPrintReady(
      fontFamiliesOf(data),
      data.pages.map((page) => page.party.name).join(' '),
    );
    const took = performance.now() - started;
    process.stdout.write(
      `print-speed: ${language}, ${String(PAGES)} pages: ${took.toFixed(0)} ms, NFR target ${String(NFR_SPD_14_MS)} ms\n`,
    );
    expect(container.querySelectorAll('.print-sheet')).toHaveLength(PAGES);
    // The buyer's bill is the client's R5 layout: one line of تفصیل for each line of goods.
    expect(container.querySelectorAll('.print-r5-detail')).toHaveLength(PAGES * LINES_PER_PAGE);
    expect(took).toBeLessThan(NFR_SPD_14_MS);
  });

  it('the lines of every sheet are the lines the backend sent: the template adds nothing', () => {
    const data = longPrintData(3, 4, { language });
    const { container } = render(<BikriPrintDocument data={data} />);
    const first = data.pages[0]?.lines[0];
    expect(container.querySelectorAll('.print-r5-detail')).toHaveLength(12);
    expect(container.textContent).toContain(first?.detail);
  });
});
