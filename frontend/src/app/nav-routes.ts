// Which route each menu item opens, and which item a route belongs to (PRD FR-5.2, FR-5.3).
// Menu items whose module is not built yet open the "coming soon" page.

import { asScreenType } from '../features/vouchers/screen-type';
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
  receipt: '/bikri/receipt',
  payment: '/bikri/payment',
  advance: '/bikri/advance',
  pukhta: '/bikri/pukhta',
  today: '/bikri/today',
  stock: '/bikri/stock',
} as const satisfies Record<ModuleMenuId, string>;

/**
 * The addresses of the روکڑ side menu (PRD 5, plan 17). The last item is the voucher screen of the
 * بکری area (PRD FR-15.10), opened as وصولی; it is a link, not a screen of this area.
 */
export const ROKAR_ROUTES = {
  cashbook: '/rokar',
  expense: '/rokar/expense',
  lent: '/rokar/lent',
  khata: '/rokar/khata',
  general: '/rokar/general',
  voucher: '/bikri/voucher',
} as const satisfies Record<RokarMenuId, string>;

/** The screens of the partners area besides the overview (PRD 17.3, 17.4). */
export const PARTNERS_ROUTES = {
  setup: '/partners/setup',
  capital: '/partners/capital',
  statement: '/partners/statement',
} as const;

/** The روکڑ side menu item a path belongs to. */
export function rokarMenuOf(pathname: string): RokarMenuId {
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

/** The main menu item a path belongs to. */
export function mainNavOf(pathname: string): MainNavId {
  const first = pathname.split('/')[1] ?? '';
  const found = (Object.keys(MAIN_ROUTES) as MainNavId[]).find(
    (id) => id !== 'dashboard' && MAIN_ROUTES[id] === `/${first}`,
  );
  return found ?? 'dashboard';
}

/**
 * The side menu item of the بکری area a path belongs to. The voucher screen is one address for three
 * items; its `type` in the address says which one is current (PRD FR-7.67, FR-15.10). The اگراہی فہرست
 * belongs to the receipt item.
 */
export function moduleMenuOf(pathname: string, search: Record<string, unknown> = {}): ModuleMenuId {
  if (pathname === '/bikri/voucher') return asScreenType(search['type']) ?? 'receipt';
  if (pathname === '/bikri/collection') return 'receipt';
  // A later sale starts from the list of goods left (plan 16).
  if (pathname === '/bikri/later') return 'stock';
  const found = (Object.keys(MODULE_ROUTES) as ModuleMenuId[]).find(
    (id) => id !== 'bikri-book' && MODULE_ROUTES[id] === pathname,
  );
  return found ?? 'bikri-book';
}
