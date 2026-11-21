// The page each child page goes back to when there is nothing in the trail (it was opened by an address,
// after a restart, or from the search). Only pages that are opened from another page are listed; a page
// reached from a menu has no parent, and shows no Back button unless the user really came from a page.
// Plan 78 F1. The list of pages and where Back was missing is in the plan's Section 8.
//
// Each rule is a pattern for the path and the path to go to. The first rule that fits wins, so the more
// special paths come first (the form of an account before the list of accounts).

interface ParentRule {
  match: RegExp;
  parent: string;
}

const RULES: readonly ParentRule[] = [
  // The pages the dashboard tiles and the status bar open. The five tile pages of plan 79 are the dues pages
  // under /dues (sellers, total, lent, advances) and the cash sales page (PRD FR-9.1 v1.44, FR-7.92). Plan 83 adds
  // the sales page (PRD FR-9.9).
  {
    match:
      /^\/(dues|dues\/(sellers|total|lent|advances)|others-dues|kharch|commission|sales|day|cash-sales)$/,
    parent: '/',
  },
  // انڈکس: the forms are opened from their list, the lists from the menu of lists.
  { match: /^\/index\/accounts\/[^/]+$/, parent: '/index/accounts' },
  { match: /^\/index\/rules\/[^/]+$/, parent: '/index/rules' },
  { match: /^\/index\/[^/]+$/, parent: '/index' },
  // One garden or trip, and a new one, from its list.
  { match: /^\/gardens\/[^/]+$/, parent: '/gardens' },
  { match: /^\/trips\/[^/]+$/, parent: '/trips' },
  // The partners area: the screens opened from the overview, and the month close.
  { match: /^\/partners\/(setup|capital|statement)$/, parent: '/partners' },
  { match: /^\/month$/, parent: '/partners' },
  // A report and an analytics view from their home.
  { match: /^\/reports\/[^/]+$/, parent: '/reports' },
  { match: /^\/analytics\/[^/]+$/, parent: '/analytics' },
  // The collection round is opened from the receipt screen.
  { match: /^\/bikri\/collection$/, parent: '/bikri/voucher' },
  // A later sale is opened from the stock list.
  { match: /^\/bikri\/later$/, parent: '/bikri/stock' },
];

/**
 * The parent of a path, or null when the page has none. The form of a record under a list of the index
 * (`/index/$kind/$id`) goes to that list: `/index/cities/new` goes to `/index/cities`.
 */
export function parentOf(path: string): string | null {
  const record = /^\/index\/([^/]+)\/(?:new|\d+)$/.exec(path);
  if (record !== null) return `/index/${record[1] ?? ''}`;
  return RULES.find((rule) => rule.match.test(path))?.parent ?? null;
}
