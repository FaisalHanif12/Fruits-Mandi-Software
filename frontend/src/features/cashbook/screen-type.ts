// The types of the three entry screens of the روکڑ area and the addresses that choose them (PRD
// FR-7.67). Kept apart from the screens so the router can read the address without loading a
// screen (Code Standards F36). A type is the voucher type of the entry it makes, so one word names
// the type list, the number series (`vouchers-number-<type>`) and the posting.

/** The types in the type list of the خرچہ screen, in the order of the list (PRD 16.2). */
export type ExpenseType = 'shop_expense' | 'personal_expense' | 'personal_return';
/** The types of the ادھار رقم screen: money lent, and money returned (PRD 16.5). */
export type LentType = 'money_lent' | 'money_lent_return';
/** The types of the الگ کھاتہ screen: paid out (minus) and received (plus) (PRD 16.6). */
export type KhataType = 'khata_out' | 'khata_in';

export const EXPENSE_TYPES: readonly ExpenseType[] = [
  'shop_expense',
  'personal_expense',
  'personal_return',
];
export const LENT_TYPES: readonly LentType[] = ['money_lent', 'money_lent_return'];
export const KHATA_TYPES: readonly KhataType[] = ['khata_out', 'khata_in'];

function pick<T extends string>(types: readonly T[], value: unknown): T | null {
  return types.find((type) => type === value) ?? null;
}

/**
 * The address of an entry screen is `/rokar/expense?type=personal_expense`. A value that is not
 * one of the screen's types is left out, and the screen starts with its first type.
 */
export interface TypeSearch<T extends string> {
  type?: T;
}

function parser<T extends string>(types: readonly T[]) {
  return (search: Record<string, unknown>): TypeSearch<T> => {
    const type = pick(types, search['type']);
    return type === null ? {} : { type };
  };
}

export const parseExpenseSearch = parser(EXPENSE_TYPES);
export const parseLentSearch = parser(LENT_TYPES);
export const parseKhataSearch = parser(KHATA_TYPES);
