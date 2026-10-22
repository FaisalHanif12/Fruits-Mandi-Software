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
import { parseReportSearch } from '../features/reports/search';
import { parseVoucherSearch, type ScreenType } from '../features/vouchers/screen-type';
import { moveDevPathToHash } from './dev/hash-route';

const rootRoute = createRootRoute({ component: Outlet });

// The hidden print window (plans 14 and 15): `#/print?bikri=...&doc=...&language=...&job=...` for a
// بکری, `#/print?voucher=...&doc=receipt_slip&language=...&job=...` for a voucher. It sits outside
// the app shell on purpose: no sign-in screen, no menus, only the document. The backend opens it,
// waits for its `print-ready-<job>` event and prints it.
const printRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/print',
  component: lazyRouteComponent(() => import('./PrintRoute'), 'PrintRoute'),
});

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

/** A positive whole number from the address, or nothing. */
function positiveId(value: unknown): number | undefined {
  const id = Number(value);
  return Number.isSafeInteger(id) && id > 0 ? id : undefined;
}

// `/bikri/book` is a new بکری; `/bikri/book?id=12` opens a saved one read-only (plan 13);
// `/bikri/book?id=12&edit=57` opens posting 57 of it for edit (plan 16).
const bikriBookRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/book',
  validateSearch: (search: Record<string, unknown>): { id?: number; edit?: number } => {
    const id = positiveId(search['id']);
    const edit = positiveId(search['edit']);
    return {
      ...(id === undefined ? {} : { id }),
      ...(id !== undefined && edit !== undefined ? { edit } : {}),
    };
  },
  component: lazyRouteComponent(
    () => import('../features/bikri/pages/BikriBookPage'),
    'BikriBookPage',
  ),
});

// آج کی بکری: the list of بکری (PRD FR-12.7). The پختہ کھاتہ register (PRD FR-13.6).
const todayRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/today',
  component: lazyRouteComponent(
    () => import('../features/bikri/pages/TodayBikriPage'),
    'TodayBikriPage',
  ),
});

// باقی مال: the lots with goods left (PRD FR-12.14). A later sale of one lot (PRD FR-12.17) opens
// from it: `/bikri/later?bikri=12&lot=40`.
const stockRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/stock',
  component: lazyRouteComponent(() => import('../features/bikri/pages/StockPage'), 'StockPage'),
});

// مارکیٹ سفر (PRD 12.10, FR-5.2): the trips list, the form of a new trip (shares first, FR-12.29),
// and one trip with its type list (FR-7.67). `/trips/12?purchase=7` shows purchase 7 of trip 12
// in the card under its list. Static `new` wins over the `$id` parameter in the router.
const tripsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/trips',
  component: lazyRouteComponent(() => import('../features/trips/pages/TripsPage'), 'TripsPage'),
});

const tripNewRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/trips/new',
  component: lazyRouteComponent(() => import('../features/trips/pages/NewTripPage'), 'NewTripPage'),
});

const tripRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/trips/$id',
  validateSearch: (search: Record<string, unknown>): { purchase?: number } => {
    const purchase = positiveId(search['purchase']);
    return purchase === undefined ? {} : { purchase };
  },
  component: lazyRouteComponent(() => import('../features/trips/pages/TripPage'), 'TripPage'),
});

const laterSaleRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/later',
  validateSearch: (search: Record<string, unknown>): { bikri?: number; lot?: number } => {
    const bikri = positiveId(search['bikri']);
    const lot = positiveId(search['lot']);
    return {
      ...(bikri === undefined ? {} : { bikri }),
      ...(lot === undefined ? {} : { lot }),
    };
  },
  component: lazyRouteComponent(
    () => import('../features/bikri/pages/LaterSalePage'),
    'LaterSalePage',
  ),
});

const pukhtaRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/pukhta',
  component: lazyRouteComponent(
    () => import('../features/pukhta_khata/pages/PukhtaRegisterPage'),
    'PukhtaRegisterPage',
  ),
});

// وصولی, ادائیگی and پیشگی are one voucher screen with a type list at the top (PRD FR-7.67,
// FR-15.10). The three side menu items and Alt+V, Alt+A and Alt+P keep their own addresses, which
// send the user to that one screen with the type chosen; the screen itself follows `?type=`.
// `?id=12` opens a saved voucher (edit while its day is open, else read only).
const voucherRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/voucher',
  validateSearch: parseVoucherSearch,
  component: lazyRouteComponent(
    () => import('../features/vouchers/pages/VoucherPage'),
    'VoucherPage',
  ),
});

/** An address of a menu item that opens the voucher screen on a type. */
function voucherTypeRoute<Path extends string>(path: Path, type: ScreenType) {
  return createRoute({
    getParentRoute: () => appRoute,
    path,
    beforeLoad: () => redirect({ to: '/bikri/voucher', search: { type }, replace: true }),
  });
}
const receiptRoute = voucherTypeRoute('/bikri/receipt', 'receipt');
const paymentRoute = voucherTypeRoute('/bikri/payment', 'payment');
const advanceRoute = voucherTypeRoute('/bikri/advance', 'advance');

// اگراہی فہرست: today's planned collections and the collection round (PRD FR-15.6, FR-7.61).
const collectionRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/collection',
  component: lazyRouteComponent(
    () => import('../features/vouchers/pages/CollectionPage'),
    'CollectionPage',
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

// ---- Run B, plan 18 (D5 early track): بیک اپ. Replaces the "coming soon" page of /backup. ----
const backupRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/backup',
  component: lazyRouteComponent(() => import('../features/backup/pages/BackupPage'), 'BackupPage'),
});

// ---- Run B, plan 20 (D5 early track): رپورٹ. Replaces the "coming soon" page of /reports. The
// catalogue is /reports; one report is /reports/R18, with its filters optionally in the address
// (`?party=20&from=01-10-2026`, PRD FR-7.65). ----
const reportsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/reports',
  component: lazyRouteComponent(
    () => import('../features/reports/pages/ReportsHomePage'),
    'ReportsHomePage',
  ),
});

const reportRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/reports/$id',
  validateSearch: parseReportSearch,
  component: lazyRouteComponent(() => import('../features/reports/pages/ReportPage'), 'ReportPage'),
});
// ---- end of Run B block ----

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
  voucherRoute,
  receiptRoute,
  paymentRoute,
  advanceRoute,
  collectionRoute,
  dayRoute,
  pukhtaRoute,
  todayRoute,
  stockRoute,
  laterSaleRoute,
  comingSoonRoute('/rokar'),
  tripsRoute,
  tripNewRoute,
  tripRoute,
  partnersRoute,
  partnersSetupRoute,
  comingSoonRoute('/analytics'),
  backupRoute,
  reportsRoute,
  reportRoute,
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
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/trips',
        component: lazyRouteComponent(() => import('./dev/TripsPage'), 'TripsPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/vouchers',
        component: lazyRouteComponent(() => import('./dev/VouchersPage'), 'VouchersPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/print',
        component: lazyRouteComponent(() => import('./dev/PrintPage'), 'PrintPage'),
      }),
    ]
  : [];

// ---- Run B, plan 18: the development page of the backup screen (dev only, like the others). ----
const backupDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/backup',
        component: lazyRouteComponent(() => import('./dev/BackupPage'), 'BackupPage'),
      }),
    ]
  : [];
// ---- end of Run B block ----

// ---- Run B, plan 20: the development pages of the report screens (dev only, like the others). ----
const reportsDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/reports',
        component: lazyRouteComponent(() => import('./dev/ReportsPage'), 'ReportsPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/reports/$id',
        component: lazyRouteComponent(() => import('./dev/ReportsPage'), 'ReportsPage'),
      }),
    ]
  : [];
// ---- end of Run B block ----

const routeTree = rootRoute.addChildren([
  appRoute.addChildren(appRoutes),
  printRoute,
  ...devRoutes,
  ...backupDevRoutes,
  ...reportsDevRoutes,
]);

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
