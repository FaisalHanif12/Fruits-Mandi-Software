// All routes (Tech Stack 5.3). Screens load only when opened (Code Standards F36).
// Hash history: a desktop app has no address bar, and the URL never reaches a server.

import {
  Outlet,
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  lazyRouteComponent,
} from '@tanstack/react-router';
import { moveDevPathToHash } from './dev/hash-route';

const rootRoute = createRootRoute({ component: Outlet });

const startRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: lazyRouteComponent(() => import('./start/StartPage'), 'StartPage'),
});

// The component gallery exists only in development (plan 02 T3). In a build, DEV is false, so
// this branch and the gallery's code are removed from the bundle.
const devRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/gallery',
        component: lazyRouteComponent(() => import('./dev/GalleryPage'), 'GalleryPage'),
      }),
    ]
  : [];

const routeTree = rootRoute.addChildren([startRoute, ...devRoutes]);

// In development, /dev/gallery typed without the # moves to /#/dev/gallery before the router
// reads the address. In a build, DEV is false, so this call and the helper are removed.
if (import.meta.env.DEV) moveDevPathToHash();

export const router = createRouter({ routeTree, history: createHashHistory() });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
