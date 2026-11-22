// Plan 93 T6 (Umar, 22-11-2026): a row of buttons that is always ONE line, with the buttons at their natural width and
// the ones that do not fit behind a "More" button at the end of the row. Used by the quick actions of the dashboard. The
// arithmetic is a plain function (shared with the tabs of a section); the row is checked with a measured width, in the
// Urdu and in the English app.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { Button, OverflowRow, itemsThatFit, sectionTabsThatFit } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';
import { at } from '../helpers/sample';

afterEach(() => {
  vi.restoreAllMocks();
});

describe('how many items fit', () => {
  it('shows all of them when the whole row fits, or when the row has no width yet', () => {
    expect(itemsThatFit(1000, [100, 100, 100], 80, 8)).toBe(3);
    expect(itemsThatFit(0, [100, 100, 100], 80, 8)).toBe(3);
  });

  it('keeps room for the More button and always shows one item', () => {
    // 100 + 8 + 100 + 8 + 100 = 316 does not fit 250: More (80) takes room, 80 + 8 + 100 = 188 fits, + 8 + 100 does not.
    expect(itemsThatFit(250, [100, 100, 100], 80, 8)).toBe(1);
    expect(itemsThatFit(300, [100, 100, 100], 80, 8)).toBe(2);
    expect(itemsThatFit(10, [100, 100, 100], 80, 8)).toBe(1);
  });

  it('is the same arithmetic as the tabs of a section, with its own gap of 4', () => {
    expect(sectionTabsThatFit(300, [100, 100, 100], 80)).toBe(
      itemsThatFit(300, [100, 100, 100], 80, 4),
    );
  });
});

describe.each(LANGUAGES)('the row of buttons (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);
  const IDS = ['common-back', 'common-retry', 'common-loading', 'common-select-placeholder'];

  function Row({ onSelect = vi.fn() }: { onSelect?: (id: string) => void }) {
    return (
      <>
        <h2 id="row-title">{m('common-notifications')}</h2>
        <OverflowRow
          aria-labelledby="row-title"
          items={IDS.map((id) => ({
            id,
            label: m(id),
            onSelect: () => {
              onSelect(id);
            },
            node: (
              <Button
                onClick={() => {
                  onSelect(id);
                }}
              >
                {m(id)}
              </Button>
            ),
          }))}
        />
      </>
    );
  }

  const group = () => screen.getByRole('group', { name: m('common-notifications') });
  const buttons = () => within(group()).getAllByRole('button');

  it('shows every button at its natural width, and no More button, before the row has a width', () => {
    renderIn(language, <Row />);
    expect(buttons().map((button) => visible(button.textContent))).toEqual(IDS.map(m));
    expect(screen.queryByRole('button', { name: m('common-tabs-more') })).toBeNull();
    // Never stretched: no grid, no full-width button.
    for (const button of buttons()) {
      expect(button).not.toHaveClass('w-full');
      expect(button).not.toHaveClass('justify-center');
    }
  });

  it('measures with an invisible, inert, zero-height ruler that no screen reader or key reaches', () => {
    renderIn(language, <Row />);
    const ruler = group().querySelector('[aria-hidden="true"]');
    expect(ruler).toHaveClass('h-0', 'invisible', 'overflow-hidden');
    expect(ruler?.hasAttribute('inert')).toBe(true);
    // The buttons of the ruler are copies: the group's reachable buttons are the shown ones only.
    expect(ruler?.querySelectorAll('button')).toHaveLength(IDS.length);
    expect(buttons()).toHaveLength(IDS.length);
  });

  it('puts the buttons that do not fit behind More at the end of the row, and a pick from the menu does that action', async () => {
    // A row 400 wide, buttons 150 wide and a More button 150 wide: one button fits beside More (150 + 8 + 150 = 308).
    vi.spyOn(HTMLElement.prototype, 'clientWidth', 'get').mockReturnValue(400);
    vi.spyOn(HTMLElement.prototype, 'offsetWidth', 'get').mockReturnValue(150);
    const onSelect = vi.fn();
    renderIn(language, <Row onSelect={onSelect} />);
    const user = userEvent.setup();
    await waitFor(() => {
      expect(buttons()).toHaveLength(2);
    });
    // The first button, then More: one row, in this order.
    expect(visible(at(buttons(), 0).textContent)).toBe(m(IDS[0] ?? ''));
    const more = at(buttons(), 1);
    expect(visible(more.textContent)).toBe(m('common-tabs-more'));
    await user.click(more);
    const items = await screen.findAllByRole('menuitem');
    expect(items.map((item) => visible(item.textContent))).toEqual(IDS.slice(1).map(m));
    await user.click(at(items, 1));
    expect(onSelect).toHaveBeenCalledWith(IDS[2]);
  });
});
