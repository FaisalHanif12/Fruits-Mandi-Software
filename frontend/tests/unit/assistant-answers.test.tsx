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
import type { AnswerEstimate } from '../../src/bindings/AnswerEstimate';
import type { AnswerWorking } from '../../src/bindings/AnswerWorking';
import type { AskReply } from '../../src/bindings/AskReply';
import type { EntryScreen } from '../../src/bindings/EntryScreen';
import type { ScreenRoute } from '../../src/bindings/ScreenRoute';
import type { EstimateMethod } from '../../src/bindings/EstimateMethod';
import type { AssistantStep } from '../../src/bindings/AssistantStep';
import * as assistantApi from '../../src/features/assistant/api';
import { AnswerBlock } from '../../src/features/assistant/components/AnswerBlock';
import { Figure } from '../../src/features/assistant/components/Figure';
import { AnswerText } from '../../src/features/assistant/components/AnswerText';
import { useAssistantStore } from '../../src/features/assistant/store';
import { PARTNER_CODE_KEY } from '../../src/features/partners';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { press } from '../helpers/app';
import { paisaForTest } from '../helpers/paisa';
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

/** A money cell, as the backend sends one. */
const money = (paisa: number) => ({ type: 'money' as const, paisa: paisaForTest(paisa) });

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
    // Two closing steps: the books' months added up, then the books plus the months left.
    expect(visible(document.body.textContent)).toContain(
      '36,000 + 39,000 + 42,000 + 45,000 + 48,000 + 51,000 = 261,000',
    );
    // The last step's two parts are named under the steps.
    expect(visible(document.body.textContent)).toContain(
      message(language, 'assistant-working-year-parts'),
    );
  });

  /** The year-end fixture with another `year` (and each method's year total), as asked in another month. */
  function yearEndAs(
    year: AnswerEstimate['year'],
    yearTotal: (method: EstimateMethod) => EstimateMethod['year_total'],
  ): AskReply {
    const base = real.REPLY_ESTIMATE_YEAR_END;
    const answer = base.answer;
    const estimate = answer?.estimate ?? null;
    if (answer === null || estimate === null) throw new Error('the fixture has an estimate');
    return {
      ...base,
      answer: {
        ...answer,
        estimate: {
          ...estimate,
          year,
          methods: estimate.methods.map((method) => ({
            ...method,
            year_total: yearTotal(method),
          })),
        },
      },
    };
  }

  it('asked in January there is no month of the books: the year total is the plain total, with no year lines', async () => {
    await renderShellRouted();
    await ask(
      language,
      yearEndAs({ so_far: money(0), months: 0, from: null, to: null }, () => null),
      'saal ke aakhir tak commission',
    );
    const band = within(panelOf(language)).getByRole('region', {
      name: message(language, 'assistant-band-estimate'),
    });
    const shown = visible(band.textContent);
    expect(shown).not.toContain(message(language, 'assistant-estimate-year-so-far'));
    expect(shown).not.toContain(message(language, 'assistant-estimate-year-total'));
    expect(shown).not.toContain(message(language, 'assistant-estimate-year-left'));
    expect(shown).toContain(message(language, 'assistant-estimate-total'));
    expect(shown).toContain('144,000');
  });

  it('with one month of the books it says that one month, not "1 months"', async () => {
    await renderShellRouted();
    const january = { year: 2026, month: 1 };
    await ask(
      language,
      yearEndAs(
        { so_far: money(3_600_000), months: 1, from: january, to: january },
        (method) => method.year_total,
      ),
      'saal ke aakhir tak commission',
    );
    const shown = visible(
      within(panelOf(language)).getByRole('region', {
        name: message(language, 'assistant-band-estimate'),
      }).textContent,
    );
    expect(shown).toContain(
      message(language, 'assistant-estimate-year-month', {
        from: `${message(language, 'report-month-1')} 2026`,
      }),
    );
    // The plural line ("6 months of the books, from to") is not used for one month.
    expect(shown).not.toMatch(/\b1 months\b/);
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

describe.each(LANGUAGES)('the working of a percent more or less (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it("backend's own answer: this year's commission and 10% more, with its working", async () => {
    await renderShellRouted();
    const user = await ask(
      language,
      real.REPLY_ANSWERED_PERCENT_MORE,
      'agar is saal commission 10 percent zyada hota to kitna hota',
    );
    const answer = within(panelOf(language)).getByLabelText(
      message(language, 'assistant-answer-label'),
    );
    expect([...answer.querySelectorAll('bdi.number-run')].map((node) => node.textContent)).toEqual([
      '5,448',
      '5,993',
    ]);
    // Only the figure the calculator made has the ⓘ; the books' figure is checked by its source.
    const infos = within(answer).getAllByRole('button', {
      name: message(language, 'calc-info-label'),
    });
    expect(infos).toHaveLength(1);
    await user.click(infos[0] ?? answer);
    const shown = visible(document.body.textContent);
    expect(shown).toContain(message(language, 'assistant-working-percent-more'));
    expect(shown).toContain('5,448 × 110.00% = 5,993');
  });

  it.each([
    // 8,360,110 and 10.00% more: the amount times 110.00%.
    [
      'assistant-working-percent-more',
      836_011_000,
      11_000,
      919_612_100,
      '8,360,110 × 110.00% = 9,196,121',
    ],
    // 48,333 and 50.00% less: the amount times 50.00%, rounded once.
    ['assistant-working-percent-less', 4_833_300, 5_000, 2_416_700, '48,333 × 50.00% = 24,167'],
  ])(
    '%s: its own title and the step, with the percent and its sign',
    async (id, base, factor, result, line) => {
      const working: AnswerWorking = {
        title_id: id,
        steps: [
          {
            op: 'times',
            inputs: [money(base), { type: 'percent', hundredths: factor }],
            result: money(result),
          },
        ],
      };
      const view = renderIn(
        language,
        <Figure cell={money(result)} dir={null} checked={false} working={working} />,
      );
      const user = userEvent.setup();
      await user.click(view.getByRole('button', { name: message(language, 'calc-info-label') }));
      const shown = visible(document.body.textContent);
      expect(shown).toContain(message(language, id));
      expect(shown).toContain(line);
    },
  );
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

  // Plan 71 L9 (PRD FR-12.45): two lots of one seller and one item are told apart by the day each came and
  // how many نگ it had, beside the lot's name, which is the option's name.
  it('which one for a lot: the button shows the day it came and the نگ received', async () => {
    await renderShellRouted();
    const lots = real.REPLY_WHICH_ONE.options.map((option, index) => ({
      ...option,
      balance: null,
      date: index === 0 ? '2026-10-10' : '2026-10-12',
      qty: index === 0 ? 200 : 150,
    }));
    await ask(language, { ...real.REPLY_WHICH_ONE, options: lots }, 'ehsan ke 200 peti aam');
    const panel = panelOf(language);
    const buttons = within(panel).getAllByRole('button', { name: /0022|0020/ });
    expect(buttons).toHaveLength(2);
    expect(visible(buttons[0]?.textContent)).toContain('10-10-2026');
    expect(visible(buttons[0]?.textContent)).toContain('200');
    expect(visible(buttons[1]?.textContent)).toContain('12-10-2026');
    expect(visible(buttons[1]?.textContent)).toContain('150');
  });

  // The backend ends a list of lots with one more option, the key `l*`: every lot. Its label is ours, in
  // the language of the app, never the name the backend sends with it; a click picks the key `l*`.
  it('which one for a lot: the last button says "all lots" in the app language, ignoring its name', async () => {
    await renderShellRouted();
    const lots = real.REPLY_WHICH_ONE.options.map((option, index) => ({
      ...option,
      balance: null,
      date: index === 0 ? '2026-10-10' : '2026-10-12',
      qty: index === 0 ? 200 : 150,
    }));
    const base = real.REPLY_WHICH_ONE.options[0];
    if (base === undefined) throw new Error('no option');
    const all = {
      ...base,
      key: 'l*',
      name: { ...base.name, name_ur: 'NAME FROM THE BACKEND', name_en: 'NAME FROM THE BACKEND' },
    };
    const user = await ask(
      language,
      { ...real.REPLY_WHICH_ONE, options: [...lots, all] },
      'ehsan ke aam',
    );
    const panel = panelOf(language);
    const label = message(language, 'assistant-option-all-lots');
    const button = within(panel).getByRole('button', { name: (name) => visible(name) === label });
    expect(button).toBeInTheDocument();
    // It stands last, after the lots, and shows nothing of the backend's name.
    const all2 = within(panel)
      .getAllByRole('button')
      .filter((candidate) => candidate.closest('ul') === button.closest('ul'));
    expect(all2.at(-1)).toBe(button);
    expect(visible(panel.textContent)).not.toContain('NAME FROM THE BACKEND');
    vi.mocked(assistantApi.assistantPick).mockResolvedValue(real.PICK_MATCH);
    await user.click(button);
    await waitFor(() => {
      expect(assistantApi.assistantPick).toHaveBeenCalledWith({
        question_id: real.REPLY_WHICH_ONE.question_id,
        choice: { type: 'match', key: 'l*' },
      });
    });
  });

  it.each(['seller', 'buyer', 'item', 'partner', 'garden', 'trip', 'any'] as const)(
    'not found as a %s: says that no such record has that name, in the role asked, with no list when there are no near names',
    async (role) => {
      await renderShellRouted();
      await ask(
        language,
        { ...real.REPLY_NOT_FOUND, asked_role: role, options: [] },
        'zzzz ka hisab',
      );
      expect(text(language)).toContain(message(language, 'assistant-not-found-role', { role }));
      // The role word is the glossary's: a seller is not called a buyer, and "any" names no role.
      if (role !== 'any') {
        const other = role === 'seller' ? 'buyer' : 'seller';
        expect(text(language)).not.toContain(
          message(language, 'assistant-not-found-role', { role: other }),
        );
      }
      expect(text(language)).not.toContain(message(language, 'assistant-not-found-closest'));
      expect(text(language)).not.toContain(
        message(language, 'assistant-found-as-other', { asked: '', found: '' }),
      );
    },
  );

  it('not found as a name of another role: says what he is, offers his purchases as a pick on the waiting question, and keeps no copy of him in the list', async () => {
    await renderShellRouted();
    const user = await ask(language, real.REPLY_NOT_FOUND_OTHER_ROLE, 'faisal ne aaj kya becha');
    const found = real.REPLY_NOT_FOUND_OTHER_ROLE.found_as;
    if (found === undefined) throw new Error('the fixture names the other role');
    const name = language === 'ur' ? found.name.name_ur : found.name.name_en;
    expect(text(language)).toContain(
      message(language, 'assistant-found-as-other', {
        asked: message(language, 'assistant-role-seller'),
        found: message(language, 'assistant-role-buyer'),
      }),
    );
    // It says what he is instead of "no seller by that name".
    expect(text(language)).not.toContain(
      message(language, 'assistant-not-found-role', { role: 'seller' }),
    );
    // The button is the follow-up question for a buyer: his purchases.
    const question = message(language, 'assistant-found-as-ask', { found: 'buyer', name });
    vi.mocked(assistantApi.assistantAsk).mockClear();
    vi.mocked(assistantApi.assistantPick).mockResolvedValue(real.PICK_MATCH);
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: (label) => visible(label) === question,
      }),
    );
    // The question waits for his choice, so the button goes on with it as a pick (not counted again,
    // PRD FR-7.81) and does not send a new question.
    await waitFor(() => {
      expect(assistantApi.assistantPick).toHaveBeenCalledWith({
        question_id: real.REPLY_NOT_FOUND_OTHER_ROLE.question_id,
        choice: { type: 'match', key: found.key },
      });
    });
    expect(assistantApi.assistantAsk).not.toHaveBeenCalled();
    // The only near name is he himself: no list of near names under the line.
    expect(text(language)).not.toContain(message(language, 'assistant-not-found-closest'));
  });

  it('not found: the words and the three closest names as buttons', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_NOT_FOUND, 'akrm');
    expect(text(language)).toContain(
      message(language, 'assistant-not-found-role', { role: 'any' }),
    );
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

  describe('an analytics source opens each of the eleven views A to K with its filters (plan 55 W2)', () => {
    it.each(['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K'] as const)(
      'view %s: the button opens it, and the print is offered',
      async (view) => {
        const base = real.REPLY_ANSWERED_CHART;
        const source = base.answer?.sources.find((candidate) => candidate.kind === 'analytics');
        if (base.answer === null || source?.kind !== 'analytics') {
          throw new Error('the fixture has no analytics source to build from');
        }
        await renderShellRouted();
        const user = await ask(language, {
          ...base,
          answer: {
            ...base.answer,
            sources: [
              {
                ...source,
                view,
                label_id: `analytics-view-${view.toLowerCase()}-title`,
                filters: { ...source.filters, period: 'this_month', compare: 'off' },
              },
            ],
          },
        });
        const panel = panelOf(language);
        expect(
          within(panel).getByRole('button', {
            name: new RegExp(message(language, 'assistant-print')),
          }),
        ).toBeInTheDocument();
        await user.click(
          within(panel).getByRole('button', {
            name: new RegExp(message(language, 'assistant-open-report')),
          }),
        );
        expect(routeNow().path).toBe('/analytics/$view');
        expect(routeNow().params).toEqual({ view });
        expect(routeNow().search).toMatchObject({ period: 'this_month' });
      },
    );
  });

  describe('a screen as the source of an answer (plan 55 Section 13): it is named, and one button opens it', () => {
    const answerFrom = (route: ScreenRoute): AskReply => {
      const base = real.REPLY_ANSWERED;
      if (base.answer === null) throw new Error('the fixture has no answer');
      return {
        ...base,
        answer: { ...base.answer, sources: [{ kind: 'screen', result_id: 'r9', route }] },
      };
    };

    it.each([
      ['dashboard', 'layout-nav-dashboard', '/', {}],
      ['commission_rules', 'masters-rules-title', '/index/rules', {}],
      ['day_close', 'day-title', '/day', {}],
      ['audit', 'settings-tab-audit', '/settings', { tab: 'audit' }],
      // Plan 59 D331: a partner's capital is read from the partner overview, which is behind the code.
      ['partner_overview', 'partners-title', '/partners', {}],
    ] as const)(
      '%s: named by the screen title, opened by "open the screen"',
      async (route, titleId, path, search) => {
        await renderShellRouted();
        const user = await ask(language, answerFrom(route), 'kya hua');
        // The source line names it, and the one button says it opens the screen, never a report.
        expect(text(language)).toContain(message(language, titleId));
        const panel = panelOf(language);
        expect(
          within(panel).queryByRole('button', {
            name: new RegExp(message(language, 'assistant-open-report')),
          }),
        ).toBeNull();
        // A screen has no print.
        expect(
          within(panel).queryByRole('button', {
            name: new RegExp(message(language, 'assistant-print')),
          }),
        ).toBeNull();
        await user.click(
          within(panel).getByRole('button', { name: message(language, 'assistant-open-screen') }),
        );
        expect(routeNow().path).toBe(path);
        expect(routeNow().search).toEqual(search);
        // The panel stays open over the next screen.
        expect(panelOf(language)).not.toHaveAttribute('inert');
      },
    );

    it.each([
      ['dashboard', real.REPLY_SCREEN_SOURCE, real.REPLY_SCREEN_SOURCE_REQUEST, '/', {}],
      [
        'audit',
        real.REPLY_SCREEN_SOURCE_AUDIT,
        real.REPLY_SCREEN_SOURCE_AUDIT_REQUEST,
        '/settings',
        { tab: 'audit' },
      ],
    ] as const)(
      "the backend's own %s answer: its figure is drawn, and the button opens the screen",
      async (route, reply, request, path, search) => {
        expect(reply.answer?.sources).toEqual([expect.objectContaining({ kind: 'screen', route })]);
        await renderShellRouted();
        const user = await ask(language, reply, request.text);
        const panel = panelOf(language);
        // The answer is drawn like any answer: its figure is a number, and the source line names the screen.
        const answer = within(panel).getByLabelText(message(language, 'assistant-answer-label'));
        expect(answer.querySelectorAll('bdi.number-run').length).toBeGreaterThan(0);
        await user.click(
          within(panel).getByRole('button', { name: message(language, 'assistant-open-screen') }),
        );
        expect(routeNow().path).toBe(path);
        expect(routeNow().search).toEqual(search);
      },
    );

    it('with a report and a screen, each button says which one it opens', async () => {
      const base = answerFrom('audit');
      const report = real.REPLY_ANSWERED.answer?.sources.find((source) => source.kind === 'report');
      if (base.answer === null || report === undefined) throw new Error('no source to build from');
      await renderShellRouted();
      await ask(
        language,
        { ...base, answer: { ...base.answer, sources: [report, ...base.answer.sources] } },
        'kya hua',
      );
      const panel = panelOf(language);
      const screenLabel = message(language, 'settings-tab-audit');
      expect(
        within(panel).getByRole('button', {
          name: named(language, 'assistant-open-screen-of', { source: screenLabel }),
        }),
      ).toBeInTheDocument();
    });
  });

  describe('the entries of a garden, a trip and a بکری (plan 55 Section 11.1)', () => {
    const GARDEN = {
      id: 5,
      code: null,
      name_ur: 'شاہ پور باغ',
      name_en: 'Shahpur Garden',
      tag: null,
    };
    const TRIP = { id: 7, code: null, name_ur: 'سفر 0070', name_en: 'Trip 0070', tag: null };
    const entryButton = () =>
      within(panelOf(language)).getByRole('button', {
        name: new RegExp(language === 'ur' ? 'اندراج' : 'entry screen'),
      });
    const openRoute = async (screen: EntryScreen, party: typeof GARDEN | null) => {
      await renderShellRouted();
      const user = await ask(language, {
        ...real.REPLY_ANSWERED_ROUTE,
        entry_route: { screen, party },
      });
      await user.click(entryButton());
      return routeNow();
    };

    it.each([
      ['garden_installment', 'installment'],
      ['garden_cost', 'cost'],
      ['garden_sale', 'other_sale'],
      ['garden_close', 'close'],
    ] as const)('%s opens the garden on its %s', async (screen, type) => {
      const now = await openRoute(screen, GARDEN);
      expect(now.path).toBe('/gardens/$id');
      expect(now.params).toEqual({ id: '5' });
      expect(now.search).toEqual({ type });
    });

    it.each([
      ['trip_purchase', 'purchase'],
      ['trip_cost', 'cost'],
      ['trip_sale', 'other_sale'],
    ] as const)('%s opens the trip on its %s', async (screen, type) => {
      const now = await openRoute(screen, TRIP);
      expect(now.path).toBe('/trips/$id');
      expect(now.params).toEqual({ id: '7' });
      expect(now.search).toEqual({ type });
    });

    it.each([
      ['garden_installment', '/gardens'],
      ['garden_close', '/gardens'],
      ['trip_purchase', '/trips'],
      ['trip_sale', '/trips'],
    ] as const)(
      '%s with no record opens the list, and the Owner picks one',
      async (screen, path) => {
        const now = await openRoute(screen, null);
        expect(now.path).toBe(path);
        expect(now.search).toEqual({});
      },
    );

    it('a later sale opens the unsold stock, where each lot has its own button', async () => {
      const now = await openRoute('bikri_later_sale', null);
      expect(now.path).toBe('/bikri/stock');
      expect(now.search).toEqual({});
    });

    it.each(['bikri_edit', 'bikri_cancel'] as const)(
      '%s opens the بکری book with its find box (F3) asked for',
      async (screen) => {
        const now = await openRoute(screen, null);
        expect(now.path).toBe('/bikri/book');
        expect(now.search).toEqual({ find: true });
      },
    );
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

  it('out of time (plan 55): says the answer is slow, shows the plain tables that were read and their buttons, no model figure', async () => {
    await renderShellRouted();
    await ask(language, real.REPLY_TIME_LIMIT, 'ikram ka hisab batao');
    expect(text(language)).toContain(message(language, 'assistant-time-limit'));
    expect(text(language)).toContain(message(language, 'assistant-stopped-tables'));
    expect(text(language)).not.toContain(message(language, 'assistant-not-understood'));
    const panel = panelOf(language);
    expect(within(panel).getAllByRole('table')).toHaveLength(1);
    expect(
      panel.querySelector('[aria-label="' + message(language, 'assistant-answer-label') + '"]'),
    ).toBeNull();
    // The one report that was read opens, and it prints, with the filters it was read with.
    expect(
      within(panel).getByRole('button', {
        name: named(language, 'assistant-open-report'),
      }),
    ).toBeInTheDocument();
    expect(
      within(panel).getByRole('button', { name: named(language, 'assistant-print') }),
    ).toBeInTheDocument();
  });

  it('a source with an empty result_id is only a link: both buttons draw and print, and the source line leaves it out (plan 54 S2)', async () => {
    const report = real.REPLY_TIME_LIMIT.answer?.sources.find((source) => source.kind === 'report');
    const read = real.REPLY_TIME_LIMIT.answer?.sources.find(
      (source) => source.kind === 'party_card',
    );
    if (report?.kind !== 'report' || read === undefined || real.REPLY_TIME_LIMIT.answer === null) {
      throw new Error('the fixture has no report source');
    }
    const link = (id: 'R13' | 'R14') => ({ ...report, id, result_id: '' });
    const reply: AskReply = {
      ...real.REPLY_TIME_LIMIT,
      outcome: 'answered',
      answer: {
        ...real.REPLY_TIME_LIMIT.answer,
        parts: [{ type: 'text', text: 'x' }],
        tables: [],
        sources: [link('R13'), link('R14')],
      },
    };
    await renderShellRouted();
    await ask(language, reply);
    const panel = panelOf(language);
    const label = (id: string) => `${id} ${message(language, report.label_id)}`;
    for (const id of ['R13', 'R14']) {
      expect(
        within(panel).getByRole('button', {
          name: named(language, 'assistant-open-report-of', { source: label(id) }),
        }),
      ).toBeInTheDocument();
      expect(
        within(panel).getByRole('button', {
          name: named(language, 'assistant-print-of', { source: label(id) }),
        }),
      ).toBeInTheDocument();
    }
    // Nothing was read, so no "source:" line names them and no read time is claimed.
    const sourceLine = message(language, 'assistant-sources-line', {
      sources: 'SRC',
      time: 'TIME',
    });
    const lead = sourceLine.slice(0, sourceLine.indexOf('SRC')).trim();
    expect(text(language)).not.toContain(lead);
  });

  it('when one source was read and one is a link, the source line names only the one that was read', async () => {
    const answer = real.REPLY_TIME_LIMIT.answer;
    const report = answer?.sources.find((source) => source.kind === 'report');
    const card = answer?.sources.find((source) => source.kind === 'party_card');
    if (answer === null || report?.kind !== 'report' || card === undefined) {
      throw new Error('the fixture has no report source');
    }
    const reply: AskReply = {
      ...real.REPLY_TIME_LIMIT,
      outcome: 'answered',
      answer: {
        ...answer,
        parts: [{ type: 'text', text: 'x' }],
        tables: [],
        sources: [card, { ...report, id: 'R35', result_id: '' }],
      },
    };
    await renderShellRouted();
    await ask(language, reply);
    const line = message(language, 'assistant-sources-line', { sources: 'SRC', time: 'TIME' });
    const lead = line.slice(0, line.indexOf('SRC')).trim();
    const shown = text(language);
    expect(shown).toContain(lead);
    // The party card was read and is named in the line; the R35 report is only a button.
    const afterLead = shown.slice(shown.indexOf(lead));
    expect(afterLead).toContain(message(language, 'assistant-source-party-card'));
    expect(afterLead).not.toContain('R35');
  });

  it('outside the books (plan 55 Section 7): the plain line of what the app does not keep, above the nearest report and its button', async () => {
    const base = real.REPLY_OUTSIDE_BOOKS;
    if (base.answer === null) throw new Error('the fixture has no answer');
    const reply: AskReply = {
      ...base,
      answer: {
        ...base.answer,
        parts: [{ type: 'text', text: message(language, 'assistant-outside-tax') }],
      },
    };
    await renderShellRouted();
    await ask(language, reply, 'gst kitna banta hai');
    const shown = text(language);
    expect(shown).toContain(message(language, 'assistant-outside-books'));
    expect(shown).toContain(message(language, 'assistant-outside-tax'));
    // The line of what the app keeps comes before the nearest report.
    expect(shown.indexOf(message(language, 'assistant-outside-tax'))).toBeLessThan(
      shown.indexOf(message(language, 'assistant-outside-books-nearest')),
    );
    expect(
      within(panelOf(language)).getByRole('button', {
        name: named(language, 'assistant-open-report'),
      }),
    ).toBeInTheDocument();
    // Every one of the six lines the backend can send has words in this language.
    for (const id of ['weather', 'tax', 'bank', 'market-rate', 'prices', 'other']) {
      expect(message(language, `assistant-outside-${id}`)).not.toContain('$');
    }
  });

  it('outside the books, the real backend answer to a tax question: its words are the ones in the app, drawn above the nearest report', async () => {
    const reply = real.REPLY_OUTSIDE_TAX;
    const first = reply.answer?.parts[0];
    expect(reply.outcome).toBe('outside_books');
    if (first?.type !== 'text') throw new Error('the real answer has no plain line');
    // The backend writes the line from the same Fluent file, in the language of the app (Urdu here).
    expect(first.text).toBe(message('ur', 'assistant-outside-tax'));
    await renderShellRouted();
    await ask(language, reply, real.REPLY_OUTSIDE_TAX_REQUEST.text);
    // The line is drawn as the backend sent it (a sentence is never translated), above the nearest report.
    const shown = text(language);
    expect(shown).toContain(first.text);
    expect(shown.indexOf(first.text)).toBeLessThan(
      shown.indexOf(message(language, 'assistant-outside-books-nearest')),
    );
  });

  it('an instant answer (FR-7.90): the sentence is drawn like any answer, with the name, the figure and its ✓, and the buttons', async () => {
    const base = real.REPLY_ANSWERED;
    const name = base.answer?.parts[0];
    const figure = base.answer?.parts[2];
    if (base.answer === null || name?.type !== 'name' || figure?.type !== 'figure') {
      throw new Error('the fixture answer has no name and balance to build from');
    }
    // The backend writes the sentence from the Fluent message and puts the checked name and figure where
    // the arguments were: here the marks are cut out of the real message and replaced by the real parts.
    const sentence = message(language, 'assistant-instant-party', {
      kind: 'balance',
      name: 'NAMEMARK',
      figure: 'FIGUREMARK',
    });
    const parts = sentence
      .split(/(NAMEMARK|FIGUREMARK)/)
      .filter((piece) => piece !== '')
      .map((piece) =>
        piece === 'NAMEMARK'
          ? name
          : piece === 'FIGUREMARK'
            ? figure
            : { type: 'text' as const, text: piece },
      );
    const reply: AskReply = { ...base, answer: { ...base.answer, parts, tables: [] } };
    await renderShellRouted();
    await ask(language, reply, 'akram ka balance');
    const answer = within(panelOf(language)).getByLabelText(
      message(language, 'assistant-answer-label'),
    );
    const shown = visible(answer.textContent);
    expect(shown).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(shown).toContain(message(language, 'glossary-baqaya'));
    // The figure is a number with its side word and the ✓ of a checked figure.
    expect([...answer.querySelectorAll('bdi.number-run')].map((node) => node.textContent)).toEqual([
      '2,880',
    ]);
    expect(answer.querySelectorAll('[data-checked]')).toHaveLength(1);
    expect(shown).toContain(message(language, 'report-side-dr'));
    // Where it was read is a button, as for any answer.
    expect(
      within(panelOf(language)).getByRole('button', {
        name: new RegExp(message(language, 'assistant-open-report')),
      }),
    ).toBeInTheDocument();
  });

  it('an instant answer of two sentences: each "•" line is on its own line, with its own figure', async () => {
    const base = real.REPLY_ANSWERED;
    const money = base.answer?.parts[6];
    if (base.answer === null || money?.type !== 'figure') {
      throw new Error('the fixture answer has no money figure to build from');
    }
    const period = message(language, 'assistant-instant-period', { shape: 'today' });
    const lineOf = (id: string, role: string) =>
      message(language, id, { role, period, figure: 'FIGUREMARK' })
        .split(/(FIGUREMARK)/)
        .filter((piece) => piece !== '')
        .map((piece) => (piece === 'FIGUREMARK' ? money : { type: 'text' as const, text: piece }));
    // The backend joins the sentences with a new line and "• " itself.
    const parts = [
      { type: 'text' as const, text: '• ' },
      ...lineOf('assistant-instant-sale', 'shop'),
      { type: 'text' as const, text: '\n• ' },
      ...lineOf('assistant-instant-commission', 'shop'),
    ];
    const reply: AskReply = { ...base, answer: { ...base.answer, parts, tables: [] } };
    await renderShellRouted();
    await ask(language, reply, 'aaj ki sale aur commission');
    const answer = within(panelOf(language)).getByLabelText(
      message(language, 'assistant-answer-label'),
    );
    // The line break is kept (CSS keeps it on screen: pre-line), so the second "•" starts a new line.
    expect(answer).toHaveClass('whitespace-pre-line');
    expect(answer.textContent).toContain('\n• ');
    expect(answer.textContent.split('\n')).toHaveLength(2);
    expect(answer.querySelectorAll('bdi.number-run')).toHaveLength(2);
  });

  it('out of time, a party card that was read gives his name and his checked balance above the tables (plan 55)', async () => {
    const base = real.REPLY_TIME_LIMIT;
    const answered = real.REPLY_ANSWERED.answer;
    if (base.answer === null || answered === null) throw new Error('a fixture has no answer');
    const parts = answered.parts.filter((part) => part.type === 'name' || part.type === 'figure');
    expect(parts.length).toBeGreaterThan(1);
    const reply: AskReply = { ...base, answer: { ...base.answer, parts } };
    await renderShellRouted();
    await ask(language, reply, 'ikram ka hisab');
    const panel = panelOf(language);
    expect(text(language)).toContain(message(language, 'assistant-time-limit'));
    expect(text(language)).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(within(panel).getAllByRole('table')).toHaveLength(1);
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

  it('a timeout shows the PRD line first (FR-7.81, no answer came), then the reason, with the ask again button', async () => {
    await renderShellRouted();
    const user = await ask(language, {
      ...real.REPLY_ERROR_404,
      failure: { kind: 'timeout', http_status: null },
    });
    const shown = text(language);
    expect(shown).toContain(message(language, 'assistant-error'));
    expect(shown).toContain(message(language, 'assistant-failure-timeout'));
    // The PRD line is the main line and the reason follows it.
    expect(shown.indexOf(message(language, 'assistant-error'))).toBeLessThan(
      shown.indexOf(message(language, 'assistant-failure-timeout')),
    );
    expect(shown).not.toContain(message(language, 'assistant-offline'));
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
    // The PRD line (FR-7.81) and then its own reason.
    expect(text(language)).toContain(message(language, 'assistant-error'));
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

describe.each(LANGUAGES)(
  'how to print this: steps and link-only buttons (plan 54 S2) (%s)',
  (language) => {
    beforeEach(() => {
      setLanguage(language);
    });

    const akram = language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax';
    const howItems = () =>
      within(
        within(panelOf(language)).getByRole('list', {
          name: message(language, 'assistant-how-label'),
        }),
      )
        .getAllByRole('listitem')
        .map((item) => visible(item.textContent));

    it('a range of buyer bills: the how-to words, the numbered steps with the real party and dates, and nothing claimed as read', async () => {
      await renderShellRouted();
      await ask(language, real.REPLY_PRINT_R5_RANGE, 'akram ke bills ka print kaise nikalen');
      const panel = panelOf(language);
      // The words come first, then the steps as a numbered list in the order sent.
      expect(
        within(panel).getByRole('list', { name: message(language, 'assistant-how-label') }).tagName,
      ).toBe('OL');
      expect(howItems()).toEqual([
        message(language, 'assistant-how-open-report', { report: 'R5' }),
        message(language, 'assistant-how-set-party', {
          role: 'buyer',
          party: akram,
          code: '0020',
        }),
        message(language, 'assistant-how-set-dates', { from: '01-09-2026', to: '30-09-2026' }),
        message(language, 'assistant-how-press-print'),
      ]);
      expect(within(panel).queryAllByRole('table')).toHaveLength(0);
      // Nothing was read, so there is no "source:" line and no read time.
      const line = message(language, 'assistant-sources-line', { sources: 'SRC', time: 'TIME' });
      expect(text(language)).not.toContain(line.slice(0, line.indexOf('SRC')).trim());
    });

    it('a range of buyer bills has two buttons, each with its print: the bills and the ledger', async () => {
      await renderShellRouted();
      await ask(language, real.REPLY_PRINT_R5_RANGE, 'akram ke bills ka print kaise nikalen');
      const panel = panelOf(language);
      for (const id of ['R5', 'R13']) {
        const label = `${id} ${message(language, id === 'R5' ? 'report-r5-title' : 'report-r13-title')}`;
        expect(
          within(panel).getByRole('button', {
            name: named(language, 'assistant-open-report-of', { source: label }),
          }),
        ).toBeInTheDocument();
        expect(
          within(panel).getByRole('button', {
            name: named(language, 'assistant-print-of', { source: label }),
          }),
        ).toBeInTheDocument();
      }
    });

    it('a day slip: one day in the step, one button, and it opens the report with the party and the day set', async () => {
      await renderShellRouted();
      const user = await ask(language, real.REPLY_PRINT_R35_DAY, 'akram ki parchi 10 tarikh ki');
      expect(howItems()).toContain(
        message(language, 'assistant-how-set-day', { day: '10-10-2026' }),
      );
      expect(howItems().join(' ')).not.toContain('2026-10-10');
      await user.click(
        within(panelOf(language)).getByRole('button', {
          name: named(language, 'assistant-open-report'),
        }),
      );
      const route = routeNow();
      expect(route.path).toBe('/reports/$id');
      expect(route.params).toEqual({ id: 'R35' });
      expect(route.search).toMatchObject({ party: 20, as_of: '10-10-2026' });
    });

    it('the report button opens R5 with the party and the dates already set, so the Owner only presses F9', async () => {
      await renderShellRouted();
      const user = await ask(
        language,
        real.REPLY_PRINT_R5_RANGE,
        'akram ke bills ka print kaise nikalen',
      );
      await user.click(
        within(panelOf(language)).getByRole('button', {
          name: named(language, 'assistant-open-report-of', {
            source: `R5 ${message(language, 'report-r5-title')}`,
          }),
        }),
      );
      const route = routeNow();
      expect(route.params).toEqual({ id: 'R5' });
      expect(route.search).toMatchObject({ party: 20, from: '01-09-2026', to: '30-09-2026' });
    });

    it('the print button of the second report prints with the same party and dates', async () => {
      vi.mocked(reportsApi.reportsPrintReport).mockResolvedValue({ printed: [], pdf_paths: null });
      await renderShellRouted();
      const user = await ask(
        language,
        real.REPLY_PRINT_R5_RANGE,
        'akram ke bills ka print kaise nikalen',
      );
      await user.click(
        within(panelOf(language)).getByRole('button', {
          name: named(language, 'assistant-print-of', {
            source: `R13 ${message(language, 'report-r13-title')}`,
          }),
        }),
      );
      await waitFor(() => {
        expect(reportsApi.reportsPrintReport).toHaveBeenCalledTimes(1);
      });
      const sent = vi.mocked(reportsApi.reportsPrintReport).mock.calls[0]?.[0];
      expect(sent?.report.id).toBe('R13');
      expect(sent?.report.language).toBe(language);
      expect(sent?.report.filters.party).toEqual({ account_id: 20, person_key: null });
      expect(sent?.report.filters.from).toBe('01-09-2026');
      expect(sent?.report.filters.to).toBe('30-09-2026');
    });

    it('keyboard only: Tab reaches the open button and Enter opens the report', async () => {
      await renderShellRouted();
      const user = await ask(language, real.REPLY_PRINT_R35_DAY, 'akram ki parchi 10 tarikh ki');
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
      expect(routeNow().params).toEqual({ id: 'R35' });
    });

    it('a switch of language rewrites the steps in the other language and keeps the dates', async () => {
      await renderShellRouted();
      await ask(language, real.REPLY_PRINT_R5_RANGE, 'akram ke bills ka print kaise nikalen');
      const other: Language = language === 'ur' ? 'en' : 'ur';
      setLanguage(other);
      const items = within(
        within(panelOf(other)).getByRole('list', {
          name: message(other, 'assistant-how-label'),
        }),
      )
        .getAllByRole('listitem')
        .map((item) => visible(item.textContent));
      expect(items[2]).toBe(
        message(other, 'assistant-how-set-dates', { from: '01-09-2026', to: '30-09-2026' }),
      );
      expect(items[1]).toContain(other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
      expect(items[1]).not.toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    });
  },
);

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
