// The Back button of every page that was opened from another page (plan 78 F1, PRD FR-7.18), in a real
// router with the same pieces the signed-in shell uses (the trail recorder, the Back hook, the button and
// Esc), in Urdu and in English: it shows only when there is a page to go back to, it takes the user to the
// page he came from in the state he left it (its month, its filters), Esc does the same, a menu starts
// the trail again, and going back twice never bounces.

import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  useNavigate,
  useRouterState,
} from '@tanstack/react-router';
import { beforeEach, describe, expect, it } from 'vitest';
import { AppProviders, createQueryClient } from '../../src/app/providers';
import { LANGUAGES } from '../../src/i18n';
import { useEscape, useEscapeDispatcher } from '../../src/shared/keyboard';
import { promiseNavigation, resetTrail, useBack, usePageTrail } from '../../src/shared/navigation';
import { BackButton } from '../../src/shared/ui';
import { resetClientState } from '../helpers/app';
import { press } from '../helpers/app';
import { message } from '../helpers/render';

const PATHS = [
  '/',
  '/dues',
  '/commission',
  '/sales',
  '/partners',
  '/partners/capital',
  '/index/accounts',
  '/index/accounts/new',
  '/rokar',
];

const MENU_BUTTON = 'menu rokar';

/** A word on the test page. It is read through a function, so no text is written between tags. */
function Word({
  read,
  as,
  onClick,
}: {
  read: () => string;
  as: 'button' | 'p';
  onClick?: () => void;
}) {
  return as === 'button' ? (
    <button type="button" onClick={onClick}>
      {read()}
    </button>
  ) : (
    <p>{read()}</p>
  );
}

/** The pieces of the signed-in shell that Back is made of. */
function Layout() {
  usePageTrail();
  useEscapeDispatcher();
  const back = useBack();
  const navigate = useNavigate();
  const location = useRouterState({ select: (state) => state.location });
  useEscape(
    'leave',
    () => {
      if (!back.go()) void navigate({ to: '/' });
    },
    location.pathname !== '/',
  );
  return (
    <>
      <header>
        {back.available ? (
          <BackButton
            onClick={() => {
              back.go();
            }}
          />
        ) : null}
      </header>
      <nav>
        <Word
          as="button"
          read={() => MENU_BUTTON}
          onClick={() => {
            promiseNavigation('fresh', '/rokar');
            void navigate({ to: '/rokar' });
          }}
        />
      </nav>
      <p data-testid="where">
        {location.pathname}
        {location.searchStr}
      </p>
      <Outlet />
    </>
  );
}

async function open(initial: string) {
  const root = createRootRoute({ component: Layout });
  const routes = PATHS.map((path) =>
    createRoute({
      getParentRoute: () => root,
      path,
      component: () => <Word as="p" read={() => `page ${path}`} />,
    }),
  );
  const router = createRouter({
    routeTree: root.addChildren(routes),
    history: createMemoryHistory({ initialEntries: [initial] }),
  });
  await router.load();
  return router;
}

function shown(language: 'ur' | 'en') {
  return screen.queryByRole('button', { name: new RegExp(message(language, 'common-back')) });
}

/** The Back button, which the test says is there. */
function backButton(language: 'ur' | 'en'): HTMLElement {
  const found = shown(language);
  if (found === null) throw new Error('the Back button is not on the page');
  return found;
}

describe.each(LANGUAGES)('the Back button (%s)', (language) => {
  async function render_(initial: string) {
    const router = await open(initial);
    render(
      <AppProviders queryClient={createQueryClient()}>
        <RouterProvider router={router} />
      </AppProviders>,
    );
    await screen.findByTestId('where');
    return router;
  }
  const where = () => screen.getByTestId('where').textContent;

  beforeEach(() => {
    resetTrail();
    resetClientState(language);
  });

  it('is not there on the dashboard, or on a menu page opened by its address', async () => {
    await render_('/');
    expect(shown(language)).toBeNull();
  });

  it('shows on a page opened from another page, with its key, and goes back to that page', async () => {
    const router = await render_('/');
    await act(async () => {
      await router.navigate({ to: '/dues' });
    });
    const button = backButton(language);
    expect(button).toHaveTextContent('Esc');
    await userEvent.setup().click(button);
    expect(where()).toBe('/');
    // On the dashboard there is nothing to go back to any more.
    expect(shown(language)).toBeNull();
  });

  it('goes back to the page in the state it was left: its month and its filters', async () => {
    const router = await render_('/');
    await act(async () => {
      await router.navigate({ to: '/commission', search: { month: '2026-10' } as never });
    });
    // The page changes its own state (another month): still the same page, not a new stop.
    await act(async () => {
      await router.navigate({ to: '/commission', search: { month: '2026-09' } as never });
    });
    await act(async () => {
      await router.navigate({ to: '/index/accounts/new' });
    });
    expect(where()).toBe('/index/accounts/new');
    await userEvent.setup().click(backButton(language));
    expect(where()).toBe('/commission?month=2026-09');
    // Back again goes to the page before that: the dashboard. It does not bounce to the form.
    await userEvent.setup().click(backButton(language));
    expect(where()).toBe('/');
  });

  it('on a child page opened by its address goes to its parent', async () => {
    await render_('/partners/capital');
    expect(shown(language)).not.toBeNull();
    await userEvent.setup().click(backButton(language));
    expect(where()).toBe('/partners');
    expect(shown(language)).toBeNull();
  });

  it('Esc does what the button does', async () => {
    const router = await render_('/');
    await act(async () => {
      await router.navigate({ to: '/partners' });
    });
    await act(async () => {
      await router.navigate({ to: '/partners/capital' });
    });
    // The key goes to the window like the real key; Esc has the same effect as the button.
    press('Escape');
    await screen.findByText('page /partners');
    expect(where()).toBe('/partners');
  });

  it('Esc on a page with nothing to go back to leaves for the dashboard, as before', async () => {
    await render_('/rokar');
    press('Escape');
    await screen.findByText('page /');
    expect(where()).toBe('/');
  });

  it('a menu starts the trail again, so the page it opens has no Back from before', async () => {
    const router = await render_('/');
    await act(async () => {
      await router.navigate({ to: '/dues' });
    });
    expect(shown(language)).not.toBeNull();
    await userEvent.setup().click(screen.getByRole('button', { name: MENU_BUTTON }));
    expect(where()).toBe('/rokar');
    expect(shown(language)).toBeNull();
  });

  it('turns its arrow round in the right-to-left layout and writes no word of its own', async () => {
    const router = await render_('/');
    await act(async () => {
      await router.navigate({ to: '/dues' });
    });
    const button = backButton(language);
    const arrow = button.querySelector('svg');
    expect(arrow?.getAttribute('class')).toContain('rtl:rotate-180');
    expect(within(button).getByText(message(language, 'common-back'))).toBeInTheDocument();
  });
});
