// The two parts every one-month window shares (plan 76, PRD FR-9.3, FR-17.12, FR-19.11): the buttons that
// move by a month (`MonthNav`) and the box that goes to a month (`GoToMonth`), each in the Urdu and the
// English app. The date work is tested in format-months.test.ts; here are the words, the buttons that
// go off at the two ends, the arrow keys, and the refusals in words.

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { GoToMonth, MonthNav } from '../../src/shared/ui';
import { message, renderIn, visible } from '../helpers/render';

const TODAY = '2026-11-12';

describe.each(LANGUAGES)('the month buttons (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const button = (id: string) =>
    screen.getByRole('button', { name: (name) => visible(name).includes(m(id)) });

  it('on the current month: the later button is off and says why; the earlier one shows the month it goes to', () => {
    renderIn(language, <MonthNav month="2026-11" today={TODAY} first="2026-02" onMove={vi.fn()} />);
    expect((button('common-month-later') as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(m('common-month-latest-note'))).toBeTruthy();
    expect((button('common-month-earlier') as HTMLButtonElement).disabled).toBe(false);
    // The words under the earlier button name the month before: October 2026.
    expect(visible(button('common-month-earlier').textContent)).toContain(
      m('common-month-title', { month: m('report-month-10'), year: '2026' }),
    );
    // No way back to "this month" when it is this month.
    expect(screen.queryByRole('button', { name: m('common-month-latest-button') })).toBeNull();
  });

  it('moves to the month before and, from there, back to this month', async () => {
    const onMove = vi.fn();
    renderIn(language, <MonthNav month="2026-11" today={TODAY} first="2026-02" onMove={onMove} />);
    await userEvent.click(button('common-month-earlier'));
    expect(onMove).toHaveBeenCalledWith('2026-10');
  });

  it('on an earlier month: the later button works and a button goes straight to this month', async () => {
    const onMove = vi.fn();
    renderIn(language, <MonthNav month="2026-08" today={TODAY} first="2026-02" onMove={onMove} />);
    await userEvent.click(button('common-month-later'));
    expect(onMove).toHaveBeenLastCalledWith('2026-09');
    await userEvent.click(screen.getByRole('button', { name: m('common-month-latest-button') }));
    expect(onMove).toHaveBeenLastCalledWith('2026-11');
  });

  it('on the first month with entries the earlier button is off and says why', () => {
    renderIn(language, <MonthNav month="2026-02" today={TODAY} first="2026-02" onMove={vi.fn()} />);
    expect((button('common-month-earlier') as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(m('common-month-first-note'))).toBeTruthy();
  });

  it('moves with the arrow that points the way the text reads', async () => {
    const onMove = vi.fn();
    renderIn(language, <MonthNav month="2026-08" today={TODAY} first={null} onMove={onMove} />);
    button('common-month-later').focus();
    // Urdu reads from the right: the arrow towards the start (right) goes earlier. English: the left one.
    await userEvent.keyboard(language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}');
    expect(onMove).toHaveBeenLastCalledWith('2026-07');
    await userEvent.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
    expect(onMove).toHaveBeenLastCalledWith('2026-09');
  });
});

describe.each(LANGUAGES)('the go-to-month box (%s)', (language: Language) => {
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const box = () => screen.getByRole('textbox');

  it('goes to the month typed, in the short forms too, and empties the box', async () => {
    const onGo = vi.fn();
    renderIn(language, <GoToMonth today={TODAY} first="2026-02" onGo={onGo} />);
    await userEvent.type(box(), '9{Enter}');
    expect(onGo).toHaveBeenLastCalledWith('2026-09');
    expect((box() as HTMLInputElement).value).toBe('');
    await userEvent.type(box(), '03-2026{Enter}');
    expect(onGo).toHaveBeenLastCalledWith('2026-03');
  });

  it('refuses a month that is not real, in words, and moves nothing', async () => {
    const onGo = vi.fn();
    renderIn(language, <GoToMonth today={TODAY} first="2026-02" onGo={onGo} />);
    await userEvent.type(box(), '13{Enter}');
    expect(screen.getByText(m('err-month-invalid'))).toBeTruthy();
    expect(onGo).not.toHaveBeenCalled();
  });

  it('refuses a month after this one and a month before the first one with entries', async () => {
    const onGo = vi.fn();
    renderIn(language, <GoToMonth today={TODAY} first="2026-02" onGo={onGo} />);
    await userEvent.type(box(), '12-2026{Enter}');
    // The month is a Fluent variable, so the page wraps it in isolation marks: the visible text is compared.
    expect(visible(document.body.textContent)).toContain(
      m('err-month-in-future', { month: '11-2026' }),
    );
    await userEvent.clear(box());
    await userEvent.type(box(), '01-2026{Enter}');
    expect(visible(document.body.textContent)).toContain(
      m('err-month-before-first', { month: '02-2026' }),
    );
    expect(onGo).not.toHaveBeenCalled();
  });
});
