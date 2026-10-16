// All routes (Tech Stack 5.3). Screens load only when opened (Code Standards F36).
// Hash history: a desktop app has no address bar, and the URL never reaches a server.

import {
  Outlet,
  createHashHistory,
  createRootRoute,
  createRoute,
  createRouter,
  lazyRouteComponent,
  redirect,
  type RouterHistory,
} from '@tanstack/react-router';
import { moveDevPathToHash } from './dev/hash-route';

const rootRoute = createRootRoute({ component: Outlet });

// Everything a signed-in user reaches sits under one layout route: the start-up data check, the
// sign-in screens and the app shell come first (plan 04), and the screens render inside the shell.
const appRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: 'app',
  component: lazyRouteComponent(() => import('./AppFrame'), 'AppFrame'),
});

const dashboardRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/',
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/DashboardPage'),
    'DashboardPage',
  ),
});

const settingsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/settings',
  // `?tab=charges` opens a tab of the settings (the rules screens link to the Charges tab).
  validateSearch: (search: Record<string, unknown>): { tab?: string } =>
    typeof search['tab'] === 'string' ? { tab: search['tab'] } : {},
  component: lazyRouteComponent(
    () => import('../features/settings/pages/SettingsPage'),
    'SettingsPage',
  ),
});

// انڈکس: the menu of the ten master lists, one list, and the form to add or change one (plan 06).
// Static `new` wins over the `$id` parameter in the router.
const indexRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index',
  component: lazyRouteComponent(
    () => import('../features/masters/pages/MastersMenuPage'),
    'MastersMenuPage',
  ),
});

const accountListRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/accounts',
  component: lazyRouteComponent(
    () => import('../features/masters/pages/AccountListPage'),
    'AccountListPage',
  ),
});

const accountNewRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/accounts/new',
  component: lazyRouteComponent(
    () => import('../features/masters/pages/AccountFormPage'),
    'AccountFormPage',
  ),
});

const accountEditRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/accounts/$id',
  component: lazyRouteComponent(
    () => import('../features/masters/pages/AccountFormPage'),
    'AccountFormPage',
  ),
});

const openingsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/openings',
  component: lazyRouteComponent(
    () => import('../features/masters/pages/OpeningsPage'),
    'OpeningsPage',
  ),
});

// بکری: the area opens on the sale book (PRD FR-5.3). `/bikri` goes to it, so the main menu item and
// the side menu item both lead to one screen (plan 12).
const bikriRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri',
  // The router's own way to send the user on: `redirect` is thrown from beforeLoad.
  beforeLoad: () => redirect({ to: '/bikri/book' }),
});

const bikriBookRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/book',
  component: lazyRouteComponent(
    () => import('../features/bikri/pages/BikriBookPage'),
    'BikriBookPage',
  ),
});

const advanceRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/advance',
  component: lazyRouteComponent(
    () => import('../features/vouchers/pages/AdvancePage'),
    'AdvancePage',
  ),
});

const dayRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/day',
  component: lazyRouteComponent(() => import('../features/day/pages/DayPage'), 'DayPage'),
});

const chartRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/chart',
  component: lazyRouteComponent(() => import('../features/masters/pages/ChartPage'), 'ChartPage'),
});

const rulesListRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/rules',
  component: lazyRouteComponent(
    () => import('../features/rules/pages/RulesListPage'),
    'RulesListPage',
  ),
});

const rulesTestRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/rules/test',
  component: lazyRouteComponent(
    () => import('../features/rules/pages/RuleTestPage'),
    'RuleTestPage',
  ),
});

const rulesNewRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/rules/new',
  component: lazyRouteComponent(
    () => import('../features/rules/pages/RuleFormPage'),
    'RuleFormPage',
  ),
});

const rulesEditRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/rules/$id',
  component: lazyRouteComponent(
    () => import('../features/rules/pages/RuleFormPage'),
    'RuleFormPage',
  ),
});

const masterListRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/$kind',
  component: lazyRouteComponent(
    () => import('../features/masters/pages/MasterListPage'),
    'MasterListPage',
  ),
});

const masterNewRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/$kind/new',
  component: lazyRouteComponent(
    () => import('../features/masters/pages/MasterFormPage'),
    'MasterFormPage',
  ),
});

const masterEditRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/index/$kind/$id',
  component: lazyRouteComponent(
    () => import('../features/masters/pages/MasterFormPage'),
    'MasterFormPage',
  ),
});

// شراکت دار: the overview of the partners and their shares, and the setup page (plan 11).
const partnersRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/partners',
  component: lazyRouteComponent(
    () => import('../features/partners/pages/PartnersOverviewPage'),
    'PartnersOverviewPage',
  ),
});

const partnersSetupRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/partners/setup',
  component: lazyRouteComponent(
    () => import('../features/partners/pages/PartnerSetupPage'),
    'PartnerSetupPage',
  ),
});

/** A menu item whose module comes in a later plan opens the "coming soon" page (PRD FR-7.46). */
function comingSoonRoute<Path extends string>(path: Path) {
  return createRoute({
    getParentRoute: () => appRoute,
    path,
    component: lazyRouteComponent(
      () => import('../features/dashboard/pages/ComingSoonPage'),
      'ComingSoonPage',
    ),
  });
}

const appRoutes = [
  dashboardRoute,
  settingsRoute,
  indexRoute,
  accountListRoute,
  accountNewRoute,
  accountEditRoute,
  openingsRoute,
  chartRoute,
  rulesListRoute,
  rulesTestRoute,
  rulesNewRoute,
  rulesEditRoute,
  masterListRoute,
  masterNewRoute,
  masterEditRoute,
  bikriRoute,
  bikriBookRoute,
  comingSoonRoute('/bikri/receipt'),
  comingSoonRoute('/bikri/payment'),
  advanceRoute,
  dayRoute,
  comingSoonRoute('/bikri/pukhta'),
  comingSoonRoute('/bikri/today'),
  comingSoonRoute('/bikri/stock'),
  comingSoonRoute('/rokar'),
  partnersRoute,
  partnersSetupRoute,
  comingSoonRoute('/analytics'),
  comingSoonRoute('/reports'),
  comingSoonRoute('/backup'),
];

// The component gallery exists only in development (plan 02 T3). In a build, DEV is false, so
// this branch and the gallery's code are removed from the bundle.
const devRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/gallery',
        component: lazyRouteComponent(() => import('./dev/GalleryPage'), 'GalleryPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/shell',
        component: lazyRouteComponent(() => import('./dev/ShellPage'), 'ShellPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/grid',
        component: lazyRouteComponent(() => import('./dev/GridPage'), 'GridPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/bikri',
        component: lazyRouteComponent(() => import('./dev/BikriPage'), 'BikriPage'),
      }),
    ]
  : [];

const routeTree = rootRoute.addChildren([appRoute.addChildren(appRoutes), ...devRoutes]);

// In development, /dev/gallery typed without the # moves to /#/dev/gallery before the router
// reads the address. In a build, DEV is false, so this call and the helper are removed.
if (import.meta.env.DEV) moveDevPathToHash();

/** The app's router. Tests give it a memory history; the app uses the hash (no address bar). */
export function createAppRouter(history: RouterHistory = createHashHistory()) {
  return createRouter({ routeTree, history });
}

export const router = createAppRouter();

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
