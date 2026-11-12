// The live summary card keeps a label, its figure and its ⓘ on one line (plan 71 follow-up, PRD 12.5,
// F17, NFR-USE-19). At the wide step the card is a narrow third column, and two sides of the summary in
// two columns made the ⓘ of one row sit on the next column's label and cut the figure at the edge
// (1920, English). The two sides now follow the CARD's own width, with a container query: one under the
// other until the card is as wide as the `form3` step (640 px), side by side from there. jsdom does not
// lay out, so this reads the structure and the token; the sizes (1024, 1366, 1920, both languages) were
// measured in WebKit and are in the handoff.

import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { SummaryPanel } from '../../src/features/bikri/components/SummaryPanel';
import { LANGUAGES } from '../../src/i18n';
import { EVENT1 } from '../helpers/bikri-data';
import { message, renderIn, visible } from '../helpers/render';

const TOKENS = fs.readFileSync(
  path.resolve(import.meta.dirname, '../../src/styles/tokens.css'),
  'utf8',
);

describe.each(LANGUAGES)('the summary card (%s)', (language) => {
  const draw = () =>
    renderIn(
      language,
      <SummaryPanel
        totals={EVENT1.lots[0]?.totals ?? null}
        whole={false}
        lotNumber={1}
        netIsNegative={false}
      />,
    );

  it('puts its two sides in a grid that is one column until the card itself is wide enough', () => {
    const { container } = draw();
    const grid = container.querySelector('.grid');
    expect(grid).not.toBeNull();
    expect(grid?.className).toContain('grid-cols-1');
    expect(grid?.className).toContain('@form3:grid-cols-2');
    // The grid answers to the width of the card, so a wrapper is the container, not the window.
    expect(grid?.parentElement?.className).toContain('@container');
    // Never a fixed two columns whatever the width (the defect of the wide step).
    expect(grid?.className).not.toMatch(/(^|\s)grid-cols-2(\s|$)/);
  });

  it('keeps the two sides in their order: the seller side, then the buyer side', () => {
    const { container } = draw();
    const headings = Array.from(container.querySelectorAll('h3')).map((h) =>
      visible(h.textContent),
    );
    expect(headings).toEqual([
      message(language, 'bikri-summary-seller-side'),
      message(language, 'bikri-summary-buyer-side'),
    ]);
  });

  it('has the label, the figure and its ⓘ in one row, for every line of both sides', () => {
    const { container } = draw();
    const rows = Array.from(container.querySelectorAll('dl > div'));
    // Ten lines on the seller side and five on the buyer side (no rate difference in Event 1).
    expect(rows).toHaveLength(15);
    for (const row of rows) {
      expect(row.querySelectorAll(':scope > dt')).toHaveLength(1);
      const value = row.querySelector(':scope > dd');
      expect(value?.className).toContain('value-col');
      expect(value?.querySelector('bdi[dir="ltr"]')).not.toBeNull();
      expect(value?.querySelectorAll('button')).toHaveLength(1);
    }
  });

  it('shows the figures the backend sent, written left to right', () => {
    const { container } = draw();
    const figures = Array.from(container.querySelectorAll('dd bdi[dir="ltr"]')).map((bdi) =>
      visible(bdi.textContent),
    );
    expect(figures).toEqual(expect.arrayContaining(['29,400', '1,764', '10,084', '19,316']));
  });
});

describe('the step at which the two sides go side by side', () => {
  it('is a plain length of 640 px, as a container query needs (never var())', () => {
    expect(TOKENS).toMatch(/--container-form3:\s*640px;/);
  });
});
