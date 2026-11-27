// Plan 95 item 1 and plan 97 item 4 (PRD FR-12.52, FR-12.7, Umar 24-11-2026): a problem in the list above the lots (a
// warning such as a net below zero, or an error such as a garden lot that is not fully sold) shows for 5 seconds and then
// hides, and shows again each time the user repeats the action (the `pulse`, or the screen's count). The words come
// from the Fluent files, in both languages.

import { LocalizationProvider } from '@fluent/react';
import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ProblemList } from '../../src/features/bikri/components/ProblemList';
import { NOTICE_MS } from '../../src/features/bikri/notices';
import type { Problem } from '../../src/features/bikri/problems';
import { LANGUAGES } from '../../src/i18n';
import { createLocalization } from '../../src/i18n/localization';
import { message, visible } from '../helpers/render';

vi.mock('@tanstack/react-router', () => ({ useNavigate: () => vi.fn() }));

const WARNING: Problem = {
  item: { code: 'ERR_LOT_NET_NEGATIVE', field: 'lots[0].qty_received', values: { net: '-1,000' } },
  severity: 'warning',
  where: { kind: 'lot', lot: 0, sub: 'qty_received' },
};
const ERROR: Problem = {
  item: { code: 'ERR_QTY_EXCEEDS_LOT', field: 'lots[0].qty_received', values: { available: 5 } },
  severity: 'error',
  where: { kind: 'lot', lot: 0, sub: 'qty_received' },
};

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'Date'] });
});

afterEach(() => {
  vi.useRealTimers();
});

describe.each(LANGUAGES)('a lot warning above the lots (%s)', (language) => {
  const words = () => visible(message(language, 'err-lot-net-negative', { net: '-1,000' }));
  const shown = () => screen.queryAllByRole('status').map((node) => visible(node.textContent));
  // The same providers on every draw, so a rerender keeps them.
  const draw = (pulse: number, problems: readonly Problem[] = [WARNING]) => (
    <LocalizationProvider l10n={createLocalization(language)}>
      <ProblemList problems={problems} selectedLot={0} activeLine={0} pulse={pulse} />
    </LocalizationProvider>
  );

  it('shows at once, hides after 5 seconds, and shows again when the user repeats the action', () => {
    const view = render(draw(0));
    expect(shown().join(' ')).toContain(words().slice(0, 20));
    // Just before the end it is still there.
    act(() => {
      vi.advanceTimersByTime(NOTICE_MS - 100);
    });
    expect(shown()).toHaveLength(1);
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(shown()).toHaveLength(0);
    // The user typed in the lots again: the same warning shows for another 5 seconds.
    view.rerender(draw(1));
    expect(shown()).toHaveLength(1);
    act(() => {
      vi.advanceTimersByTime(NOTICE_MS + 100);
    });
    expect(shown()).toHaveLength(0);
  });

  it('a new answer of the preview with the same warning does not show it again by itself', () => {
    const view = render(draw(0));
    act(() => {
      vi.advanceTimersByTime(NOTICE_MS + 100);
    });
    expect(shown()).toHaveLength(0);
    // The same warning arrives again from a background preview: it is not a user action.
    view.rerender(draw(0, [{ ...WARNING, item: { ...WARNING.item, values: { net: '-1,000' } } }]));
    expect(shown()).toHaveLength(0);
  });

  it('a warning that goes and comes back shows again', () => {
    const view = render(draw(0, [WARNING]));
    act(() => {
      vi.advanceTimersByTime(NOTICE_MS + 100);
    });
    expect(shown()).toHaveLength(0);
    // The net was fixed (the warning is gone) and then broken again: it is a new warning.
    view.rerender(draw(0, []));
    view.rerender(draw(0, [WARNING]));
    expect(shown()).toHaveLength(1);
  });

  // Plan 97 item 4: an error hides too (Umar's screenshot: "the garden lot must be fully sold in this بکری" stayed on screen).
  // The red mark on the cell stays until it is fixed, and the save is refused while it stands; only the words come and go.
  it('an error in the list hides after 5 seconds too, and shows again when the user repeats the action', () => {
    const view = render(draw(0, [ERROR]));
    expect(screen.getAllByRole('alert')).toHaveLength(1);
    act(() => {
      vi.advanceTimersByTime(NOTICE_MS - 100);
    });
    expect(screen.queryAllByRole('alert')).toHaveLength(1);
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(screen.queryAllByRole('alert')).toHaveLength(0);
    // The same key again, or a try to save: the words are back for another 5 seconds.
    view.rerender(draw(1, [ERROR]));
    expect(screen.getAllByRole('alert')).toHaveLength(1);
    act(() => {
      vi.advanceTimersByTime(NOTICE_MS + 100);
    });
    expect(screen.queryAllByRole('alert')).toHaveLength(0);
  });

  it('an error and a warning at once hide together, and a repeat shows both', () => {
    const view = render(draw(0, [WARNING, ERROR]));
    expect(screen.getAllByRole('alert')).toHaveLength(1);
    expect(shown()).toHaveLength(1);
    act(() => {
      vi.advanceTimersByTime(NOTICE_MS + 100);
    });
    expect(screen.queryAllByRole('alert')).toHaveLength(0);
    expect(shown()).toHaveLength(0);
    view.rerender(draw(1, [WARNING, ERROR]));
    expect(screen.getAllByRole('alert')).toHaveLength(1);
    expect(shown()).toHaveLength(1);
  });
});
