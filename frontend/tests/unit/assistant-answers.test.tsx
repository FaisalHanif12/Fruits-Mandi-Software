// Every state of the question panel against the backend's own fixtures (plan 49, PRD FR-7.74 to
// FR-7.82, FR-7.86, FR-7.88; Code Standards F8), in Urdu and English: the answer with its figures,
// ✓, ⓘ, names, tables and charts, the estimate and advice bands, the buttons, the source line and the
// next questions; which one, not found, need dates, need party, locked, outside the books, change
// refused, not understood, checks failed, records differ, limit reached, offline and error; the quick
// questions; the step lines; and a keyboard-only run. The panel shows what the backend sent and
// nothing else: no `text` part is ever drawn as a number, and every `figure` part is.

import { QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AskReply } from '../../src/bindings/AskReply';
import type { AssistantStep } from '../../src/bindings/AssistantStep';
import * as assistantApi from '../../src/features/assistant/api';
import { AnswerBlock } from '../../src/features/assistant/components/AnswerBlock';
import { AnswerText } from '../../src/features/assistant/components/AnswerText';
import { useAssistantStore } from '../../src/features/assistant/store';
import { PARTNER_CODE_KEY } from '../../src/features/partners';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { press } from '../helpers/app';
import * as real from '../helpers/assistant-real';
import {
  boxOf,
  named,
  newClient,
  panelOf,
  renderShellRouted,
  resetAssistantFakes,
  routeNow,
  setLanguage,
} from '../helpers/assistant-shell';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/assistant/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/features/reports/api')>()),
  reportsPrintReport: vi.fn(),
}));

beforeEach(() => {
  resetAssistantFakes();
});

afterEach(() => {
  useAssistantStore.getState().reset();
});

const QUESTION = 'ikram ke total bakaya kitne hai';

/** Opens the panel and asks, the way the Owner does, and waits until the answer is on screen. */
async function ask(language: Language, reply: AskReply, text = QUESTION) {
  vi.mocked(assistantApi.assistantAsk).mockResolvedValue(reply);
  const user = userEvent.setup();
  press('F1');
  await user.type(boxOf(language), `${text}{Enter}`);
  await waitFor(() => {
    expect(useAssistantStore.getState().entries.at(-1)?.working).toBe(false);
  });
  return user;
}

const text = (language: Language) => visible(panelOf(language).textContent);

describe.each(LANGUAGES)('an answer (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('writes the name through displayName, the figure large with ✓, and only the checked one has ✓', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_ANSWERED);
    const answer = within(panelOf(language)).getByLabelText(
      message(language, 'assistant-answer-label'),
    );
    // The name in the app language, with its code in brackets, isolated.
    expect(visible(answer.textContent)).toContain(
      language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    );
    const code = answer.querySelector('bdi[data-kind="code"]');
    expect(code).toHaveTextContent('0020');
    // The three figures are numbers (Inter, equal-width digits, isolated); the words are not.
    const figures = [...answer.querySelectorAll('bdi.number-run')].map((node) => node.textContent);
    expect(figures).toEqual(['2,880', '12,880', '10,000']);
    // ✓ only on the balance whose two places agree.
    expect(answer.querySelectorAll('[data-checked]')).toHaveLength(1);
    expect(answer.querySelector('[data-checked]')?.parentElement).toHaveTextContent('2,880');
    // The side word goes with the balance (بنام / Dr).
    expect(visible(answer.textContent)).toContain(message(language, 'report-side-dr'));
  });

  it('draws no `text` part as a number and every `figure` part as one, for every fixture answer', () => {
    const replies = [
      real.REPLY_ANSWERED,
      real.REPLY_ANSWERED_SUM,
      real.REPLY_ANSWERED_CHART,
      real.REPLY_ESTIMATE,
      real.REPLY_ESTIMATE_TARGET,
      real.REPLY_ADVICE,
      real.QUICK_CASH_IN_HAND,
    ];
    for (const reply of replies) {
      const parts = reply.answer?.parts ?? [];
      const { container, unmount } = renderIn(language, <AnswerText parts={parts} />);
      const numbers = [...container.querySelectorAll('bdi.number-run')];
      const figureParts = parts.filter((part) => part.type === 'figure');
      const dateParts = parts.filter((part) => part.type === 'date');
      expect(numbers).toHaveLength(figureParts.length + dateParts.length);
      for (const part of parts) {
        if (part.type !== 'text') continue;
        // The words are written as they are, and no number element holds any of them.
        expect(numbers.some((node) => node.textContent.includes(part.text.trim()))).toBe(false);
        expect(container.textContent).toContain(part.text);
      }
      unmount();
    }
  });

  it('shows the table of the source with its totals row, and a click on a row opens its record', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_ANSWERED);
    const panel = panelOf(language);
    const table = within(panel).getByRole('table');
    expect(within(table).getAllByRole('row')).toHaveLength(1 + 2 + 1);
    // The backend's totals row, labelled, under its columns.
    expect(visible(within(table).getAllByRole('row').at(-1)?.textContent)).toContain(
      message(language, 'report-total'),
    );
    expect(visible(table.textContent)).toContain('12,880');
    await user.click(within(table).getAllByRole('button').at(-1) ?? table);
    expect(routeNow().path).toBe('/bikri/book');
    expect(routeNow().search).toEqual({ id: 1 });
    // The panel stays open over the next screen.
    expect(panelOf(language)).not.toHaveAttribute('inert');
  });

  it('opens the source report with its filters, and says where and when it read', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_ANSWERED);
    const panel = panelOf(language);
    expect(text(language)).toContain(message(language, 'assistant-source-party-card'));
    expect(text(language)).toMatch(/R13/);
    await user.click(
      within(panel).getByRole('button', {
        name: new RegExp(message(language, 'assistant-open-report')),
      }),
    );
    expect(routeNow().path).toBe('/reports/$id');
    expect(routeNow().params).toEqual({ id: 'R13' });
    // The party is in the address with his name, so the report's box shows who he is.
    expect(routeNow().search).toMatchObject({ party: 20 });
  });

  it('prints the source with its own filters, in the app language', async () => {
    vi.mocked(reportsApi.reportsPrintReport).mockResolvedValue({ printed: [], pdf_paths: null });
    await renderShellRouted();
    const user = await ask(language, real.REPLY_ANSWERED);
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: new RegExp(message(language, 'assistant-print')),
      }),
    );
    await waitFor(() => {
      expect(reportsApi.reportsPrintReport).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(reportsApi.reportsPrintReport).mock.calls[0]?.[0];
    expect(sent?.to).toBe('printer');
    expect(sent?.report.id).toBe('R13');
    expect(sent?.report.language).toBe(language);
    expect(sent?.report.filters.party).toEqual({ account_id: 20, person_key: null });
  });

  it('a suggested next question is sent as a new question', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_ANSWERED);
    const next = real.REPLY_ANSWERED.answer?.next[0] ?? '';
    vi.mocked(assistantApi.assistantAsk).mockClear();
    vi.mocked(assistantApi.assistantAsk).mockResolvedValue(real.REPLY_ANSWERED_SUM);
    await user.click(within(panelOf(language)).getByRole('button', { name: next }));
    await waitFor(() => {
      expect(assistantApi.assistantAsk).toHaveBeenCalledWith({
        text: next,
        conversation_id: 'c-1',
      });
    });
  });

  it('the figure of a calculator has ⓘ with the working: the inputs with the sign, then the result', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_ANSWERED_SUM);
    const panel = panelOf(language);
    const answer = within(panel).getByLabelText(message(language, 'assistant-answer-label'));
    // The sum is the first figure; the two parts under it carry no working.
    expect(answer.querySelectorAll('button[title]')).toHaveLength(1);
    await user.click(
      within(answer).getByRole('button', { name: message(language, 'calc-info-label') }),
    );
    const working = await screen.findByText(message(language, 'assistant-working-sum'));
    const box = working.closest('[role="dialog"]');
    expect(box).not.toBeNull();
    expect(visible(box?.textContent)).toContain('2,880');
    expect(visible(box?.textContent)).toContain('18,684');
    expect(visible(box?.textContent)).toContain('21,564');
    expect(visible(box?.textContent)).toContain('+');
    expect(visible(box?.textContent)).toContain('=');
  });

  it('a chart answer draws the chart through the shared wrapper, with a label', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_ANSWERED_CHART);
    const chart = within(panelOf(language)).getByRole('figure');
    expect(chart.querySelector('svg')).not.toBeNull();
  });
});

describe.each(LANGUAGES)('the bands (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('an estimate has the amber band with each method, its months, and no ✓', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_ESTIMATE, 'agar commission isi raftar se chalta raha');
    const panel = panelOf(language);
    const band = within(panel).getByRole('region', {
      name: message(language, 'assistant-band-estimate'),
    });
    expect(visible(band.textContent)).toContain(message(language, 'assistant-band-estimate'));
    expect(visible(band.textContent)).toContain(message(language, 'assistant-estimate-same-pace'));
    expect(visible(band.textContent)).toContain(
      message(language, 'assistant-estimate-same-growth'),
    );
    // The totals are the backend's: 288,000 (same pace) and 369,000 (same growth), never worked out here.
    expect(visible(band.textContent)).toContain('288,000');
    expect(visible(band.textContent)).toContain('369,000');
    // An estimate has no second place: it is never checked.
    expect(band.querySelector('[data-checked]')).toBeNull();
    // Each total has ⓘ with its working.
    expect(
      within(band).getAllByRole('button', { name: message(language, 'calc-info-label') }).length,
    ).toBeGreaterThan(1);
    // And the small chart of the months.
    expect(within(panel).getByRole('figure').querySelector('svg')).not.toBeNull();
  });

  it('a what-if has its own title from the factor, beside same pace, and its working shows the percent', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_ESTIMATE_WHAT_IF, 'agar raftar double ho to');
    const panel = panelOf(language);
    const band = within(panel).getByRole('region', {
      name: message(language, 'assistant-band-estimate'),
    });
    // Same pace is the baseline and the scenario comes right after it, both under the estimate band.
    const shown = visible(band.textContent);
    expect(shown).toContain(message(language, 'assistant-estimate-same-pace'));
    const title = message(language, 'assistant-estimate-what-if', { factor: '200.00%' });
    expect(shown).toContain(title);
    expect(shown.indexOf(title)).toBeGreaterThan(
      shown.indexOf(message(language, 'assistant-estimate-same-pace')),
    );
    // The totals are the backend's: 144,000 (same pace) and 288,000 (the scenario), never worked out here.
    expect(shown).toContain('144,000');
    expect(shown).toContain('288,000');
    // This what-if is to the end of the year too: each method's whole year (261,000 from the books).
    expect(shown).toContain('405,000');
    expect(shown).toContain('549,000');
    expect(band.querySelector('[data-checked]')).toBeNull();
    // The scenario is a percent figure in the answer's words too, drawn as a number.
    expect([...panel.querySelectorAll('bdi.number-run')].map((node) => node.textContent)).toContain(
      '200.00%',
    );
    // Its working: the factor is written as a percent with its sign.
    const infos = within(band).getAllByRole('button', {
      name: message(language, 'calc-info-label'),
    });
    const last = infos.at(-1);
    if (last === undefined) throw new Error('the scenario total has an ⓘ');
    await user.click(last);
    const working = visible(document.body.textContent);
    expect(working).toContain(message(language, 'assistant-working-what-if-pace'));
    expect(working).toContain('48,000 × 200.00% = 96,000');
  });

  it('a what-if is a dashed bar in the chart, named by its factor, beside the other methods', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_ESTIMATE_WHAT_IF, 'agar raftar double ho to');
    const chart = within(panelOf(language)).getByRole('figure');
    const legend = chart.querySelector('[data-chart-legend]');
    expect(legend).not.toBeNull();
    const names = [...(legend?.querySelectorAll('li') ?? [])].map((item) =>
      visible(item.textContent),
    );
    expect(names).toEqual([
      message(language, 'assistant-chart-past'),
      message(language, 'assistant-estimate-same-pace'),
      message(language, 'assistant-chart-what-if', { factor: '200.00%' }),
    ]);
    // Only the scenario has the dashed box in its legend.
    const dashed = legend?.querySelectorAll('li > span.border-dashed') ?? [];
    expect(dashed).toHaveLength(1);
    expect(visible(dashed[0]?.parentElement?.textContent ?? '')).toBe(
      message(language, 'assistant-chart-what-if', { factor: '200.00%' }),
    );
  });

  it('to the end of the year: the books so far, the estimate for the months left, and the whole year with its working', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_ESTIMATE_YEAR_END, 'saal ke aakhir tak commission');
    const band = within(panelOf(language)).getByRole('region', {
      name: message(language, 'assistant-band-estimate'),
    });
    const shown = visible(band.textContent);
    // The books part, from the books, with the months it covers (never estimated).
    expect(shown).toContain(message(language, 'assistant-estimate-year-so-far'));
    expect(shown).toContain('261,000');
    expect(shown).toContain(
      message(language, 'assistant-estimate-year-months', {
        count: 6,
        from: `${message(language, 'report-month-4')} 2026`,
        to: `${message(language, 'report-month-9')} 2026`,
      }),
    );
    // The current month is counted whole: said once.
    expect(shown).toContain(message(language, 'assistant-estimate-year-note'));
    // Each method leads with its whole year (it is what was asked), then the months left, with the
    // backend's figures.
    expect(shown).toContain(message(language, 'assistant-estimate-year-left'));
    expect(shown).toContain(message(language, 'assistant-estimate-year-total'));
    expect(shown.indexOf(message(language, 'assistant-estimate-year-total'))).toBeLessThan(
      shown.indexOf(message(language, 'assistant-estimate-year-left')),
    );
    for (const figure of ['144,000', '405,000', '171,000', '432,000']) {
      expect(shown).toContain(figure);
    }
    expect(band.querySelector('[data-checked]')).toBeNull();
    // The whole year has the working that ends with the books' months plus the months left.
    const infos = within(band).getAllByRole('button', {
      name: message(language, 'calc-info-label'),
    });
    expect(infos).toHaveLength(2);
    const first = infos[0];
    if (first === undefined) throw new Error('the whole year has an ⓘ');
    await user.click(first);
    expect(visible(document.body.textContent)).toContain('261,000 + 144,000 = 405,000');
    // The last step's two parts are named under the steps.
    expect(visible(document.body.textContent)).toContain(
      message(language, 'assistant-working-year-parts'),
    );
  });

  it('a target says the month each method reaches it', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_ESTIMATE_TARGET, '3 lakh ka target');
    const band = within(panelOf(language)).getByRole('region', {
      name: message(language, 'assistant-band-estimate'),
    });
    expect(visible(band.textContent)).toContain('300,000');
    expect(band.querySelectorAll('p').length).toBeGreaterThan(3);
  });

  it('advice has the calm band above the answer', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_ADVICE, 'kis se pehle wasooli karun');
    const panel = panelOf(language);
    const band = within(panel).getByRole('note');
    expect(band).toHaveTextContent(message(language, 'assistant-band-advice'));
    const answer = within(panel).getByLabelText(message(language, 'assistant-answer-label'));
    // The band comes before the answer text.
    expect(band.compareDocumentPosition(answer) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    // The advice answer's table is the R20 rows.
    expect(within(panel).getAllByRole('table')).toHaveLength(1);
    // No button in an advice answer changes anything: only reports and prints open.
    expect(
      within(panel).queryByRole('button', {
        name: new RegExp(message(language, 'assistant-open-entry')),
      }),
    ).toBeNull();
  });
});

describe.each(LANGUAGES)('general advice (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('a general advice answer shows its own band with the note, never the advice band, and no figure', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_GENERAL_ADVICE, 'sale kaise barhayen');
    const panel = panelOf(language);
    const band = panel.querySelector('[data-band="general-advice"]');
    expect(band).not.toBeNull();
    expect(visible(band?.textContent ?? '')).toContain(message(language, 'assistant-band-general'));
    expect(visible(band?.textContent ?? '')).toContain(
      message(language, 'assistant-band-general-note'),
    );
    const shown = text(language);
    expect(shown).not.toContain(message(language, 'assistant-band-advice'));
    // The band stands above the words, and the words come as they are: no figure, table, chart or source.
    expect(shown.indexOf(message(language, 'assistant-band-general'))).toBeLessThan(
      shown.indexOf('•'),
    );
    // The words of the answer carry no number (the count of the day, in the header, is not the answer's).
    const answer = within(panel).getByLabelText(message(language, 'assistant-answer-label'));
    expect(answer.querySelectorAll('bdi.number-run')).toHaveLength(0);
    expect(within(panel).queryAllByRole('table')).toHaveLength(0);
    expect(panel.querySelector('[data-checked]')).toBeNull();
  });

  it('has its own calm band instead of the advice band; advice alone keeps the advice band', () => {
    const answer = real.REPLY_ADVICE.answer;
    if (answer === null) throw new Error('the advice fixture has an answer');
    const general = renderIn(
      language,
      <QueryClientProvider client={newClient()}>
        <AnswerBlock answer={answer} outcome="advice" general onAsk={vi.fn()} busy={false} />
      </QueryClientProvider>,
    );
    expect(general.container.querySelector('[data-band="general-advice"]')).not.toBeNull();
    expect(visible(general.container.textContent)).toContain(
      message(language, 'assistant-band-general'),
    );
    expect(visible(general.container.textContent)).not.toContain(
      message(language, 'assistant-band-advice'),
    );
    general.unmount();
    const advice = renderIn(
      language,
      <QueryClientProvider client={newClient()}>
        <AnswerBlock answer={answer} outcome="advice" onAsk={vi.fn()} busy={false} />
      </QueryClientProvider>,
    );
    expect(advice.container.querySelector('[data-band="general-advice"]')).toBeNull();
    expect(visible(advice.container.textContent)).toContain(
      message(language, 'assistant-band-advice'),
    );
  });
});

describe.each(LANGUAGES)('the states that are not an answer (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('which one: a button for each match with name, code, city and balance; a click goes on without counting', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_WHICH_ONE, 'akram');
    const panel = panelOf(language);
    expect(text(language)).toContain(message(language, 'assistant-which-one'));
    const buttons = within(panel).getAllByRole('button', { name: /0022|0020/ });
    expect(buttons).toHaveLength(2);
    expect(buttons[0]).toHaveTextContent('2,880');
    vi.mocked(assistantApi.assistantPick).mockResolvedValue(real.PICK_MATCH);
    await user.click(buttons[0] ?? panel);
    await waitFor(() => {
      expect(assistantApi.assistantPick).toHaveBeenCalledWith({
        question_id: real.REPLY_WHICH_ONE.question_id,
        choice: { type: 'match', key: 'a20' },
      });
    });
    // The same entry now holds the answer, and the day's count is the backend's.
    await waitFor(() => {
      expect(
        within(panelOf(language)).getByLabelText(message(language, 'assistant-answer-label')),
      ).toBeInTheDocument();
    });
    expect(useAssistantStore.getState().entries).toHaveLength(1);
  });

  it('not found: the words and the three closest names as buttons', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_NOT_FOUND, 'akrm');
    expect(text(language)).toContain(message(language, 'assistant-not-found'));
    expect(text(language)).toContain(message(language, 'assistant-not-found-closest'));
    expect(within(panelOf(language)).getAllByRole('button', { name: /0020|0022/ })).toHaveLength(2);
  });

  it.each([
    ['need dates', real.REPLY_NEED_DATES, 'assistant-need-dates'],
    ['need party', real.REPLY_NEED_PARTY, 'assistant-need-party'],
    ['not understood', real.REPLY_NOT_UNDERSTOOD, 'assistant-not-understood'],
  ])('%s: the words, and the box has the focus again', async (_name, reply, id) => {
    await renderShellRouted();
    await ask(language, reply);
    expect(text(language)).toContain(message(language, id));
    await waitFor(() => {
      expect(boxOf(language)).toHaveFocus();
    });
  });

  it('outside the books: the words and the nearest report', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_OUTSIDE_BOOKS, 'kal lahore mein rate kya hoga');
    expect(text(language)).toContain(message(language, 'assistant-outside-books'));
    expect(
      within(panelOf(language)).getByRole('button', {
        name: new RegExp(message(language, 'assistant-open-report')),
      }),
    ).toBeInTheDocument();
  });

  it('change refused: the words and a button to the entry screen, with no amount', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_CHANGE_REFUSED, 'akram ki 5000 wasooli likh do');
    expect(text(language)).toContain(message(language, 'assistant-change-refused'));
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: new RegExp(language === 'ur' ? 'اندراج' : 'entry screen'),
      }),
    );
    expect(routeNow().path).toBe('/bikri/voucher');
    // The screen is opened on the type with the party chosen, whole, and nothing in the address is an
    // amount (FR-7.79).
    expect(routeNow().search).toEqual({
      type: 'receipt',
      party: {
        id: 20,
        code: '0020',
        name_ur: 'اکرم ٹول ٹیکس',
        name_en: 'Akram Toll Tax',
        tag: null,
      },
    });
  });

  describe('an answer or advice that points to an entry screen (PRD E98, plan 51 F3)', () => {
    // The route the backend sends with an answer; an estimate with it is made up here, because the
    // backend never sends one (it is the rule the screen must keep anyway).
    const route = real.REPLY_ANSWERED_ROUTE.entry_route;
    const entryButton = () =>
      within(panelOf(language)).queryByRole('button', {
        name: new RegExp(language === 'ur' ? 'اندراج' : 'entry screen'),
      });

    it('an answer shows only the neutral line and the button under it, not the refusal title', async () => {
      await renderShellRouted();
      const user = await ask(language, real.REPLY_ANSWERED_ROUTE);
      const shown = text(language);
      // The answer is all there, with its checked figure, and the neutral line comes under it.
      expect(panelOf(language).querySelectorAll('[data-checked]').length).toBeGreaterThan(0);
      expect(shown).toContain(message(language, 'assistant-entry-neutral'));
      expect(shown.indexOf(message(language, 'assistant-entry-neutral'))).toBeGreaterThan(
        shown.indexOf('10,000'),
      );
      // "I can only tell, I cannot write" and its help line are for a refused change only.
      expect(shown).not.toContain(message(language, 'assistant-change-refused'));
      expect(shown).not.toContain(message(language, 'assistant-change-refused-help'));
      const button = entryButton();
      expect(button).not.toBeNull();
      if (button !== null) await user.click(button);
      expect(routeNow().path).toBe('/bikri/voucher');
      expect(routeNow().search).toMatchObject({ type: 'receipt', party: { id: 20 } });
    });

    it('advice shows it under the calm band and the answer', async () => {
      await renderShellRouted();
      const user = await ask(language, real.REPLY_ADVICE_ROUTE);
      const shown = text(language);
      expect(shown).toContain(message(language, 'assistant-band-advice'));
      expect(shown).toContain(message(language, 'assistant-entry-neutral'));
      expect(shown).not.toContain(message(language, 'assistant-change-refused'));
      expect(shown.indexOf(message(language, 'assistant-entry-neutral'))).toBeGreaterThan(
        shown.indexOf(message(language, 'assistant-band-advice')),
      );
      const button = entryButton();
      expect(button).not.toBeNull();
      if (button !== null) await user.click(button);
      expect(routeNow().path).toBe('/bikri/voucher');
    });

    it('an estimate never shows it, even when a route came with it', async () => {
      await renderShellRouted();
      await ask(language, { ...real.REPLY_ESTIMATE, entry_route: route });
      expect(text(language)).not.toContain(message(language, 'assistant-entry-neutral'));
      expect(text(language)).not.toContain(message(language, 'assistant-change-refused'));
      expect(entryButton()).toBeNull();
    });

    it('an answer or advice with no route has no refusal and no button', async () => {
      await renderShellRouted();
      await ask(language, real.REPLY_ADVICE);
      expect(text(language)).not.toContain(message(language, 'assistant-entry-neutral'));
      expect(text(language)).not.toContain(message(language, 'assistant-change-refused'));
      expect(entryButton()).toBeNull();
    });

    it('a refused change still has its title, its help line and the button', async () => {
      await renderShellRouted();
      await ask(language, real.REPLY_CHANGE_REFUSED, 'akram ki 5000 wasooli likh do');
      expect(text(language)).toContain(message(language, 'assistant-change-refused'));
      expect(text(language)).toContain(message(language, 'assistant-change-refused-help'));
      expect(entryButton()).not.toBeNull();
    });
  });

  describe('a help answer that points to a main screen (PRD FR-7.89, plan 52)', () => {
    const screenButton = () =>
      within(panelOf(language)).queryByRole('button', {
        name: message(language, 'assistant-open-screen'),
      });

    it('shows its words and one button that opens the screen as it is: no amount line, no party', async () => {
      await renderShellRouted();
      const user = await ask(language, real.REPLY_HELP_ROUTE, 'market safar kya hai');
      const shown = text(language);
      expect(shown).not.toContain(message(language, 'assistant-entry-neutral'));
      expect(shown).not.toContain(message(language, 'assistant-change-refused'));
      const button = screenButton();
      expect(button).not.toBeNull();
      if (button !== null) await user.click(button);
      expect(routeNow().path).toBe('/trips');
      expect(routeNow().search).toEqual({});
    });

    it('never opens a main screen with a party, even if one came with the route', async () => {
      await renderShellRouted();
      const party = real.REPLY_ANSWERED_ROUTE.entry_route?.party ?? null;
      const user = await ask(language, {
        ...real.REPLY_HELP_ROUTE,
        entry_route: { screen: 'partners', party },
      });
      const button = screenButton();
      expect(button).not.toBeNull();
      if (button !== null) await user.click(button);
      expect(routeNow().path).toBe('/partners');
      expect(routeNow().search).toEqual({});
    });
  });

  describe('a question that a limit stopped (plan 51 B2, PRD E96)', () => {
    it('after the calls: says it could not finish reading and to ask in smaller parts', async () => {
      await renderShellRouted();
      await ask(language, real.REPLY_NOT_UNDERSTOOD_CALLS, 'apni sale ko ziyada kaise karain ?');
      expect(text(language)).toContain(message(language, 'assistant-stopped-calls'));
      expect(text(language)).not.toContain(message(language, 'assistant-not-understood'));
      expect(within(panelOf(language)).queryAllByRole('table')).toHaveLength(0);
      await waitFor(() => {
        expect(boxOf(language)).toHaveFocus();
      });
    });

    it('after the tokens: the other words, and the plain table that was read, with no model figure', async () => {
      await renderShellRouted();
      await ask(
        language,
        real.REPLY_NOT_UNDERSTOOD_TOKENS_TABLE,
        'ikram ke total bakaya kitne hai',
      );
      expect(text(language)).toContain(message(language, 'assistant-stopped-tokens'));
      expect(text(language)).toContain(message(language, 'assistant-stopped-tables'));
      expect(text(language)).not.toContain(message(language, 'assistant-not-understood'));
      expect(within(panelOf(language)).getAllByRole('table')).toHaveLength(1);
      expect(
        panelOf(language).querySelector(
          '[aria-label="' + message(language, 'assistant-answer-label') + '"]',
        ),
      ).toBeNull();
    });
  });

  it('checks failed: the words, the plain tables of the records, and no figure of the model', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_CHECKS_FAILED);
    const panel = panelOf(language);
    expect(text(language)).toContain(message(language, 'assistant-checks-failed'));
    expect(within(panel).getAllByRole('table')).toHaveLength(1);
    expect(
      panel.querySelector('[aria-label="' + message(language, 'assistant-answer-label') + '"]'),
    ).toBeNull();
  });

  it('records differ: the words and both values of the figure with the place of each', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_RECORDS_DIFFER);
    expect(text(language)).toContain(message(language, 'assistant-records-differ'));
    expect(text(language)).toContain(message(language, 'assistant-place-party-card'));
    expect(text(language)).toContain(message(language, 'assistant-place-r13-closing'));
    expect(text(language)).toContain('2,880');
    expect(text(language)).toContain('2,900');
    // A difference is final: nothing in the panel is marked checked.
    expect(panelOf(language).querySelector('[data-checked]')).toBeNull();
  });

  it('limit reached: the words with the 20, and the quick questions still work', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_LIMIT_REACHED);
    expect(text(language)).toContain(message(language, 'assistant-limit-reached', { limit: 20 }));
    // The quick questions are open, not folded behind their button.
    expect(
      within(panelOf(language)).getByRole('list', {
        name: message(language, 'assistant-chips-label'),
      }),
    ).toBeInTheDocument();
  });

  it('offline: the words', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_OFFLINE);
    expect(text(language)).toContain(message(language, 'assistant-offline'));
  });

  it('error: the words, why it failed, the status, and a button that asks again', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_ERROR_404);
    expect(text(language)).toContain(message(language, 'assistant-error'));
    expect(text(language)).toContain(message(language, 'assistant-failure-no-endpoint'));
    expect(text(language)).toContain('(404)');
    vi.mocked(assistantApi.assistantAsk).mockClear();
    vi.mocked(assistantApi.assistantAsk).mockResolvedValue(real.REPLY_ANSWERED);
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: message(language, 'assistant-error-retry'),
      }),
    );
    expect(assistantApi.assistantAsk).toHaveBeenCalledWith(
      expect.objectContaining({ text: QUESTION }),
    );
  });

  it('a timeout says the answer took too long and to ask again, with the ask again button', async () => {
    await renderShellRouted();
    const user = await ask(language, {
      ...real.REPLY_ERROR_404,
      failure: { kind: 'timeout', http_status: null },
    });
    expect(text(language)).toContain(message(language, 'assistant-failure-timeout'));
    // One line says it: the general "no answer came" line would repeat it.
    expect(text(language)).not.toContain(message(language, 'assistant-error'));
    vi.mocked(assistantApi.assistantAsk).mockClear();
    vi.mocked(assistantApi.assistantAsk).mockResolvedValue(real.REPLY_ANSWERED);
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: message(language, 'assistant-error-retry'),
      }),
    );
    expect(assistantApi.assistantAsk).toHaveBeenCalledWith(
      expect.objectContaining({ text: QUESTION }),
    );
  });

  it('a dropped connection is not told as "no internet": its own words and the ask again button', async () => {
    await renderShellRouted();
    await ask(language, {
      ...real.REPLY_ERROR_404,
      failure: { kind: 'dropped', http_status: null },
    });
    expect(text(language)).toContain(message(language, 'assistant-failure-dropped'));
    expect(text(language)).not.toContain(message(language, 'assistant-offline'));
    expect(text(language)).not.toContain(message(language, 'assistant-failure-offline'));
    expect(
      within(panelOf(language)).getByRole('button', {
        name: message(language, 'assistant-error-retry'),
      }),
    ).toBeInTheDocument();
  });

  it('a refusal of the command itself (off) is told in words, with a way to Settings', async () => {
    await renderShellRouted();
    vi.mocked(assistantApi.assistantAsk).mockRejectedValue({
      ok: false,
      errors: [{ code: 'ERR_ASSISTANT_OFF', field: null, values: {} }],
      ref: '',
    });
    const user = userEvent.setup();
    press('F1');
    await user.type(boxOf(language), `${QUESTION}{Enter}`);
    await waitFor(() => {
      expect(text(language)).toContain(message(language, 'err-assistant-off'));
    });
    expect(text(language)).not.toContain('ERR_');
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: message(language, 'assistant-open-settings'),
      }),
    );
    expect(routeNow().path).toBe('/settings');
    expect(routeNow().search).toEqual({ tab: 'assistant' });
  });
});

describe.each(LANGUAGES)('a question waiting for the partner code (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('shows the code box, and after the code the question goes on once, not counted again', async () => {
    const partners = await import('../../src/features/partners/api');
    vi.mocked(partners.partnersCodeStatus).mockResolvedValue({
      is_set: true,
      unlocked: false,
      wait_seconds: 0,
    });
    vi.mocked(assistantApi.assistantPick).mockResolvedValue(real.PICK_PARTNER_CODE);
    const client = newClient();
    await renderShellRouted(client);
    await ask(language, real.REPLY_LOCKED, 'partner ki kiya total hain abhi tak');
    expect(
      within(panelOf(language)).getByRole('button', {
        name: message(language, 'partners-code-enter'),
      }),
    ).toBeInTheDocument();
    expect(assistantApi.assistantPick).not.toHaveBeenCalled();
    // The code is given: the partner section opens.
    act(() => {
      client.setQueryData(PARTNER_CODE_KEY, { is_set: true, unlocked: true, wait_seconds: 0 });
    });
    await waitFor(() => {
      expect(assistantApi.assistantPick).toHaveBeenCalledTimes(1);
    });
    expect(assistantApi.assistantPick).toHaveBeenCalledWith({
      question_id: real.REPLY_LOCKED.question_id,
      choice: { type: 'partner_code' },
    });
  });
});

describe.each(LANGUAGES)('the quick questions (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('answer at once from the chip, with no model, and the chip is named in the conversation', async () => {
    await renderShellRouted();
    vi.mocked(assistantApi.assistantQuick).mockResolvedValue(real.QUICK_CASH_IN_HAND);
    const user = userEvent.setup();
    press('F1');
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: named(language, 'assistant-chip-cash'),
      }),
    );
    await waitFor(() => {
      expect(text(language)).toContain('58,484');
    });
    expect(assistantApi.assistantQuick).toHaveBeenCalledWith({
      chip: 'cash_in_hand',
      party_id: null,
      conversation_id: null,
    });
    expect(assistantApi.assistantAsk).not.toHaveBeenCalled();
    expect(panelOf(language).querySelector('[data-checked]')).not.toBeNull();
  });

  it('are all there, in the order of PRD FR-7.80, even when questions are off', async () => {
    vi.mocked(assistantApi.assistantStatus).mockResolvedValue(real.STATUS_OFF);
    await renderShellRouted();
    press('F1');
    const ids = [
      'assistant-chip-sales',
      'assistant-chip-commission',
      'assistant-chip-cash',
      'assistant-chip-dues',
      'assistant-chip-total-dues',
      'assistant-chip-dead',
      'assistant-chip-payable',
      'assistant-chip-advances',
      'assistant-chip-stock',
      'assistant-chip-party',
    ];
    const list = within(panelOf(language)).getByRole('list', {
      name: message(language, 'assistant-chips-label'),
    });
    expect(
      within(list)
        .getAllByRole('listitem')
        .map((item) => visible(item.textContent)),
    ).toEqual(ids.map((id) => message(language, id)));
  });

  it.each([
    ['short', false],
    ['tall', true],
  ])(
    'in the %s height the chips show in an empty conversation, fold behind their button once a conversation has begun, and the button opens them',
    async (_name, tall) => {
      await renderShellRouted();
      const user = userEvent.setup();
      press('F1');
      act(() => {
        useAssistantStore.getState().setTall(tall);
      });
      const chips = () =>
        within(panelOf(language)).queryByRole('list', {
          name: message(language, 'assistant-chips-label'),
        });
      expect(chips()).not.toBeNull();
      vi.mocked(assistantApi.assistantAsk).mockResolvedValue(real.REPLY_ANSWERED);
      await user.type(boxOf(language), `${QUESTION}{Enter}`);
      await waitFor(() => {
        expect(useAssistantStore.getState().entries.at(-1)?.working).toBe(false);
      });
      expect(chips()).toBeNull();
      const toggle = within(panelOf(language)).getByRole('button', {
        name: message(language, 'assistant-chips-label'),
      });
      expect(toggle).toHaveAttribute('aria-expanded', 'false');
      await user.click(toggle);
      expect(chips()).not.toBeNull();
      expect(toggle).toHaveAttribute('aria-expanded', 'true');
    },
  );

  it('the party chip opens a picker inside the panel, and Never mind brings the chips back', async () => {
    await renderShellRouted();
    const user = userEvent.setup();
    press('F1');
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: named(language, 'assistant-chip-party'),
      }),
    );
    expect(
      within(panelOf(language)).getByText(message(language, 'assistant-party-pick-label')),
    ).toBeInTheDocument();
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: message(language, 'assistant-party-cancel'),
      }),
    );
    expect(
      within(panelOf(language)).getByRole('list', {
        name: message(language, 'assistant-chips-label'),
      }),
    ).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('asking is off or has no key (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('off: the box is replaced by the words and a way to Settings; the chips stay', async () => {
    vi.mocked(assistantApi.assistantStatus).mockResolvedValue(real.STATUS_OFF);
    await renderShellRouted();
    const user = userEvent.setup();
    press('F1');
    await waitFor(() => {
      expect(
        within(panelOf(language)).queryByRole('textbox', {
          name: message(language, 'assistant-box-label'),
        }),
      ).toBeNull();
    });
    expect(text(language)).toContain(
      message(language, real.STATUS_OFF.key_saved ? 'assistant-off' : 'assistant-no-key'),
    );
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: message(language, 'assistant-open-settings'),
      }),
    );
    expect(routeNow().path).toBe('/settings');
  });

  it('no key: says so', async () => {
    vi.mocked(assistantApi.assistantStatus).mockResolvedValue({
      ...real.STATUS_OFF,
      key_saved: false,
      key_last4: null,
    });
    await renderShellRouted();
    press('F1');
    await waitFor(() => {
      expect(text(language)).toContain(message(language, 'assistant-no-key'));
    });
  });
});

describe.each(LANGUAGES)('the steps and the count (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('the first line at once, then each step in the app’s words; an unknown step is the plain line', async () => {
    let step: (value: AssistantStep) => void = () => undefined;
    vi.mocked(assistantApi.listenSteps).mockImplementation((handler) => {
      step = handler;
      return Promise.resolve(() => undefined);
    });
    await renderShellRouted();
    const user = userEvent.setup();
    press('F1');
    await user.type(boxOf(language), `${QUESTION}{Enter}`);
    expect(text(language)).toContain(message(language, 'assistant-understanding'));
    act(() => {
      step(real.STEP_READING_REPORT);
      step({ id: 'assistant-step-from-the-future', args: [] });
    });
    expect(text(language)).toContain(
      message(language, 'assistant-step-reading-report', { report: 'R13' }),
    );
    expect(text(language)).toContain(message(language, 'assistant-step-generic'));
    expect(text(language)).not.toContain('from-the-future');
  });

  it('every step line the backend sends is in the app’s words, with the arguments it really sends', async () => {
    let step: (value: AssistantStep) => void = () => undefined;
    vi.mocked(assistantApi.listenSteps).mockImplementation((handler) => {
      step = handler;
      return Promise.resolve(() => undefined);
    });
    await renderShellRouted();
    const user = userEvent.setup();
    press('F1');
    await user.type(boxOf(language), `${QUESTION}{Enter}`);
    // run.rs `step_of`: a text, or the party's name in both scripts (`names`, with the tag), or a report
    // id, or nothing.
    const named = (name: string, value: string) => ({
      name,
      value: { type: 'text' as const, text: value },
    });
    const bothNames = (name: string, ur: string, en: string) => ({
      name,
      value: { type: 'names' as const, names: { ur, en } },
    });
    const AKRAM_UR = 'اکرم ٹول ٹیکس';
    const AKRAM_EN = 'Akram Toll Tax';
    const AKRAM = language === 'ur' ? AKRAM_UR : AKRAM_EN;
    const steps: [AssistantStep, string][] = [
      [
        { id: 'assistant-step-finding', args: [named('text', 'ikram')] },
        message(language, 'assistant-step-finding', { text: 'ikram' }),
      ],
      [
        {
          id: 'assistant-step-reading-card',
          args: [bothNames('name', AKRAM_UR, AKRAM_EN)],
        },
        message(language, 'assistant-step-reading-card', { name: AKRAM }),
      ],
      [
        {
          id: 'assistant-step-reading-ledger',
          args: [bothNames('name', AKRAM_UR, AKRAM_EN)],
        },
        message(language, 'assistant-step-reading-ledger', { name: AKRAM }),
      ],
      [
        { id: 'assistant-step-reading-report', args: [named('report', 'R20')] },
        message(language, 'assistant-step-reading-report', { report: 'R20' }),
      ],
      ...[
        'assistant-step-reading-analytics',
        'assistant-step-reading-tiles',
        'assistant-step-period',
        'assistant-step-calculating',
        'assistant-step-estimating',
        'assistant-step-checking',
        'assistant-step-rewriting',
        'assistant-step-generic',
      ].map((id): [AssistantStep, string] => [{ id, args: [] }, message(language, id)]),
    ];
    act(() => {
      for (const [one] of steps) step(one);
    });
    // A party that could not be found has no name to write: the plain line stands in, with no hole.
    const before = text(language).split(message(language, 'assistant-step-generic')).length;
    act(() => {
      step({ id: 'assistant-step-reading-card', args: [] });
    });
    expect(text(language).split(message(language, 'assistant-step-generic')).length).toBe(
      before + 1,
    );
    const shown = text(language);
    for (const [one, words] of steps) {
      expect(shown, one.id).toContain(words);
    }
    // No raw id and no unfilled placeholder reaches the screen.
    expect(shown).not.toContain('assistant-step');
    expect(shown).not.toContain('$');
    // The line reads in the app's language: the English one has no Urdu name in it.
    if (language === 'en') expect(shown).not.toContain(AKRAM_UR);
    else expect(shown).not.toContain(AKRAM_EN);
  });

  it('the status bar counts the day: the backend’s number, before and after a question', async () => {
    await renderShellRouted();
    const bar = await screen.findByRole('contentinfo');
    await waitFor(() => {
      expect(visible(bar.textContent)).toContain(
        message(language, 'assistant-button-count', { used: 9, limit: 20 }),
      );
    });
    await ask(language, real.REPLY_ANSWERED);
    expect(visible(bar.textContent)).toContain(
      message(language, 'assistant-button-count', {
        used: real.REPLY_ANSWERED.count_today,
        limit: real.REPLY_ANSWERED.limit,
      }),
    );
  });
});

describe.each(LANGUAGES)('keyboard only and a switch of language (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('F1, type, Enter, Tab to a button, Enter, Esc: the report opens, the panel closes, the focus returns', async () => {
    await renderShellRouted();
    const user = userEvent.setup();
    const entry = screen.getByRole('textbox', { name: 'entry' });
    entry.focus();
    press('F1');
    vi.mocked(assistantApi.assistantAsk).mockResolvedValue(real.REPLY_ANSWERED);
    await user.type(boxOf(language), `${QUESTION}{Enter}`);
    await waitFor(() => {
      expect(useAssistantStore.getState().entries.at(-1)?.working).toBe(false);
    });
    // Tab goes round the panel's own controls until it reaches "open the full report".
    let reached = false;
    for (let step = 0; step < 40 && !reached; step += 1) {
      await user.tab();
      reached = (document.activeElement?.textContent ?? '').includes(
        message(language, 'assistant-open-report'),
      );
    }
    expect(reached).toBe(true);
    await user.keyboard('{Enter}');
    expect(routeNow().path).toBe('/reports/$id');
    await user.keyboard('{Escape}');
    expect(panelOf(language)).toHaveAttribute('inert');
    expect(screen.getByRole('textbox', { name: 'entry' })).toBeInTheDocument();
  });

  it('an answer on screen, an unsaved entry under it, and a switch of language: nothing is lost', async () => {
    const client = newClient();
    await renderShellRouted(client);
    const user = userEvent.setup();
    await user.type(screen.getByRole('textbox', { name: 'entry' }), '20 box at 600');
    await ask(language, real.REPLY_ANSWERED);
    await user.type(boxOf(language), 'half typed');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    setLanguage(other);
    // The same answer in the other language: the name changes, the figures do not.
    const answer = within(panelOf(other)).getByLabelText(message(other, 'assistant-answer-label'));
    expect(visible(answer.textContent)).toContain(
      other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax',
    );
    expect([...answer.querySelectorAll('bdi.number-run')].map((node) => node.textContent)).toEqual([
      '2,880',
      '12,880',
      '10,000',
    ]);
    expect(boxOf(other)).toHaveValue('half typed');
    expect(screen.getByRole('textbox', { name: 'entry' })).toHaveValue('20 box at 600');
  });
});
