// Plan 93 T8 (Umar, 22-11-2026): the error of a field in a compact row (the date of the sale book) is a bubble that
// floats under the field, not a line in the row. Long words ("A future date is not allowed. Enter today (22-11-2026) or an
// earlier date.") used to widen the one-line toolbar and push its tabs out. Now the row never changes size: the bubble is
// drawn in a portal, kept inside the window; the field keeps its red border and `aria-invalid` while the value is wrong;
// the words stay for a screen reader (`aria-describedby`); the bubble hides after 5 seconds and shows again each time the
// user repeats the action (a change, a blur, a try to save). In the Urdu and in the English app. A browser lays things out;
// here the structure that keeps the row the same size is held.

import { act, fireEvent, render, screen } from '@testing-library/react';
import { Localized } from '@fluent/react';
import { StrictMode, useEffect, useState } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { DateInput, ERROR_BUBBLE_MS, Field, fieldError } from '../../src/shared/ui';
import { refusal } from '../helpers/ui-error';
import { message, renderIn, visible } from '../helpers/render';

const TODAY = '22-11-2026';
const ERROR = refusal([{ code: 'ERR_DATE_IN_FUTURE', field: 'date', values: { today: TODAY } }]);

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

function Row({
  failing = true,
  pulse,
  inline = true,
}: {
  failing?: boolean;
  pulse?: number;
  inline?: boolean;
}) {
  return (
    <div data-testid="row">
      <Field
        inline={inline}
        bubblePulse={pulse}
        label={<Localized id="bikri-header-date" />}
        error={failing ? fieldError(ERROR, 'date') : undefined}
      >
        {(control) => <DateInput {...control} value="23-11-2026" onChange={() => undefined} />}
      </Field>
    </div>
  );
}

type RowProps = Parameters<typeof Row>[0];
type Change = (props: Partial<RowProps>) => void;

/** The row with props the test can change in place (inside the same language and direction providers). */
function Harness({ changeRef, ...first }: RowProps & { changeRef: { current: Change | null } }) {
  const [props, set] = useState<Partial<RowProps>>(first);
  useEffect(() => {
    changeRef.current = (next) => {
      set((now) => ({ ...now, ...next }));
    };
  }, [changeRef]);
  return <Row {...props} />;
}

const words = (language: Language) =>
  visible(message(language, 'err-date-in-future', { today: TODAY }));
/** The bubble in the window: outside the row, in a portal. */
const bubble = () => document.querySelector('[data-error-bubble]');

describe.each(LANGUAGES)('the error bubble of a field in a compact row (%s)', (language) => {
  const changeRef: { current: Change | null } = { current: null };
  const setRowProps: Change = (next) => {
    changeRef.current?.(next);
  };
  const show = (props: Partial<RowProps> = {}) =>
    renderIn(language, <Harness changeRef={changeRef} {...props} />);

  it('floats: the visible words are in a bubble outside the row, so the row has no line of its own for the error', () => {
    show();
    const row = screen.getByTestId('row');
    const found = bubble();
    expect(found).not.toBeNull();
    expect(visible(found?.textContent)).toContain(words(language));
    // In a portal, outside the row: it cannot change the row's size.
    expect(row.contains(found)).toBe(false);
    // The row holds the label and the control, and a line for readers that takes no room.
    const lines = Array.from(row.querySelectorAll('p'));
    expect(lines).toHaveLength(1);
    expect(lines[0]).toHaveClass('sr-only');
    expect(visible(lines[0]?.textContent)).toContain(words(language));
    // No visible error line under the field.
    expect(row.querySelector('p.text-due')).toBeNull();
  });

  it('keeps the red border and aria-invalid, and the words for a screen reader, whether the bubble shows or not', () => {
    show();
    const input = screen.getByRole('textbox');
    expect(input).toHaveAttribute('aria-invalid', 'true');
    const describedBy = input.getAttribute('aria-describedby') ?? '';
    expect(describedBy).not.toBe('');
    expect(visible(document.getElementById(describedBy)?.textContent)).toContain(words(language));
    act(() => {
      vi.advanceTimersByTime(ERROR_BUBBLE_MS);
    });
    expect(bubble()).toBeNull();
    // The border and the words stay while the value is wrong.
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(visible(document.getElementById(describedBy)?.textContent)).toContain(words(language));
    // The bubble is only a picture of the line: readers hear the line, not the bubble twice.
    show();
    expect(bubble()?.getAttribute('aria-hidden')).toBe('true');
  });

  it('shows for 5 seconds (a named constant) and then hides', () => {
    expect(ERROR_BUBBLE_MS).toBe(5_000);
    show();
    expect(bubble()).not.toBeNull();
    act(() => {
      vi.advanceTimersByTime(ERROR_BUBBLE_MS - 1);
    });
    expect(bubble()).not.toBeNull();
    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(bubble()).toBeNull();
  });

  it('the same error sent again unchanged, as a background preview does, is not a user action: the bubble still hides at 5,000 ms', () => {
    show();
    expect(bubble()).not.toBeNull();
    // The screen draws again with the same refusal (a new object each time) at 1, 2, 3 and 4 seconds.
    for (let second = 1; second <= 4; second += 1) {
      act(() => {
        vi.advanceTimersByTime(1000);
      });
      act(() => {
        setRowProps({ failing: true });
      });
      expect(bubble()).not.toBeNull();
    }
    act(() => {
      vi.advanceTimersByTime(999);
    });
    expect(bubble()).not.toBeNull();
    act(() => {
      vi.advanceTimersByTime(1);
    });
    // 5,000 ms after it showed: hidden, though the error was sent again four times meanwhile.
    expect(bubble()).toBeNull();
    // And it stays hidden when the same error is sent again later.
    act(() => {
      setRowProps({ failing: true });
      vi.advanceTimersByTime(10_000);
    });
    expect(bubble()).toBeNull();
  });

  it('hides at 5,000 ms in strict mode too, where an effect runs twice (the development build)', () => {
    renderIn(
      language,
      <StrictMode>
        <Harness changeRef={changeRef} />
      </StrictMode>,
    );
    expect(bubble()).not.toBeNull();
    act(() => {
      vi.advanceTimersByTime(ERROR_BUBBLE_MS - 1);
    });
    expect(bubble()).not.toBeNull();
    act(() => {
      vi.advanceTimersByTime(1);
    });
    expect(bubble()).toBeNull();
  });

  it('shows again when the user repeats the action: a change, a blur, and a try to save', () => {
    show({ pulse: 0 });
    const input = screen.getByRole('textbox');
    const hideIt = () => {
      act(() => {
        vi.advanceTimersByTime(ERROR_BUBBLE_MS);
      });
      expect(bubble()).toBeNull();
    };
    // A change that still fails.
    hideIt();
    fireEvent.input(input, { target: { value: '24-11-2026' } });
    expect(bubble()).not.toBeNull();
    // A blur that still fails.
    hideIt();
    fireEvent.blur(input);
    expect(bubble()).not.toBeNull();
    // A try to save (the screen raises the pulse).
    hideIt();
    act(() => {
      setRowProps({ pulse: 1 });
    });
    expect(bubble()).not.toBeNull();
    // The red border stayed the whole time.
    expect(input).toHaveAttribute('aria-invalid', 'true');
  });

  it('goes away when the value is right, and comes back with a new error', () => {
    show();
    expect(bubble()).not.toBeNull();
    act(() => {
      setRowProps({ failing: false });
    });
    expect(bubble()).toBeNull();
    expect(screen.getByRole('textbox')).not.toHaveAttribute('aria-invalid', 'true');
    act(() => {
      setRowProps({ failing: true });
    });
    expect(bubble()).not.toBeNull();
  });

  it('a field that is not in a compact row keeps its error under it, as before', () => {
    show({ inline: false });
    expect(bubble()).toBeNull();
    const line = screen.getByTestId('row').querySelector('p.text-due');
    expect(visible(line?.textContent)).toContain(words(language));
  });
});

describe('the bubble is not part of the layout', () => {
  it('is positioned by the popper in a portal and takes no focus, so typing goes on', () => {
    render(<div />);
    renderIn('en', <Row />);
    const found = bubble();
    expect(found?.closest('[data-radix-popper-content-wrapper]')).not.toBeNull();
    // It does not take the focus: the control keeps it.
    const input = screen.getByRole('textbox');
    input.focus();
    act(() => {
      vi.advanceTimersByTime(10);
    });
    expect(input).toHaveFocus();
  });
});
