// Date field keys (PRD FR-7.19): + is the next day, - the previous day, and a shortcut such as
// "10" or "10-9" is read before the key moves it. The two keys are shortcuts only while the field is empty
// or all of its text is selected (the way it stands on arrival, FR-7.15); after a digit "-" is the dash of
// "5-9" or "05-10-2026" and is typed (robustness finding, plan 72). The tests send the REAL key, with the
// code `Minus`, which user-event leaves out for a plain "-": so the dash a keyboard types is tested too.

import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { readDateText, shiftDateText } from '../../src/shared/format';
import { DateInput } from '../../src/shared/ui';
import { renderIn } from '../helpers/render';

const TODAY = new Date(2026, 9, 10); // 10-10-2026 on the computer's calendar

describe('shiftDateText', () => {
  it.each([
    ['10-10-2026', 1, '11-10-2026'],
    ['10-10-2026', -1, '09-10-2026'],
    ['31-10-2026', 1, '01-11-2026'],
    ['01-01-2027', -1, '31-12-2026'],
    ['28-02-2028', 1, '29-02-2028'],
    ['29-02-2028', 1, '01-03-2028'],
    ['01-03-2026', -1, '28-02-2026'],
    ['5-3-2026', 1, '06-03-2026'],
  ] as const)('%s moved by %i gives %s', (text, days, expected) => {
    expect(shiftDateText(text, days, TODAY)).toBe(expected);
  });

  it('reads the typing shortcuts first: "10" is this month, "10-9" is this year', () => {
    expect(shiftDateText('15', 1, TODAY)).toBe('16-10-2026');
    expect(shiftDateText('10-9', -1, TODAY)).toBe('09-09-2026');
  });

  it('starts from today when the text is empty, partial or not a real date', () => {
    expect(shiftDateText('', 1, TODAY)).toBe('11-10-2026');
    expect(shiftDateText('', -1, TODAY)).toBe('09-10-2026');
    expect(shiftDateText('31-02-2026', 1, TODAY)).toBe('11-10-2026');
    expect(shiftDateText('abc', -1, TODAY)).toBe('09-10-2026');
    expect(shiftDateText('10-10-', 1, TODAY)).toBe('11-10-2026');
  });

  it('readDateText accepts only real dates', () => {
    expect(readDateText('29-02-2027', TODAY)).toBeNull();
    expect(readDateText('29-02-2028', TODAY)).toEqual({ day: 29, month: 2, year: 2028 });
    expect(readDateText('32', TODAY)).toBeNull();
    expect(readDateText('1-13', TODAY)).toBeNull();
  });
});

function Harness({ initial }: { initial: string }) {
  const [value, setValue] = useState(initial);
  return <DateInput value={value} onChange={setValue} />;
}

/** A keyboard that has the real key `Minus`, so `[Minus]` presses it with its code, as a keyboard does. */
const REAL_KEYS = [{ code: 'Minus', key: '-' }];

describe.each(LANGUAGES)('DateInput keys (%s)', (language) => {
  const input = () => screen.getByRole<HTMLInputElement>('textbox');
  /** What the browser does on arrival in a field: all of the text is selected (FR-7.15). */
  const selectAll = () => {
    input().focus();
    input().select();
  };

  it('+ moves to the next day and - to the previous day while all the text is selected, and arrows are left alone', () => {
    renderIn(language, <Harness initial="10-10-2026" />);
    selectAll();
    expect(fireEvent.keyDown(input(), { code: 'Equal', key: '+', shiftKey: true })).toBe(false);
    expect(input()).toHaveValue('11-10-2026');
    // The whole text is selected again after a move, so the key can be pressed again for the next day.
    expect(input().selectionStart).toBe(0);
    expect(input().selectionEnd).toBe(10);
    expect(fireEvent.keyDown(input(), { code: 'NumpadAdd', key: '+' })).toBe(false);
    expect(input()).toHaveValue('12-10-2026');
    expect(fireEvent.keyDown(input(), { code: 'Minus', key: '-' })).toBe(false);
    expect(input()).toHaveValue('11-10-2026');
    expect(fireEvent.keyDown(input(), { code: 'NumpadSubtract', key: '-' })).toBe(false);
    expect(input()).toHaveValue('10-10-2026');
    // An arrow key is not taken by the field, so the form can move on.
    expect(fireEvent.keyDown(input(), { code: 'ArrowLeft', key: 'ArrowLeft' })).toBe(true);
    expect(input()).toHaveValue('10-10-2026');
  });

  it('on an empty field the keys move from today', () => {
    renderIn(language, <Harness initial="" />);
    input().focus();
    expect(fireEvent.keyDown(input(), { code: 'Minus', key: '-' })).toBe(false);
    expect(input()).toHaveValue(shiftDateText('', -1, new Date()));
  });

  it('after a digit "-" and "+" are typed, not taken as shortcuts', () => {
    renderIn(language, <Harness initial="5" />);
    input().focus();
    input().setSelectionRange(1, 1);
    // `fireEvent` says true when the page did not stop the key, so the browser types the character.
    expect(fireEvent.keyDown(input(), { code: 'Minus', key: '-' })).toBe(true);
    expect(fireEvent.keyDown(input(), { code: 'NumpadSubtract', key: '-' })).toBe(true);
    expect(fireEvent.keyDown(input(), { code: 'Equal', key: '+', shiftKey: true })).toBe(true);
    expect(fireEvent.keyDown(input(), { code: 'NumpadAdd', key: '+' })).toBe(true);
    expect(input()).toHaveValue('5');
  });

  it('a complete date with the cursor in it, or a part of it selected, types the key too', () => {
    renderIn(language, <Harness initial="10-10-2026" />);
    input().focus();
    input().setSelectionRange(10, 10);
    expect(fireEvent.keyDown(input(), { code: 'Minus', key: '-' })).toBe(true);
    input().setSelectionRange(0, 2);
    expect(fireEvent.keyDown(input(), { code: 'Minus', key: '-' })).toBe(true);
    expect(input()).toHaveValue('10-10-2026');
  });

  it('types 5-9 and 05-10-2026 with the real minus key', async () => {
    const user = userEvent.setup({ keyboardMap: REAL_KEYS });
    renderIn(language, <Harness initial="" />);
    await user.click(input());
    await user.keyboard('5[Minus]9');
    expect(input()).toHaveValue('5-9');
    await user.clear(input());
    await user.keyboard('05[Minus]10[Minus]2026');
    expect(input()).toHaveValue('05-10-2026');
  });

  it('the real minus key still moves the day on an empty field and on a fully selected one', async () => {
    const user = userEvent.setup({ keyboardMap: REAL_KEYS });
    renderIn(language, <Harness initial="10-10-2026" />);
    selectAll();
    await user.keyboard('[Minus]');
    expect(input()).toHaveValue('09-10-2026');
    // And once more without selecting again: the text was selected for it.
    await user.keyboard('[Minus]');
    expect(input()).toHaveValue('08-10-2026');
    await user.clear(input());
    await user.keyboard('[Minus]');
    expect(input()).toHaveValue(shiftDateText('', -1, new Date()));
  });

  it('does not move a read-only date', () => {
    renderIn(language, <DateInput value="10-10-2026" onChange={() => undefined} readOnly />);
    expect(fireEvent.keyDown(screen.getByRole('textbox'), { code: 'Equal', key: '+' })).toBe(true);
  });
});
