// Which route each menu item opens, and which item a route belongs to (PRD FR-5.2, FR-5.3).
// Menu items whose module is not built yet open the "coming soon" page.

import { EXPENSE_TABS } from '../features/cashbook/expense-tabs';
import { asScreenType, type ScreenType } from '../features/vouchers/screen-type';
import type { MainNavId } from './layout/MainNav';
import type { ModuleMenuId, RokarMenuId } from './layout/ModuleMenu';

export const MAIN_ROUTES = {
  dashboard: '/',
  index: '/index',
  bikri: '/bikri',
  rokar: '/rokar',
  gardens: '/gardens',
  trips: '/trips',
  partners: '/partners',
  analytics: '/analytics',
  reports: '/reports',
  backup: '/backup',
  settings: '/settings',
} as const satisfies Record<MainNavId, string>;

export const MODULE_ROUTES = {
  'bikri-book': '/bikri/book',
  'party-ledgers': '/bikri/party-ledgers',
  today: '/bikri/today',
  stock: '/bikri/stock',
} as const satisfies Record<ModuleMenuId, string>;

/**
 * The addresses of the tabs of the روکڑ area (PRD 5, plan 17, FR-5.3 v1.57). وصولی, ادائیگی and پیشگی are
 * one voucher screen (PRD FR-15.10) with the type of the tab in its address (`?type=`); the address here is
 * the screen's, and `asScreenType(id)` says which type a tab opens.
 */
export const ROKAR_ROUTES = {
  cashbook: '/rokar',
  receipt: '/bikri/voucher',
  payment: '/bikri/voucher',
  advance: '/bikri/voucher',
  expense: '/rokar/expense',
  lent: '/rokar/lent',
  khata: '/rokar/khata',
  general: '/rokar/general',
} as const satisfies Record<RokarMenuId, string>;

/**
 * The addresses that send a user to the voucher screen with a type chosen (the router redirects each one,
 * `voucherTypeRoute`). The top bar's search opens them by name (PRD FR-5.4, FR-15.10).
 */
export const VOUCHER_ROUTES = {
  receipt: '/bikri/receipt',
  payment: '/bikri/payment',
  advance: '/bikri/advance',
} as const satisfies Record<ScreenType, string>;

/** The screens of the partners area besides the overview (PRD 17.3, 17.4). */
export const PARTNERS_ROUTES = {
  setup: '/partners/setup',
  capital: '/partners/capital',
  statement: '/partners/statement',
} as const;

/**
 * The روکڑ tab a path belongs to. The پھل tab belongs to خرچہ (PRD 16.7). The voucher screen is one address
 * for three tabs; its `type` in the address says which one is open, and the اگراہی فہرست belongs to وصولی
 * (PRD FR-5.3 v1.57, FR-7.67, FR-15.10).
 */
export function rokarMenuOf(pathname: string, search: Record<string, unknown> = {}): RokarMenuId {
  if (EXPENSE_TABS.some((tab) => tab.path === pathname)) return 'expense';
  if (pathname === '/bikri/voucher') return asScreenType(search['type']) ?? 'receipt';
  if (pathname === '/bikri/collection') return 'receipt';
  const found = (Object.keys(ROKAR_ROUTES) as RokarMenuId[]).find(
    (id) => id !== 'cashbook' && ROKAR_ROUTES[id] === pathname,
  );
  return found ?? 'cashbook';
}

/**
 * The address of the screen a voucher kind belongs to, for a search result (PRD FR-5.4). Expenses,
 * money lent and the khata have their own screens in the روکڑ area; the rest open the cash book.
 */
export function rokarAddressOf(voucherType: string): string {
  switch (voucherType) {
    case 'shop_expense':
    case 'personal_expense':
    case 'personal_return':
      return ROKAR_ROUTES.expense;
    case 'money_lent':
    case 'money_lent_return':
      return ROKAR_ROUTES.lent;
    case 'khata_out':
    case 'khata_in':
      return ROKAR_ROUTES.khata;
    case 'general':
      return ROKAR_ROUTES.general;
    // The capital screen is in the partners area (PRD 17.4).
    case 'capital_in':
    case 'capital_out':
      return PARTNERS_ROUTES.capital;
    default:
      return ROKAR_ROUTES.cashbook;
  }
}

/**
 * The main menu item a path belongs to. The voucher screen and the اگراہی فہرست are screens of the روکڑ
 * area since v1.57 (PRD FR-5.3), though their addresses start with /bikri.
 */
export function mainNavOf(pathname: string): MainNavId {
  if (pathname === '/bikri/voucher' || pathname === '/bikri/collection') return 'rokar';
  const first = pathname.split('/')[1] ?? '';
  const found = (Object.keys(MAIN_ROUTES) as MainNavId[]).find(
    (id) => id !== 'dashboard' && MAIN_ROUTES[id] === `/${first}`,
  );
  return found ?? 'dashboard';
}

/** The tab of the بکری area a path belongs to (PRD FR-5.3 v1.57). */
export function moduleMenuOf(pathname: string): ModuleMenuId {
  // A later sale starts from the list of goods left (plan 16).
  if (pathname === '/bikri/later') return 'stock';
  const found = (Object.keys(MODULE_ROUTES) as ModuleMenuId[]).find(
    (id) => id !== 'bikri-book' && MODULE_ROUTES[id] === pathname,
  );
  return found ?? 'bikri-book';
}
