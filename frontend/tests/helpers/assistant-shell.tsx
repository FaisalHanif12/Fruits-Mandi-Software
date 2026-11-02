// The pieces the question panel's tests share: the app shell with a screen that has an unsaved entry
// in it, the panel's two landmarks, and the fake backend (plan 49). The test file says
// `vi.mock('../../src/features/auth/api')` and `vi.mock('../../src/features/assistant/api')`; this file
// sets what those fakes answer.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, render, screen, within, type RenderResult } from '@testing-library/react';
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  useParams,
  useSearch,
} from '@tanstack/react-router';
import { useState, type ReactNode } from 'react';
import { vi } from 'vitest';
import { AppShell } from '../../src/app/layout/AppShell';
import { AppProviders } from '../../src/app/providers';
import * as assistantApi from '../../src/features/assistant/api';
import { useAssistantStore } from '../../src/features/assistant/store';
import * as authApi from '../../src/features/auth/api';
import type { Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import { authState } from './app';
import { NEW_CONVERSATION, STATUS_ON } from './assistant-real';
import { message, visible } from './render';
import { USER_NAME } from './sample';

/** What the fakes answer by default: signed in, questions on, and a question that never ends. */
export function resetAssistantFakes(): void {
  vi.clearAllMocks();
  vi.mocked(authApi.authStatus).mockResolvedValue(authState());
  vi.mocked(assistantApi.assistantStatus).mockResolvedValue(STATUS_ON);
  // A question stays being worked on until the test says its answer has come.
  vi.mocked(assistantApi.assistantAsk).mockReturnValue(new Promise(() => undefined));
  vi.mocked(assistantApi.listenSteps).mockResolvedValue(() => undefined);
  vi.mocked(assistantApi.assistantNewConversation).mockResolvedValue(NEW_CONVERSATION);
  useAssistantStore.getState().reset();
}

export function EscapeKey() {
  useEscapeDispatcher();
  return null;
}

/** A screen with an unsaved entry in it: the panel must never touch it. */
export function Screen({ label }: { label: string }) {
  const [text, setText] = useState('');
  return (
    <form>
      <input
        aria-label={label}
        value={text}
        onChange={(event) => {
          setText(event.target.value);
        }}
      />
    </form>
  );
}

export function newClient(): QueryClient {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}

/** The app shell around some screen, with the question panel in it. */
export function ShellFrame({ client, children }: { client: QueryClient; children: ReactNode }) {
  return (
    <QueryClientProvider client={client}>
      <EscapeKey />
      <AppShell
        nav={<nav />}
        userName={USER_NAME}
        onLock={vi.fn()}
        onSignOut={vi.fn()}
        onGuide={vi.fn()}
        onOpenAccount={vi.fn()}
        onOpenBikri={vi.fn()}
        onOpenVoucher={vi.fn()}
        date="10-10-2026"
        day="open"
      >
        {children}
      </AppShell>
    </QueryClientProvider>
  );
}

export function shell(label = 'entry', client: QueryClient = newClient()) {
  return (
    <ShellFrame client={client}>
      <Screen label={label} />
    </ShellFrame>
  );
}

/** The screens a button of an answer can open, each showing where it is and what its address holds. */
const ROUTES = [
  '/',
  '/reports/$id',
  '/analytics/$view',
  '/bikri/book',
  '/bikri/voucher',
  '/rokar/expense',
  '/settings',
] as const;

function Stub({ path }: { path: string }) {
  const params = useParams({ strict: false });
  const search = useSearch({ strict: false });
  return (
    <div data-testid="route" data-path={path}>
      {JSON.stringify({ path, params, search })}
    </div>
  );
}

/** What the screen under the panel shows of its address, as an object. */
export function routeNow(): { path: string; params: Record<string, string>; search: unknown } {
  const element = screen.getByTestId('route');
  return JSON.parse(element.textContent) as {
    path: string;
    params: Record<string, string>;
    search: unknown;
  };
}

/**
 * The shell inside a real router with the screens above, so a button under an answer can be followed
 * and the panel can be seen to stay. The first screen has the unsaved entry.
 */
export async function renderShellRouted(
  client: QueryClient = newClient(),
  label = 'entry',
): Promise<RenderResult> {
  const rootRoute = createRootRoute({
    component: () => (
      <ShellFrame client={client}>
        <Screen label={label} />
        <Outlet />
      </ShellFrame>
    ),
  });
  const routes = ROUTES.map((path) =>
    createRoute({ getParentRoute: () => rootRoute, path, component: () => <Stub path={path} /> }),
  );
  const router = createRouter({
    routeTree: rootRoute.addChildren(routes),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  });
  await router.load();
  return render(
    <AppProviders queryClient={client}>
      <RouterProvider router={router} />
    </AppProviders>,
  );
}

export function setLanguage(language: Language) {
  act(() => {
    useLanguageStore.setState({ language });
  });
}

export function panelOf(language: Language): HTMLElement {
  return screen.getByRole('region', { name: message(language, 'assistant-dock-label') });
}

export function boxOf(language: Language): HTMLInputElement {
  return within(panelOf(language)).getByRole('textbox', {
    name: message(language, 'assistant-box-label'),
  });
}

/**
 * A name matcher that ignores the isolation marks Fluent puts around a glossary word or a number in a
 * message ("موجود ⁨روکڑ⁩"), for `getByRole('button', { name })`.
 */
export function named(language: Language, id: string, args: Record<string, string | number> = {}) {
  return (name: string) => visible(name) === message(language, id, args);
}
