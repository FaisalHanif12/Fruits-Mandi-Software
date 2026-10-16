// Which route each menu item opens, and which item a route belongs to (PRD FR-5.2, FR-5.3).
// Menu items whose module is not built yet open the "coming soon" page.

import type { MainNavId } from './layout/MainNav';
import type { ModuleMenuId } from './layout/ModuleMenu';

export const MAIN_ROUTES = {
  dashboard: '/',
  index: '/index',
  bikri: '/bikri',
  rokar: '/rokar',
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

/** The main menu item a path belongs to. */
export function mainNavOf(pathname: string): MainNavId {
  const first = pathname.split('/')[1] ?? '';
  const found = (Object.keys(MAIN_ROUTES) as MainNavId[]).find(
    (id) => id !== 'dashboard' && MAIN_ROUTES[id] === `/${first}`,
  );
  return found ?? 'dashboard';
}

/** The side menu item of the بکری area a path belongs to. */
export function moduleMenuOf(pathname: string): ModuleMenuId {
  const found = (Object.keys(MODULE_ROUTES) as ModuleMenuId[]).find(
    (id) => id !== 'bikri-book' && MODULE_ROUTES[id] === pathname,
  );
  return found ?? 'bikri-book';
}
