// The tabs of the خرچہ area (PRD 16.7, Architecture 5): خرچہ, and پھل (fruit taken) next to it
// (plan 17d). This is the one list: a tab is one entry here and one route. The tab strip
// (`components/ExpenseTabs.tsx`) shows itself as soon as there are two; the fruit screen
// (`features/fruit`) draws it too.

export interface ExpenseTab {
  id: string;
  /** The Fluent message ID of the tab's name. */
  labelId: string;
  path: string;
}

export const EXPENSE_TABS: readonly ExpenseTab[] = [
  { id: 'expense', labelId: 'cashbook-tab-expense', path: '/rokar/expense' },
  { id: 'fruit', labelId: 'cashbook-tab-fruit', path: '/rokar/fruit' },
];
