// The numbered steps under a "how do I print this?" answer and the slower-question lines (plan 54 S2
// and S6, plan 55). The steps and the lines are written by the panel in its own words, from the ids and
// values the backend sends; they work nothing out.

import { act, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AssistantStep } from '../../src/bindings/AssistantStep';
import type { ReportMessageArg } from '../../src/bindings/ReportMessageArg';
import { HowSteps, type HowStepView } from '../../src/features/assistant/components/HowSteps';
import { StepLines } from '../../src/features/assistant/components/StepLines';
import { LANGUAGES, type Language } from '../../src/i18n';
import { resetClientState } from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';

const text = (name: string, value: string): ReportMessageArg => ({
  name,
  value: { type: 'text', text: value },
});

const names = (name: string, ur: string, en: string): ReportMessageArg => ({
  name,
  value: { type: 'names', names: { ur, en } },
});

const AKRAM_UR = 'اکرم ٹول ٹیکس';
const AKRAM_EN = 'Akram Toll Tax';

/** The steps of a buyer's slip for one day, in the order the backend sends them. */
function slipSteps(): HowStepView[] {
  return [
    { id: 'assistant-how-open-report', args: [text('report', 'R5')] },
    {
      id: 'assistant-how-set-party',
      args: [text('role', 'buyer'), names('party', AKRAM_UR, AKRAM_EN), text('code', '03040001')],
    },
    { id: 'assistant-how-set-day', args: [text('day', '2026-10-10')] },
    { id: 'assistant-how-press-print', args: [] },
  ];
}

describe.each(LANGUAGES)('the how-to steps (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
  });

  it('are a numbered list in the order sent, in the app’s words, with dates as DD-MM-YYYY', () => {
    renderIn(language, <HowSteps steps={slipSteps()} />);
    const list = screen.getByRole('list', { name: message(language, 'assistant-how-label') });
    expect(list.tagName).toBe('OL');
    const items = within(list).getAllByRole('listitem');
    const akram = language === 'ur' ? AKRAM_UR : AKRAM_EN;
    expect(items.map((item) => visible(item.textContent))).toEqual([
      message(language, 'assistant-how-open-report', { report: 'R5' }),
      message(language, 'assistant-how-set-party', {
        role: 'buyer',
        party: akram,
        code: '03040001',
      }),
      message(language, 'assistant-how-set-day', { day: '10-10-2026' }),
      message(language, 'assistant-how-press-print'),
    ]);
    // The role is the glossary word, the date is not the ISO one, and the other script is not shown.
    expect(visible(items[1]?.textContent)).toContain(language === 'ur' ? 'گاہک' : 'Buyer');
    expect(list.textContent).not.toContain('2026-10-10');
    expect(list.textContent).not.toContain(language === 'ur' ? AKRAM_EN : AKRAM_UR);
  });

  it('say "seller" for a seller and give both dates for a period', () => {
    renderIn(
      language,
      <HowSteps
        steps={[
          {
            id: 'assistant-how-set-party',
            args: [
              text('role', 'seller'),
              names('party', 'رحیم', 'Rahim'),
              text('code', '030110001'),
            ],
          },
          {
            id: 'assistant-how-set-dates',
            args: [text('from', '2026-10-01'), text('to', '2026-10-31')],
          },
        ]}
      />,
    );
    const items = screen.getAllByRole('listitem').map((item) => visible(item.textContent));
    expect(items[0]).toContain(language === 'ur' ? 'بیوپاری' : 'Seller');
    expect(items[0]).not.toContain(language === 'ur' ? 'گاہک' : 'Buyer');
    expect(items[1]).toBe(
      message(language, 'assistant-how-set-dates', { from: '01-10-2026', to: '31-10-2026' }),
    );
  });

  it('write a record with no code, every role the backend sends, and the row to look at (plan 55 Section 7)', () => {
    const roles = [
      'buyer',
      'seller',
      'partner',
      'garden',
      'trip',
      'khata',
      'staff',
      'city',
      'adda',
      'head',
    ];
    renderIn(
      language,
      <HowSteps
        steps={[
          ...roles.map((role) => ({
            id: 'assistant-how-set-party',
            args: [
              text('role', role),
              names('party', 'سفر 0070', 'Trip 0070'),
              text('code', '0070'),
            ],
          })),
          {
            id: 'assistant-how-set-record',
            args: [text('role', 'partner'), names('name', 'شراکت دار الف', 'Partner A')],
          },
          { id: 'assistant-how-find-row', args: [names('name', 'سفر 0070', 'Trip 0070')] },
        ]}
      />,
    );
    const items = screen.getAllByRole('listitem').map((item) => visible(item.textContent));
    expect(items).toHaveLength(roles.length + 2);
    // Each role has its own word, so no two lines of the same party read alike.
    expect(new Set(items.slice(0, roles.length)).size).toBe(roles.length);
    for (const line of items) {
      expect(line).not.toContain('$');
      expect(line).not.toContain('assistant-');
    }
    const trip = language === 'ur' ? 'سفر 0070' : 'Trip 0070';
    expect(items.at(-1)).toBe(message(language, 'assistant-how-find-row', { name: trip }));
    expect(items.at(-2)).toBe(
      message(language, 'assistant-how-set-record', {
        role: 'partner',
        name: language === 'ur' ? 'شراکت دار الف' : 'Partner A',
      }),
    );
  });

  it('skip a step the build has no words for or one that arrives without a value, and never show an id', () => {
    renderIn(
      language,
      <HowSteps
        steps={[
          { id: 'assistant-how-from-the-future', args: [] },
          { id: 'assistant-how-open-report', args: [] },
          { id: 'assistant-how-set-day', args: [text('day', '2026-10-10')] },
          { id: 'assistant-how-press-print', args: [] },
        ]}
      />,
    );
    const items = screen.getAllByRole('listitem').map((item) => visible(item.textContent));
    expect(items).toEqual([
      message(language, 'assistant-how-set-day', { day: '10-10-2026' }),
      message(language, 'assistant-how-press-print'),
    ]);
    expect(document.body.textContent).not.toContain('assistant-how');
    expect(document.body.textContent).not.toContain('$');
  });

  it('draw nothing when no step can be written', () => {
    renderIn(language, <HowSteps steps={[{ id: 'assistant-how-from-the-future', args: [] }]} />);
    expect(screen.queryByRole('list')).toBeNull();
  });
});

describe.each(LANGUAGES)('the progress lines of a slow question (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    vi.useFakeTimers({ toFake: ['setInterval', 'clearInterval', 'Date'] });
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('say what was understood, that the answer is being written, and the slow-line words', () => {
    const steps: AssistantStep[] = [
      { id: 'assistant-step-understood', args: [text('kind', 'print')] },
      { id: 'assistant-step-writing', args: [] },
      { id: 'assistant-step-retrying', args: [] },
      { id: 'assistant-step-tools-off', args: [] },
    ];
    renderIn(language, <StepLines steps={steps} />);
    const lines = screen.getAllByRole('listitem').map((item) => visible(item.textContent));
    expect(lines).toEqual([
      message(language, 'assistant-understanding'),
      message(language, 'assistant-step-understood', { kind: 'print' }),
      message(language, 'assistant-step-writing'),
      message(language, 'assistant-step-retrying'),
      message(language, 'assistant-step-tools-off'),
    ]);
    // Each kind has its own words, and an unknown kind reads as a figure, never as a hole.
    for (const kind of ['figure', 'list', 'print', 'help', 'advice']) {
      expect(message(language, 'assistant-step-understood', { kind })).not.toContain('$');
    }
    expect(message(language, 'assistant-step-understood', { kind: 'print' })).not.toBe(
      message(language, 'assistant-step-understood', { kind: 'figure' }),
    );
  });

  it('a step that says "understood" without its kind is told in the plain line', () => {
    renderIn(language, <StepLines steps={[{ id: 'assistant-step-understood', args: [] }]} />);
    const lines = screen.getAllByRole('listitem').map((item) => visible(item.textContent));
    expect(lines).toEqual([
      message(language, 'assistant-understanding'),
      message(language, 'assistant-step-generic'),
    ]);
  });

  it('count the seconds waited from the screen’s own clock, once it is worth saying, and keep them out of the live region', () => {
    renderIn(language, <StepLines steps={[]} />);
    expect(screen.getAllByRole('listitem')).toHaveLength(1);
    act(() => {
      vi.advanceTimersByTime(4000);
    });
    expect(screen.getAllByRole('listitem')).toHaveLength(1);
    act(() => {
      vi.advanceTimersByTime(1000);
    });
    const waited = (seconds: number) => message(language, 'assistant-waited', { seconds });
    expect(visible(document.body.textContent)).toContain(waited(5));
    act(() => {
      vi.advanceTimersByTime(7000);
    });
    expect(visible(document.body.textContent)).toContain(waited(12));
    expect(visible(document.body.textContent)).not.toContain(waited(5));
    // The count changes each second, so the screen reader does not read it.
    const line = screen
      .getAllByRole('listitem', { hidden: true })
      .find((item) => visible(item.textContent) === waited(12));
    expect(line).toHaveAttribute('aria-hidden', 'true');
  });
});
