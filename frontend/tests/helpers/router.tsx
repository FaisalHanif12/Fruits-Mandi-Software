// Test helper: render screens inside a real TanStack router with a memory history, so a Link
// on a page works and a click can be followed to the page it opens.

import { render, type RenderResult } from '@testing-library/react';
import {
  Outlet,
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router';
import type { FunctionComponent } from 'react';
import { AppProviders, createQueryClient } from '../../src/app/providers';

/**
 * Renders the app providers and a router that has one page per path. `initial` is the path
 * that is open first. The router has loaded before the first render, so the page is on screen
 * when this returns.
 */
export async function renderRouted(
  pages: Record<string, FunctionComponent>,
  initial: string,
): Promise<RenderResult> {
  const rootRoute = createRootRoute({ component: Outlet });
  const routes = Object.entries(pages).map(([path, component]) =>
    createRoute({ getParentRoute: () => rootRoute, path, component }),
  );
  const router = createRouter({
    routeTree: rootRoute.addChildren(routes),
    history: createMemoryHistory({ initialEntries: [initial] }),
  });
  await router.load();
  return render(
    <AppProviders queryClient={createQueryClient()}>
      <RouterProvider router={router} />
    </AppProviders>,
  );
}
