// The tabs of the خرچہ area (PRD 16.7, Architecture 5): خرچہ today, and پھل (fruit taken) next to it
// with plan 17d. This is the one list. The fruit screen adds one entry here and one route; the
// tab strip (`components/ExpenseTabs.tsx`) shows itself as soon as there are two.

export interface ExpenseTab {
  id: string;
  /** The Fluent message ID of the tab's name. */
  labelId: string;
  path: string;
}

export const EXPENSE_TABS: readonly ExpenseTab[] = [
  { id: 'expense', labelId: 'cashbook-tab-expense', path: '/rokar/expense' },
];
