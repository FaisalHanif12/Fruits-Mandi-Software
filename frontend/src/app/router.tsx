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
import { parseAnalyticsSearch } from '../features/analytics/search';
import { parseCommissionSearch } from '../features/dashboard/commission-search';
import { parseKharchSearch } from '../features/dashboard/kharch-search';
import { parseSalesSearch } from '../features/dashboard/sales-search';
import { parseSellerDuesSearch } from '../features/dashboard/seller-dues-search';
import { parseReportSearch } from '../features/reports/search';
import { parseMonthSearch } from '../features/month';
import { parseLedgerSearch } from '../features/party_ledgers';
import {
  parseExpenseSearch,
  parseKhataSearch,
  parseLentSearch,
} from '../features/cashbook/screen-type';
import { parseFruitSearch } from '../features/fruit/screen-search';
import { parsePastReturn, parsePastSearch, type PastReturn } from '../features/bikri/past-window';
import { parseGardenSearch } from '../features/gardens/screen-type';
import { isIsoDay } from '../shared/format';
import { parseTripSearch } from '../features/trips/screen-type';
import { parseGeneralSearch } from '../features/cashbook/screen-type';
import { parseCapitalSearch, parseStatementSearch } from '../features/partners/screen-type';
import { parseVoucherSearch, type ScreenType } from '../features/vouchers/screen-type';
import { AddRecordHost } from './AddRecordHost';
import { DeadAmountHost } from './DeadAmountHost';
import { moveDevPathToHash } from './dev/hash-route';
import { RouteError } from './RouteError';

// The root holds the Index form that the "add" button of any picker opens (plan 32, R8) and the two
// dialogs of the dead amount (plan 40).
function RootLayout() {
  return (
    <AddRecordHost>
      <DeadAmountHost>
        <Outlet />
      </DeadAmountHost>
    </AddRecordHost>
  );
}

const rootRoute = createRootRoute({ component: RootLayout });

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

// انڈکس: the menu of the nine master lists, one list, and the form to add or change one (plan 06).
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
// `/bikri/book?id=12&edit=57` opens posting 57 of it for edit (plan 16); `/bikri/book?find=true` is a new
// بکری with the find box (F3) already open, the way an answer sends the Owner to a بکری to edit or cancel
// (plan 55 Section 11.1). `past=2026-11-11` (and `pastAll=true`) say the Owner came from پچھلی بکری with
// the window ending that day: after an edit is saved he goes back to the same window and row (plan 72
// P10).
const bikriBookRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/book',
  validateSearch: (
    search: Record<string, unknown>,
  ): { id?: number; edit?: number; find?: true; carry?: number } & Partial<PastReturn> => {
    const id = positiveId(search['id']);
    const edit = positiveId(search['edit']);
    const find = search['find'] === true || search['find'] === 'true';
    // `carry=40` is a new بکری with lot 40 of the goods left already on it (plan 76 point 1).
    const carry = positiveId(search['carry']);
    return {
      ...(id === undefined ? {} : { id }),
      ...(id !== undefined && edit !== undefined ? { edit } : {}),
      ...(id === undefined && find ? { find: true as const } : {}),
      ...(id === undefined && carry !== undefined ? { carry } : {}),
      ...(id === undefined ? {} : parsePastReturn(search)),
    };
  },
  component: lazyRouteComponent(
    () => import('../features/bikri/pages/BikriBookPage'),
    'BikriBookPage',
  ),
});

// پچھلی بکری (the route keeps its old name): today's بکری and the window of earlier days (PRD 12.7).
// `?end=2026-11-05` is the last day of the window, `row=41` the بکری to focus, `all=true` the tick
// "show every بکری" (plan 72).
const todayRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/today',
  validateSearch: (search: Record<string, unknown>) => parsePastSearch(search),
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

// The two pages the dashboard tiles open (plan 76): `/dues` is the simple buyer dues page (PRD FR-9.4) and
// `/commission` the commission page: a date, a month, a year or a range, and one buyer or seller (PRD FR-9.3, `?month=2026-10`,
// `?period=year&year=2026&side=seller&party=...`).
const buyerDuesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/dues',
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/BuyerDuesPage'),
    'BuyerDuesPage',
  ),
});

// The pages behind the other dashboard tiles (plan 79, PRD FR-9.1): the sellers (`?side=shop_owes|seller_owes`),
// کل اگراہی with its four sections, the people who borrowed and still owe, the sellers with an open advance, and
// the نقدی of each day (`?month=2026-10`). Each shows what its tile counts, and its total is the tile's.
const sellerDuesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/dues/sellers',
  validateSearch: (search: Record<string, unknown>) => parseSellerDuesSearch(search),
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/SellerDuesPage'),
    'SellerDuesPage',
  ),
});

const totalDuesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/dues/total',
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/TotalDuesPage'),
    'TotalDuesPage',
  ),
});

const lentOpenRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/dues/lent',
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/LentOpenPage'),
    'LentOpenPage',
  ),
});

const advancesOpenRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/dues/advances',
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/AdvancesOpenPage'),
    'AdvancesOpenPage',
  ),
});

const cashSalesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/cash-sales',
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/CashSalesPage'),
    'CashSalesPage',
  ),
});

// The page behind the dashboard tile of garden owners and others (PRD FR-16.13, plan 76 point 10): the money
// given to people who are not staff, and what came back, one month at a time.
const othersDuesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/others-dues',
  component: lazyRouteComponent(
    () => import('../features/cashbook/pages/OthersDuesPage'),
    'OthersDuesPage',
  ),
});

// The kharch page behind the dashboard tile آج کا خرچہ (PRD FR-9.5, plan 77): the shop's kharch and each partner's own, for
// a period (`?period=year&year=2026&partner=2`).
const kharchRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/kharch',
  validateSearch: (search: Record<string, unknown>) => parseKharchSearch(search),
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/KharchPage'),
    'KharchPage',
  ),
});

const commissionRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/commission',
  validateSearch: (search: Record<string, unknown>) => parseCommissionSearch(search),
  component: lazyRouteComponent(
    () => import('../features/dashboard/pages/CommissionPage'),
    'CommissionPage',
  ),
});

// The sales page behind the dashboard tile آج کی بکری (PRD FR-9.9, plan 83), built like the commission page: a date, a
// month, a year or a range, and a party, an item and the goods (`?period=year&year=2026&side=seller&party=...`).
const salesRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/sales',
  validateSearch: (search: Record<string, unknown>) => parseSalesSearch(search),
  component: lazyRouteComponent(() => import('../features/dashboard/pages/SalesPage'), 'SalesPage'),
});

// پارٹی کھاتے (PRD FR-12.43): the party ledgers in three levels. `?side=buyer|seller` is the list,
// `?party=20` one party's statement, `?party=20&date=2026-10-10` that day's slip.
const partyLedgersRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/party-ledgers',
  validateSearch: (search: Record<string, unknown>) => parseLedgerSearch(search),
  component: lazyRouteComponent(
    () => import('../features/party_ledgers/pages/PartyLedgersPage'),
    'PartyLedgersPage',
  ),
});

// باغات (PRD 12.11, FR-5.2): the gardens list, the form of a new garden (shares first, FR-12.29),
// and one garden with its type list (FR-7.67). Static `new` wins over the `$id` parameter.
const gardensRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/gardens',
  component: lazyRouteComponent(
    () => import('../features/gardens/pages/GardensPage'),
    'GardensPage',
  ),
});

const gardenNewRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/gardens/new',
  component: lazyRouteComponent(
    () => import('../features/gardens/pages/NewGardenPage'),
    'NewGardenPage',
  ),
});

const gardenRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/gardens/$id',
  validateSearch: parseGardenSearch,
  component: lazyRouteComponent(() => import('../features/gardens/pages/GardenPage'), 'GardenPage'),
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
  validateSearch: parseTripSearch,
  component: lazyRouteComponent(() => import('../features/trips/pages/TripPage'), 'TripPage'),
});

// The old address of a later sale (`/bikri/later?lot=40`) goes on to the sale book with that lot carried
// onto a new بکری, where new lots can be added beside it (plan 76 point 1, PRD FR-12.48). Without a lot
// it goes to the goods left, where a lot is chosen.
const laterSaleRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/bikri/later',
  validateSearch: (search: Record<string, unknown>): { lot?: number } => {
    const lot = positiveId(search['lot']);
    return lot === undefined ? {} : { lot };
  },
  beforeLoad: ({ search }) => {
    if (search.lot === undefined) return redirect({ to: '/bikri/stock' });
    return redirect({ to: '/bikri/book', search: { carry: search.lot } });
  },
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

// سرمایہ and the partner statement (PRD 17.4, FR-17.9, plan 17): in the partners area.
const partnersCapitalRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/partners/capital',
  validateSearch: parseCapitalSearch,
  component: lazyRouteComponent(
    () => import('../features/partners/pages/CapitalPage'),
    'CapitalPage',
  ),
});

const partnersStatementRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/partners/statement',
  validateSearch: parseStatementSearch,
  component: lazyRouteComponent(
    () => import('../features/partners/pages/StatementPage'),
    'StatementPage',
  ),
});

// مہینہ بند, the month close (PRD FR-17.6, FR-21.2, round 7): reached from the partners overview.
const monthRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/month',
  validateSearch: parseMonthSearch,
  component: lazyRouteComponent(
    () => import('../features/month/pages/MonthClosePage'),
    'MonthClosePage',
  ),
});

// روکڑ (PRD 5, 16, plan 17): the cash book, and the entry screens of the area. Each entry screen is
// ONE screen with a type list (PRD FR-7.67); `?type=` chooses the type. Alt+K opens the خرچہ screen
// as دکان خرچہ.
const rokarRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/rokar',
  component: lazyRouteComponent(
    () => import('../features/cashbook/pages/CashbookPage'),
    'CashbookPage',
  ),
});

const expenseRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/rokar/expense',
  validateSearch: parseExpenseSearch,
  component: lazyRouteComponent(
    () => import('../features/cashbook/pages/ExpensePage'),
    'ExpensePage',
  ),
});

// پھل (PRD 16.7, plan 17d): a tab next to خرچہ in the روکڑ area, one more entry in `EXPENSE_TABS`.
const fruitRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/rokar/fruit',
  validateSearch: parseFruitSearch,
  component: lazyRouteComponent(() => import('../features/fruit/pages/FruitPage'), 'FruitPage'),
});

const lentRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/rokar/lent',
  validateSearch: parseLentSearch,
  component: lazyRouteComponent(() => import('../features/cashbook/pages/LentPage'), 'LentPage'),
});

const generalRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/rokar/general',
  validateSearch: parseGeneralSearch,
  component: lazyRouteComponent(
    () => import('../features/cashbook/pages/GeneralPage'),
    'GeneralPage',
  ),
});

const khataRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/rokar/khata',
  validateSearch: parseKhataSearch,
  component: lazyRouteComponent(() => import('../features/cashbook/pages/KhataPage'), 'KhataPage'),
});

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

// R38, the dead amount (plan 40, PRD FR-10.10). Its own address, found before `/reports/$id`: the
// people moved to ڈوبی رقم, with a way to make one active again.
const deadAmountRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/reports/dead-amount',
  component: lazyRouteComponent(
    () => import('../features/reports/pages/DeadAmountPage'),
    'DeadAmountPage',
  ),
});

// R5, ادھاری تفصیل (plan 80 Q-R5-2, PRD R5 v1.43): the day's buyers on one compact page. Its own address, found
// before `/reports/$id`; the catalogue and the links that name `R5` land on it. The day is in the address.
const adhariSheetRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/reports/adhari-sheet',
  validateSearch: (search: Record<string, unknown>) => {
    const { date } = search;
    return isIsoDay(date) ? { date } : {};
  },
  component: lazyRouteComponent(
    () => import('../features/reports/pages/AdhariSheetPage'),
    'AdhariSheetPage',
  ),
});

const reportRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/reports/$id',
  validateSearch: parseReportSearch,
  // R38 has its own screen (it can make a person active again); the catalogue, the search and the
  // links that name `R38` all land on it.
  beforeLoad: ({ params }) => {
    // Like the voucher addresses above: the redirect is returned, which the router follows.
    if (params.id === 'R38') return redirect({ to: '/reports/dead-amount', replace: true });
    if (params.id === 'R5') return redirect({ to: '/reports/adhari-sheet', replace: true });
    return undefined;
  },
  component: lazyRouteComponent(() => import('../features/reports/pages/ReportPage'), 'ReportPage'),
});
// ---- end of Run B block ----

// ---- Run B, plan 21 (D5 early track): تجزیات. Replaces the "coming soon" page of /analytics. The
// area opens on view A; one view is /analytics/B (A to E and H). ----
const analyticsRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/analytics',
  // Plan 49: a button under an answer of the question panel opens a view with its filters in the address.
  validateSearch: parseAnalyticsSearch,
  component: lazyRouteComponent(
    () => import('../features/analytics/pages/AnalyticsPage'),
    'AnalyticsPage',
  ),
});

const analyticsViewRoute = createRoute({
  getParentRoute: () => appRoute,
  path: '/analytics/$view',
  validateSearch: parseAnalyticsSearch,
  component: lazyRouteComponent(
    () => import('../features/analytics/pages/AnalyticsPage'),
    'AnalyticsPage',
  ),
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
  todayRoute,
  partyLedgersRoute,
  stockRoute,
  buyerDuesRoute,
  sellerDuesRoute,
  totalDuesRoute,
  lentOpenRoute,
  advancesOpenRoute,
  cashSalesRoute,
  commissionRoute,
  salesRoute,
  kharchRoute,
  othersDuesRoute,
  laterSaleRoute,
  rokarRoute,
  expenseRoute,
  fruitRoute,
  lentRoute,
  khataRoute,
  generalRoute,
  gardensRoute,
  gardenNewRoute,
  gardenRoute,
  tripsRoute,
  tripNewRoute,
  tripRoute,
  partnersRoute,
  partnersSetupRoute,
  partnersCapitalRoute,
  partnersStatementRoute,
  monthRoute,
  analyticsRoute,
  analyticsViewRoute,
  backupRoute,
  reportsRoute,
  deadAmountRoute,
  adhariSheetRoute,
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
        path: '/dev/gardens',
        component: lazyRouteComponent(() => import('./dev/GardensPage'), 'GardensPage'),
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
        path: '/dev/rokar',
        component: lazyRouteComponent(() => import('./dev/RokarPage'), 'RokarPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/index',
        component: lazyRouteComponent(() => import('./dev/IndexPage'), 'IndexPage'),
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

// ---- Plan 56: the development page of the first screen and the restore pages (dev only). ----
const restoreDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/restore',
        component: lazyRouteComponent(
          () => import('./dev/RestoreFirstRunPage'),
          'RestoreFirstRunPage',
        ),
      }),
    ]
  : [];
// ---- end of plan 56 block ----

// ---- Plan 75: the development page of the sign-in screen and its brand panel (dev only). ----
const loginDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/login',
        component: lazyRouteComponent(() => import('./dev/LoginDevPage'), 'LoginDevPage'),
      }),
    ]
  : [];
// ---- end of plan 75 block ----

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

// ---- Plan 66: the development page of the party ledgers (dev only, like the others). ----
const partyLedgersDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/party-ledgers',
        validateSearch: (search: Record<string, unknown>) => parseLedgerSearch(search),
        component: lazyRouteComponent(
          () => import('./dev/PartyLedgersDevPage'),
          'PartyLedgersDevPage',
        ),
      }),
    ]
  : [];
// ---- end of plan 66 block ----

// ---- Run B, plan 21: the development pages of the analytics screens (dev only, like the others). ----
const analyticsDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/analytics',
        component: lazyRouteComponent(() => import('./dev/AnalyticsDevPage'), 'AnalyticsDevPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/analytics/$view',
        component: lazyRouteComponent(() => import('./dev/AnalyticsDevPage'), 'AnalyticsDevPage'),
      }),
    ]
  : [];
// ---- end of Run B block ----

// ---- Run B, plan 22: the development page of the new Settings tabs (dev only, like the others). ----
const settingsDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/settings',
        component: lazyRouteComponent(() => import('./dev/SettingsDevPage'), 'SettingsDevPage'),
      }),
    ]
  : [];
// ---- end of Run B block ----

// ---- Run B, round 7: the development page of the month close screen (dev only, like the others). ----
const monthDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/month',
        component: lazyRouteComponent(() => import('./dev/MonthDevPage'), 'MonthDevPage'),
      }),
    ]
  : [];
// ---- end of Run B block ----

// ---- Run B, round 9: the development page of the dashboard and the bell (dev only, like the others). ----
const dashboardDevRoutes = import.meta.env.DEV
  ? [
      // Plan 40: the dead amount (R38, the move box, the party card) with the worked numbers of E2E-44.
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/dead-amount',
        component: lazyRouteComponent(() => import('./dev/DeadAmountDevPage'), 'DeadAmountDevPage'),
      }),
      // Plan 49: the question panel and its Settings group with the backend's own fixtures.
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/assistant',
        component: lazyRouteComponent(() => import('./dev/AssistantDevPage'), 'AssistantDevPage'),
      }),
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/dashboard',
        component: lazyRouteComponent(() => import('./dev/DashboardDevPage'), 'DashboardDevPage'),
      }),
      // Plan 77: the kharch page (the shop's kharch and each partner's own) with the backend's own answers.
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/kharch',
        validateSearch: (search: Record<string, unknown>) => parseKharchSearch(search),
        component: lazyRouteComponent(() => import('./dev/KharchDevPage'), 'KharchDevPage'),
      }),
      // Plan 79: the pages behind the dashboard tiles (sellers, total dues, money lent, advances, cash sales) with the numbers of the plan.
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/dues/$page',
        validateSearch: (search: Record<string, unknown>) => parseSellerDuesSearch(search),
        component: lazyRouteComponent(() => import('./dev/DuesDevPage'), 'DuesDevPage'),
      }),
      // Plan 77: the commission page with a period and a party, with the numbers of the plan.
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/commission',
        validateSearch: (search: Record<string, unknown>) => parseCommissionSearch(search),
        component: lazyRouteComponent(() => import('./dev/CommissionDevPage'), 'CommissionDevPage'),
      }),
      // Plan 83: the sales page with the numbers of the plan.
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/sales',
        validateSearch: (search: Record<string, unknown>) => parseSalesSearch(search),
        component: lazyRouteComponent(() => import('./dev/SalesDevPage'), 'SalesDevPage'),
      }),
    ]
  : [];
// ---- end of Run B block ----

// ---- Plan 38, A1: the development page of the lists and screens of the انڈکس area (dev only). ----
const listsDevRoutes = import.meta.env.DEV
  ? [
      createRoute({
        getParentRoute: () => rootRoute,
        path: '/dev/lists/$kind',
        component: lazyRouteComponent(() => import('./dev/ListsDevPage'), 'ListsDevPage'),
      }),
    ]
  : [];
// ---- end of plan 38 block ----

const routeTree = rootRoute.addChildren([
  appRoute.addChildren(appRoutes),
  printRoute,
  ...devRoutes,
  ...backupDevRoutes,
  ...restoreDevRoutes,
  ...loginDevRoutes,
  ...reportsDevRoutes,
  ...partyLedgersDevRoutes,
  ...analyticsDevRoutes,
  ...settingsDevRoutes,
  ...monthDevRoutes,
  ...dashboardDevRoutes,
  ...listsDevRoutes,
]);

// In development, /dev/gallery typed without the # moves to /#/dev/gallery before the router
// reads the address. In a build, DEV is false, so this call and the helper are removed.
if (import.meta.env.DEV) moveDevPathToHash();

/** The app's router. Tests give it a memory history; the app uses the hash (no address bar). */
export function createAppRouter(history: RouterHistory = createHashHistory()) {
  return createRouter({ routeTree, history, defaultErrorComponent: RouteError });
}

export const router = createAppRouter();

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
