// The guard around every screen (plan 38 G8, Code Standards F8): a screen that cannot draw what it
// was given shows the designed error state with a retry, in the app language, never the framework's
// red error page. Urdu and English.

import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { AppProviders, createQueryClient } from '../../src/app/providers';
import { RouteError } from '../../src/app/RouteError';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { message } from '../helpers/render';

/** The answer of the backend, or null while it is the wrong shape (the screen cannot read it). */
let answer: { rows: readonly string[] } | null = null;

function Broken() {
  if (answer === null) throw new Error('the answer has no rows');
  return <p>{answer.rows.length}</p>;
}

function open(): ReturnType<typeof render> {
  const root = createRootRoute();
  const index = createRoute({ getParentRoute: () => root, path: '/', component: Broken });
  const router = createRouter({
    routeTree: root.addChildren([index]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
    defaultErrorComponent: RouteError,
  });
  return render(
    <AppProviders queryClient={createQueryClient()}>
      <RouterProvider router={router} />
    </AppProviders>,
  );
}

describe.each(LANGUAGES)('a screen that cannot draw its answer (%s)', (language: Language) => {
  beforeEach(() => {
    answer = null;
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  it('shows the error state with a retry, and the screen draws again after the answer is right', async () => {
    const user = userEvent.setup();
    open();
    expect(
      await screen.findByRole('heading', { name: message(language, 'common-screen-failed') }),
    ).toBeInTheDocument();
    answer = { rows: ['a', 'b'] };
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(await screen.findByText('2')).toBeInTheDocument();
  });
});
