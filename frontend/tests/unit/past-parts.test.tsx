// The three parts of the new look of پچھلی بکری (plan 72, Umar 12-11-2026), each on its own in the Urdu
// and the English app: the divider (a rule and a pill, no buttons), the heading of a part with its chips
// (the day or the days, the count, the خام بکری, all as the backend sent them), and the two buttons at the
// end that move the window by 7 days (the key on each, the days each goes to under its words, "later" off
// with its reason on the latest window, the way back to the latest week, the arrow keys, the mirrored
// chevrons). The page that places them is tested in past-sales.test.tsx.

import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { PastDivider } from '../../src/features/bikri/components/PastDivider';
import { PastSectionHeader } from '../../src/features/bikri/components/PastSectionHeader';
import { PastWeekNav } from '../../src/features/bikri/components/PastWeekNav';
import { LANGUAGES, type Language } from '../../src/i18n';
import { paisaForTest } from '../helpers/paisa';
import { message, renderIn, visible } from '../helpers/render';

const TODAY = '2026-10-10';

describe.each(LANGUAGES)('the divider of پچھلی بکری (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  it('is a rule with a pill that says the earlier 7 days and the days they cover, and holds no button', () => {
    renderIn(language, <PastDivider from="2026-10-03" to="2026-10-09" />);
    const divider = screen.getByRole('separator', {
      name: (name) =>
        visible(name) === m('bikri-past-divider-label', { from: '03-10-2026', to: '09-10-2026' }),
    });
    expect(visible(divider.textContent)).toContain(m('bikri-past-divider-title', { days: 7 }));
    expect(visible(divider.textContent)).toContain(
      m('bikri-past-range', { from: '03-10-2026', to: '09-10-2026' }),
    );
    expect(screen.queryAllByRole('button')).toHaveLength(0);
    // The calendar icon is drawn, and the rules on both sides.
    expect(divider.querySelector('svg')).not.toBeNull();
    expect(divider.querySelectorAll('.bg-border')).toHaveLength(2);
  });

  it('writes its numbers in their own isolated run, so the range never flips', () => {
    renderIn(language, <PastDivider from="2026-10-03" to="2026-10-09" />);
    const runs = Array.from(document.querySelectorAll('bdi[dir="ltr"]')).map((run) =>
      visible(run.textContent),
    );
    expect(runs).toEqual(expect.arrayContaining(['03-10-2026', '09-10-2026', '7']));
  });
});

describe.each(LANGUAGES)('the heading of a part of پچھلی بکری (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const chips = () =>
    Array.from(document.querySelectorAll('li')).map((chip) => visible(chip.textContent));

  it("today's heading has its title, the day, the count and the خام بکری the backend sent", () => {
    renderIn(
      language,
      <PastSectionHeader part="today" date={TODAY} count={2} gross={paisaForTest(2_940_000)} />,
    );
    expect(visible(screen.getByRole('heading', { level: 2 }).textContent)).toBe(
      m('bikri-past-today-title'),
    );
    expect(chips()).toEqual([
      '10-10-2026',
      `${m('bikri-past-day-sales')}2`,
      `${m('glossary-kham-bikri')}29,400`,
    ]);
  });

  it("the window's heading says the earlier 7 days and shows the range as its first chip", () => {
    renderIn(
      language,
      <PastSectionHeader
        part="window"
        from="2026-10-03"
        to="2026-10-09"
        count={3}
        gross={paisaForTest(9_830_000)}
      />,
    );
    expect(visible(screen.getByRole('heading', { level: 2 }).textContent)).toBe(
      m('bikri-past-window-title', { days: 7 }),
    );
    expect(chips()[0]).toBe(m('bikri-past-range', { from: '03-10-2026', to: '09-10-2026' }));
    expect(chips()).toHaveLength(3);
    expect(chips()[2]).toContain('98,300');
  });

  it('leaves the count and the خام بکری out while they are not known, and the خام بکری when there is no بکری', () => {
    const { unmount } = renderIn(
      language,
      <PastSectionHeader part="today" date={TODAY} count={null} gross={null} />,
    );
    expect(chips()).toEqual(['10-10-2026']);
    unmount();
    renderIn(
      language,
      <PastSectionHeader part="today" date={TODAY} count={0} gross={paisaForTest(0)} />,
    );
    expect(chips()).toEqual(['10-10-2026', `${m('bikri-past-day-sales')}0`]);
  });

  it('writes the figures in digits 0 to 9, each in its own isolated run', () => {
    renderIn(
      language,
      <PastSectionHeader part="today" date={TODAY} count={12} gross={paisaForTest(12_345_600)} />,
    );
    expect(document.body.textContent).not.toMatch(/[۰-۹٠-٩]/u);
    const runs = Array.from(document.querySelectorAll('bdi[dir="ltr"]')).map((run) =>
      visible(run.textContent),
    );
    expect(runs).toEqual(expect.arrayContaining(['10-10-2026', '12', '123,456']));
  });
});

describe.each(LANGUAGES)('the buttons that move the window (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const earlier = () =>
    screen.getByRole('button', {
      name: (name) => visible(name).startsWith(m('bikri-past-back', { days: 7 })),
    });
  const later = () =>
    screen.getByRole('button', {
      name: (name) => visible(name).startsWith(m('bikri-past-forward', { days: 7 })),
    });
  const latestButton = () =>
    screen.queryByRole('button', {
      name: (name) => visible(name) === m('bikri-past-nav-latest-button'),
    });
  const draw = (
    end: string,
    handlers: Partial<Record<'onEarlier' | 'onNearer' | 'onLatest', () => void>> = {},
  ) =>
    renderIn(
      language,
      <PastWeekNav
        end={end}
        today={TODAY}
        onEarlier={handlers.onEarlier ?? (() => undefined)}
        onNearer={handlers.onNearer ?? (() => undefined)}
        onLatest={handlers.onLatest ?? (() => undefined)}
      />,
    );

  it('shows the key on each button and the days each one goes to under its words', () => {
    draw('2026-10-02');
    expect(visible(earlier().textContent)).toContain('Page Up');
    expect(visible(later().textContent)).toContain('Page Down');
    // From 26-09 to 02-10 is one week back; one week nearer is the window that ends yesterday.
    expect(visible(earlier().textContent)).toContain(
      m('bikri-past-range', { from: '19-09-2026', to: '25-09-2026' }),
    );
    expect(visible(later().textContent)).toContain(
      m('bikri-past-range', { from: '03-10-2026', to: '09-10-2026' }),
    );
    expect(later()).toBeEnabled();
  });

  it('on the window that ends yesterday the later button is off and the reason stands under it as plain text', () => {
    draw('2026-10-09');
    expect(later()).toBeDisabled();
    // The reason is not inside the dimmed button: it is its own text, linked to it for a screen reader.
    const reasonId = later().getAttribute('aria-describedby');
    expect(reasonId).not.toBeNull();
    const reason = document.getElementById(reasonId ?? '');
    expect(visible(reason?.textContent)).toBe(m('bikri-past-nav-latest'));
    expect(reason?.closest('button')).toBeNull();
    expect(visible(later().textContent)).not.toContain(
      m('bikri-past-range', { from: '10-10-2026', to: '16-10-2026' }),
    );
    // Nothing to jump back to.
    expect(latestButton()).toBeNull();
    // Earlier still goes, and says where.
    expect(earlier()).toBeEnabled();
    expect(visible(earlier().textContent)).toContain(
      m('bikri-past-range', { from: '26-09-2026', to: '02-10-2026' }),
    );
  });

  it('a window near yesterday goes nearer only as far as yesterday', () => {
    draw('2026-10-07');
    expect(visible(later().textContent)).toContain(
      m('bikri-past-range', { from: '03-10-2026', to: '09-10-2026' }),
    );
  });

  it('away from the latest window a small button jumps back to it', async () => {
    const onLatest = vi.fn();
    draw('2026-10-02', { onLatest });
    const jump = latestButton();
    if (jump === null) throw new Error('the way back to the latest week is not there');
    await userEvent.setup().click(jump);
    expect(onLatest).toHaveBeenCalledTimes(1);
  });

  it('the buttons call their handlers, and the later one does nothing when it is off', async () => {
    const onEarlier = vi.fn();
    const onNearer = vi.fn();
    const user = userEvent.setup();
    const { unmount } = draw('2026-10-02', { onEarlier, onNearer });
    await user.click(earlier());
    await user.click(later());
    expect(onEarlier).toHaveBeenCalledTimes(1);
    expect(onNearer).toHaveBeenCalledTimes(1);
    unmount();
    const onNearerOff = vi.fn();
    draw('2026-10-09', { onNearer: onNearerOff });
    await user.click(later());
    expect(onNearerOff).not.toHaveBeenCalled();
  });

  it('with focus on a button the arrow that points back in reading goes back, the other comes nearer, never past yesterday', async () => {
    const onEarlier = vi.fn();
    const onNearer = vi.fn();
    const user = userEvent.setup();
    draw('2026-10-02', { onEarlier, onNearer });
    // Urdu reads to the left, so the right arrow goes back; English reads to the right (PRD FR-7.13).
    const back = language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}';
    const nearer = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
    earlier().focus();
    await user.keyboard(back);
    expect(onEarlier).toHaveBeenCalledTimes(1);
    await user.keyboard(nearer);
    expect(onNearer).toHaveBeenCalledTimes(1);
  });

  it('on the latest window the arrow toward later does nothing', async () => {
    const onNearer = vi.fn();
    const user = userEvent.setup();
    draw('2026-10-09', { onNearer });
    earlier().focus();
    await user.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
    expect(onNearer).not.toHaveBeenCalled();
  });

  it('is reached with Tab in reading order: earlier, the way back to the latest week, later', async () => {
    const user = userEvent.setup();
    draw('2026-10-02');
    await user.tab();
    expect(document.activeElement).toBe(earlier());
    await user.tab();
    expect(document.activeElement).toBe(latestButton());
    await user.tab();
    expect(document.activeElement).toBe(later());
  });

  it('has the chevrons that turn round in right-to-left, and its group has a name', () => {
    draw('2026-10-02');
    const group = screen.getByRole('group', {
      name: (name) => visible(name) === m('bikri-past-nav-label'),
    });
    const chevrons = Array.from(group.querySelectorAll('svg')).map(
      (icon) => icon.getAttribute('class') ?? '',
    );
    expect(chevrons).toHaveLength(2);
    for (const classes of chevrons) expect(classes).toContain('rtl:rotate-180');
    expect(within(group).getAllByRole('button')).toHaveLength(3);
  });
});
