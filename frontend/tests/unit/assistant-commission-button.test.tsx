// The commission source of an answer (plan 77, PRD FR-7.74, FR-7.90): the answer to "rahim se is mahine kitna
// commission aur bikri" has one source, the commission of that seller for the month, and its one button opens
// the commission page with the party and the days already in its address. The button is named by the words of
// the page, in both languages; with another source beside it each button says which one it opens. Nothing is
// worked out here: the days and the party are the backend's.

import { waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AnswerSource } from '../../src/bindings/AnswerSource';
import type { AskReply } from '../../src/bindings/AskReply';
import type { DayStatus } from '../../src/bindings/DayStatus';
import * as assistantApi from '../../src/features/assistant/api';
import { useAssistantStore } from '../../src/features/assistant/store';
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
import { message, visible } from '../helpers/render';

vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/assistant/api');

beforeEach(() => {
  resetAssistantFakes();
});

afterEach(() => {
  useAssistantStore.getState().reset();
});

const RAHIM = { id: 19, code: '0010', name_ur: 'رحیم', name_en: 'Rahim', tag: null };

/** The source of plan 77: the commission of seller رحیم for October 2026. */
function commissionSource(from: string, to: string): AnswerSource {
  return {
    kind: 'commission',
    result_id: 'r2',
    side: 'seller',
    party: { record: RAHIM, person_key: null },
    from,
    to,
  };
}

/** The backend's own answer with its sources changed to the given ones. */
function answerWith(...sources: AnswerSource[]): AskReply {
  const base = real.REPLY_SCREEN_SOURCE;
  if (base.answer === null) throw new Error('the fixture has no answer');
  return { ...base, answer: { ...base.answer, sources } };
}

async function ask(language: Language, reply: AskReply) {
  vi.mocked(assistantApi.assistantAsk).mockResolvedValue(reply);
  const user = userEvent.setup();
  press('F1');
  await user.type(boxOf(language), 'rahim se is mahine kitna commission aur bikri{Enter}');
  await waitFor(() => {
    expect(useAssistantStore.getState().entries.at(-1)?.working).toBe(false);
  });
  return user;
}

describe.each(LANGUAGES)('the commission button of an answer (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('is named by the words of the page, and opens that month for that seller', async () => {
    await renderShellRouted();
    const user = await ask(language, answerWith(commissionSource('2026-10-01', '2026-10-31')));
    const panel = panelOf(language);
    expect(visible(panel.textContent)).toContain(message(language, 'assistant-source-commission'));
    // It is a page, not a report: the report words are not on it.
    expect(
      within(panel).queryByRole('button', {
        name: new RegExp(message(language, 'assistant-open-report')),
      }),
    ).toBeNull();
    await user.click(
      within(panel).getByRole('button', {
        name: (name) => visible(name) === message(language, 'assistant-open-commission'),
      }),
    );
    expect(routeNow().path).toBe('/commission');
    expect(routeNow().search).toEqual({
      month: '2026-10',
      side: 'seller',
      party: { id: 19, name_ur: 'رحیم', name_en: 'Rahim', code: '0010' },
    });
    // The panel stays open over the next screen.
    expect(panelOf(language)).not.toHaveAttribute('inert');
  });

  it('opens the days an answer read when they are not a whole month: the month so far is a range', async () => {
    await renderShellRouted();
    const user = await ask(language, answerWith(commissionSource('2026-10-01', '2026-10-10')));
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: (name) => visible(name) === message(language, 'assistant-open-commission'),
      }),
    );
    expect(routeNow().search).toMatchObject({
      period: 'range',
      from: '2026-10-01',
      to: '2026-10-10',
      side: 'seller',
    });
  });

  it('with no dates it opens from the first day with entries to today', async () => {
    const withFirstDay = newClient();
    const status: DayStatus = {
      today: '2026-10-10',
      closed_up_to: null,
      today_open: true,
      first_entry_date: '2026-09-20',
    };
    withFirstDay.setQueryData(['day_status'], status);
    await renderShellRouted(withFirstDay);
    const user = await ask(language, answerWith(commissionSource('', '')));
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: (name) => visible(name) === message(language, 'assistant-open-commission'),
      }),
    );
    expect(routeNow().search).toEqual({
      period: 'range',
      from: '2026-09-20',
      side: 'seller',
      party: { id: 19, name_ur: 'رحیم', name_en: 'Rahim', code: '0010' },
    });
  });

  it('with no dates and the first day not loaded, it opens the current month with the party', async () => {
    await renderShellRouted();
    const user = await ask(language, answerWith(commissionSource('', '')));
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: (name) => visible(name) === message(language, 'assistant-open-commission'),
      }),
    );
    expect(routeNow().search).toEqual({
      side: 'seller',
      party: { id: 19, name_ur: 'رحیم', name_en: 'Rahim', code: '0010' },
    });
  });

  it('with a report as well, each button says which one it opens', async () => {
    const report = real.REPLY_ANSWERED.answer?.sources.find((source) => source.kind === 'report');
    if (report === undefined) throw new Error('no report source to build from');
    await renderShellRouted();
    await ask(language, answerWith(report, commissionSource('2026-10-01', '2026-10-31')));
    expect(
      within(panelOf(language)).getByRole('button', {
        name: named(language, 'assistant-open-screen-of', {
          source: message(language, 'assistant-source-commission'),
        }),
      }),
    ).toBeInTheDocument();
  });
});
