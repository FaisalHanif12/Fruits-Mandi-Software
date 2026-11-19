// The partner overview as the source of an answer (plan 59 D331, PRD FR-7.74, FR-17.11): a partner's
// capital is read from the partner overview, so the answer's one button opens that screen. The screen
// is behind the partner code like any other way in: while the section is locked the button lands on
// the code box, and no figure of any partner is asked for or shown. With several sources each button
// says which one it opens, and the overview button is named in both languages.

import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
} from '@tanstack/react-router';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AskReply } from '../../src/bindings/AskReply';
import * as assistantApi from '../../src/features/assistant/api';
import { useAssistantStore } from '../../src/features/assistant/store';
import * as partnersApi from '../../src/features/partners/api';
import { PartnersOverviewPage } from '../../src/features/partners/pages/PartnersOverviewPage';
import { AppProviders } from '../../src/app/providers';
import { LANGUAGES, type Language } from '../../src/i18n';
import { press } from '../helpers/app';
import { paisaForTest } from '../helpers/paisa';
import * as real from '../helpers/assistant-real';
import {
  ShellFrame,
  boxOf,
  named,
  newClient,
  panelOf,
  renderShellRouted,
  resetAssistantFakes,
  routeNow,
  setLanguage,
} from '../helpers/assistant-shell';
import { PARTNER_CODE_STATUS_LOCKED } from '../helpers/edit-real';
import { message, visible } from '../helpers/render';

vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/assistant/api');
vi.mock('../../src/features/partners/api');

beforeEach(() => {
  resetAssistantFakes();
  vi.mocked(partnersApi.partnersCodeStatus).mockResolvedValue(PARTNER_CODE_STATUS_LOCKED);
});

afterEach(() => {
  useAssistantStore.getState().reset();
});

/** The backend's own answer with its source changed to the partner overview. */
function overviewAnswer(): AskReply {
  const base = real.REPLY_SCREEN_SOURCE;
  if (base.answer === null) throw new Error('the fixture has no answer');
  return {
    ...base,
    answer: {
      ...base.answer,
      sources: [{ kind: 'screen', result_id: 'r9', route: 'partner_overview' }],
    },
  };
}

async function ask(language: Language, reply: AskReply) {
  vi.mocked(assistantApi.assistantAsk).mockResolvedValue(reply);
  const user = userEvent.setup();
  press('F1');
  await user.type(boxOf(language), 'usman ka sarmaya{Enter}');
  await waitFor(() => {
    expect(useAssistantStore.getState().entries.at(-1)?.working).toBe(false);
  });
  return user;
}

describe.each(LANGUAGES)('the partner overview button of an answer (%s)', (language) => {
  beforeEach(() => {
    setLanguage(language);
  });

  it('is named by the words of the place, and one button opens the screen, never a report', async () => {
    await renderShellRouted();
    const user = await ask(language, overviewAnswer());
    const panel = panelOf(language);
    expect(visible(panel.textContent)).toContain(message(language, 'partners-title'));
    expect(
      within(panel).queryByRole('button', {
        name: new RegExp(message(language, 'assistant-open-report')),
      }),
    ).toBeNull();
    await user.click(
      within(panel).getByRole('button', { name: message(language, 'assistant-open-screen') }),
    );
    expect(routeNow().path).toBe('/partners');
    expect(routeNow().search).toEqual({});
    // The panel stays open over the next screen.
    expect(panelOf(language)).not.toHaveAttribute('inert');
  });

  it('with a report as well, each button says which one it opens', async () => {
    const base = overviewAnswer();
    const report = real.REPLY_ANSWERED.answer?.sources.find((source) => source.kind === 'report');
    if (base.answer === null || report === undefined) throw new Error('no source to build from');
    await renderShellRouted();
    await ask(language, {
      ...base,
      answer: { ...base.answer, sources: [report, ...base.answer.sources] },
    });
    expect(
      within(panelOf(language)).getByRole('button', {
        name: named(language, 'assistant-open-screen-of', {
          source: message(language, 'partners-title'),
        }),
      }),
    ).toBeInTheDocument();
  });

  it("a capital that differs is called the partner's capital, with its two places named by their own words", async () => {
    // The backend's own records-differ answer, with the figure and the two places of a capital (D331).
    const base = real.REPLY_RECORDS_DIFFER;
    if (base.answer === null) throw new Error('the fixture has no answer');
    const cell = (paisa: number) => ({ type: 'money' as const, paisa: paisaForTest(paisa) });
    await renderShellRouted();
    await ask(language, {
      ...base,
      answer: {
        ...base.answer,
        differences: [
          {
            figure_id: 'assistant-figure-partner-capital',
            name: null,
            places: [
              { place_id: 'assistant-place-capital-history', cell: cell(5_000_000) },
              { place_id: 'assistant-place-partner-overview', cell: cell(4_900_000) },
            ],
          },
        ],
      },
    });
    const shown = visible(panelOf(language).textContent);
    for (const id of [
      'assistant-figure-partner-capital',
      'assistant-place-capital-history',
      'assistant-place-partner-overview',
    ]) {
      expect(shown, id).toContain(message(language, id));
    }
    // It is not called his total, and nothing is marked checked.
    expect(shown).not.toContain(message(language, 'assistant-figure-partner-total'));
    expect(panelOf(language).querySelector('[data-checked]')).toBeNull();
  });

  it('locked: the button lands on the code box, and no partner figure is asked for', async () => {
    // The real overview page stands in the place of its route, so its own door is what the Owner meets.
    const client = newClient();
    const rootRoute = createRootRoute({
      component: () => (
        <ShellFrame client={client}>
          <Outlet />
        </ShellFrame>
      ),
    });
    const home = createRoute({
      getParentRoute: () => rootRoute,
      path: '/',
      component: () => <div data-testid="home" />,
    });
    const partners = createRoute({
      getParentRoute: () => rootRoute,
      path: '/partners',
      component: PartnersOverviewPage,
    });
    const router = createRouter({
      routeTree: rootRoute.addChildren([home, partners]),
      history: createMemoryHistory({ initialEntries: ['/'] }),
    });
    await router.load();
    render(
      <AppProviders queryClient={client}>
        <RouterProvider router={router} />
      </AppProviders>,
    );
    const user = await ask(language, overviewAnswer());
    await user.click(
      within(panelOf(language)).getByRole('button', {
        name: message(language, 'assistant-open-screen'),
      }),
    );
    // The section asks for the code, in the page, and the answer panel is still there.
    expect(
      await screen.findByText(
        (content) => visible(content) === message(language, 'partners-code-locked-title'),
      ),
    ).toBeInTheDocument();
    expect(partnersApi.partnersOverview).not.toHaveBeenCalled();
    expect(panelOf(language)).not.toHaveAttribute('inert');
  });
});
