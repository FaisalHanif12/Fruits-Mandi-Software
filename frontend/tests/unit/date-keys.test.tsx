// Date field keys (PRD FR-7.19): + is the next day, - the previous day, and a shortcut such as
// "10" or "10-9" is read before the key moves it.

import { fireEvent, screen } from '@testing-library/react';
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

describe.each(LANGUAGES)('DateInput keys (%s)', (language) => {
  it('+ moves to the next day and - to the previous day, and arrows are left alone', () => {
    renderIn(language, <Harness initial="10-10-2026" />);
    const input = screen.getByRole('textbox');
    expect(fireEvent.keyDown(input, { code: 'Equal', key: '+', shiftKey: true })).toBe(false);
    expect(input).toHaveValue('11-10-2026');
    expect(fireEvent.keyDown(input, { code: 'NumpadAdd', key: '+' })).toBe(false);
    expect(input).toHaveValue('12-10-2026');
    expect(fireEvent.keyDown(input, { code: 'Minus', key: '-' })).toBe(false);
    expect(input).toHaveValue('11-10-2026');
    expect(fireEvent.keyDown(input, { code: 'NumpadSubtract', key: '-' })).toBe(false);
    expect(input).toHaveValue('10-10-2026');
    // An arrow key is not taken by the field, so the form can move on.
    expect(fireEvent.keyDown(input, { code: 'ArrowLeft', key: 'ArrowLeft' })).toBe(true);
    expect(input).toHaveValue('10-10-2026');
  });

  it('does not move a read-only date', () => {
    renderIn(language, <DateInput value="10-10-2026" onChange={() => undefined} readOnly />);
    expect(fireEvent.keyDown(screen.getByRole('textbox'), { code: 'Equal', key: '+' })).toBe(true);
  });
});
