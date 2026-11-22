// Plan 95 item 11 (PRD FR-12.52): the پیشگی typed on the lots has a line of its own in the seller side of the summary, live and
// saved, after the net, because it is not part of the deductions or the net (C6, C7). It is shown only when it is not 0, and
// every figure is the backend's. In the Urdu and the English app.

import { within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { SummaryPanel } from '../../src/features/bikri/components/SummaryPanel';
import { SavedSummary } from '../../src/features/bikri/components/saved/SavedSummary';
import { LANGUAGES } from '../../src/i18n';
import { EVENT1 } from '../helpers/bikri-data';
import { savedEvent1 } from '../helpers/bikri-saved';
import { paisaForTest } from '../helpers/paisa';
import { message, renderIn, visible } from '../helpers/render';

const TWENTY_THOUSAND = paisaForTest(2_000_000);
const NOTHING = paisaForTest(0);

/** The seller-side rows of a summary as "label: figure", in order. */
function sellerRows(area: HTMLElement): { label: string; value: string }[] {
  const lists = area.querySelectorAll('dl');
  const first = lists[0];
  if (first === undefined) throw new Error('no seller side');
  return Array.from(first.querySelectorAll(':scope > div')).map((row) => ({
    label: visible(row.querySelector('dt')?.textContent),
    value: visible(row.querySelector('dd')?.textContent),
  }));
}

describe.each(LANGUAGES)('the پیشگی line of the summary (%s)', (language) => {
  const peshgi = message(language, 'glossary-peshgi');
  const net = message(language, 'glossary-safi-bikri');
  const totals = EVENT1.summary;

  const live = (advance: typeof NOTHING) =>
    renderIn(language, <SummaryPanel totals={{ ...totals, advance }} whole netIsNegative={false} />)
      .container;

  it('live: shows the advance after the net when it is not 0, and the net does not change', () => {
    const rows = sellerRows(live(TWENTY_THOUSAND));
    const labels = rows.map((row) => row.label);
    expect(labels.at(-1)).toBe(peshgi);
    expect(labels.at(-2)).toBe(net);
    expect(rows.at(-1)?.value).toBe('20,000');
    // The net is the one of the lot: the advance is not deducted from it.
    expect(rows.at(-2)?.value).toContain('19,316');
  });

  it('live: no line while it is 0', () => {
    const labels = sellerRows(live(NOTHING)).map((row) => row.label);
    expect(labels).not.toContain(peshgi);
    expect(labels.at(-1)).toBe(net);
  });

  it('saved: the same line from the stored figure', () => {
    const detail = savedEvent1();
    const withAdvance = {
      ...detail,
      totals: { ...detail.totals, advance: TWENTY_THOUSAND },
    };
    const { container } = renderIn(language, <SavedSummary detail={withAdvance} />);
    const rows = sellerRows(container);
    expect(rows.at(-1)?.label).toBe(peshgi);
    expect(rows.at(-1)?.value).toBe('20,000');
    expect(within(container).queryAllByText((text) => visible(text) === peshgi)).toHaveLength(1);
  });

  it('saved: no line when the advance is 0', () => {
    const { container } = renderIn(language, <SavedSummary detail={savedEvent1()} />);
    expect(sellerRows(container).map((row) => row.label)).not.toContain(peshgi);
  });
});
