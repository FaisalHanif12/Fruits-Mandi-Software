// Where the partner code stays open (plan 44, PRD FR-17.11): while the user stays on the screens
// that show partner figures. A move to a screen of another group closes it again. The partner
// section is one group; the dashboard (the partner strip), each report and the analytics are groups
// of their own, so opening the code on the dashboard does not keep partner figures open in the
// reports. Pure; the shell calls it on every move.

/** The group of screens an address belongs to. */
export function partnerAreaOf(pathname: string): string {
  if (pathname === '/partners' || pathname.startsWith('/partners/')) return 'partners';
  if (pathname === '/' || pathname === '') return 'dashboard';
  if (pathname.startsWith('/analytics')) return 'analytics';
  if (pathname.startsWith('/reports/')) return pathname.split('/').slice(0, 3).join('/');
  return pathname;
}
