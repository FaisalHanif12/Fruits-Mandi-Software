// Test helper: an in-memory fake of the rules and the charge settings commands (plan 09). The
// wrappers are mocked in the test file with vi.mock; this sets their answers from a small store,
// the way a healthy backend answers. The amounts of "Test a rule" are built the way a reply
// arrives (money): the screen only shows them.

import { vi } from 'vitest';
import * as rulesApi from '../../src/features/rules/api';
import * as settingsApi from '../../src/features/settings/api';
import type {
  ChargeTest,
  RuleOverlap,
  RuleRow,
  RuleSaved,
  RuleTest,
} from '../../src/features/rules/types';
import type { RuleTestInput } from '../../src/bindings/RuleTestInput';
import type { ChargesSettings, OverridesSettings } from '../../src/features/settings/types';
import type { RecordRef } from '../../src/bindings/RecordRef';
import { money } from './accounts';

/** The basis points of a percent typed as text ("2.5" is 250). A test double: the backend's job. */
const bpsOf = (text: string): number => Math.round(Number(text) * 100);

export function rule(id: number, extra: Partial<RuleRow> & { value?: string } = {}): RuleRow {
  const { value, ...rest } = extra;
  const method = rest.method ?? 'percent';
  const text = value ?? rest.value_text ?? '6';
  return {
    id,
    version: 1,
    side: 'seller',
    charge_type: 'commission',
    account: null,
    item: null,
    item_group: null,
    level: 'default',
    method,
    value_bps: method === 'percent' ? bpsOf(text) : null,
    value_paisa: method === 'percent' ? null : money(Number(text)),
    value_text: text,
    from_date: '2026-08-31',
    to_date: null,
    is_active: true,
    note: '',
    is_default: false,
    ...rest,
  };
}

/** The defaults seeded by the migration (PRD 4.2): 6% on both sides, لاگا 8 per نگ on fruit. */
export function seededRules(): RuleRow[] {
  return [
    rule(1, { is_default: true }),
    rule(2, { side: 'buyer', is_default: true }),
    rule(3, {
      side: 'buyer',
      charge_type: 'laga',
      method: 'per_unit',
      value: '8',
      item_group: 'fruit',
      level: 'item_group',
      is_default: true,
    }),
  ];
}

export interface FakeRules {
  rows: RuleRow[];
  /** What the next save answers with, besides the saved rule. */
  overlapsOnNextSave: RuleOverlap[];
  charges: ChargesSettings;
  overrides: OverridesSettings;
}

/**
 * Sets the answers of the rules and charge-setting commands from `rows` and returns the store, so a
 * test can read what was saved. The wrappers must have been mocked with vi.mock in the test file.
 */
export function fakeRules(rows: RuleRow[] = seededRules()): FakeRules {
  const store: FakeRules = {
    rows: [...rows],
    overlapsOnNextSave: [],
    charges: {
      seller_commission: { rule_id: 1, method: 'percent', value_text: '6', version: 1 },
      buyer_commission: { rule_id: 2, method: 'percent', value_text: '6', version: 1 },
      laga_fruit: { rule_id: 3, method: 'per_unit', value_text: '8', version: 1 },
    },
    overrides: { allow_line_override: false, credit_limit_mode: 'warn' },
  };
  let nextId = 100;

  vi.mocked(rulesApi.rulesList).mockImplementation((filter) =>
    Promise.resolve(
      store.rows.filter(
        (row) =>
          (filter.include_inactive || row.is_active) &&
          (filter.side === null || row.side === filter.side) &&
          (filter.charge_type === null || row.charge_type === filter.charge_type) &&
          (filter.account_id === null || row.account?.id === filter.account_id) &&
          (filter.item_id === null || row.item?.id === filter.item_id),
      ),
    ),
  );
  vi.mocked(rulesApi.rulesGet).mockImplementation((id) => {
    const found = store.rows.find((row) => row.id === id);
    return found === undefined ? Promise.reject(new Error('not found')) : Promise.resolve(found);
  });
  vi.mocked(rulesApi.rulesSave).mockImplementation((draft) => {
    const isoDate = (text: string) => text.split('-').reverse().join('-');
    const saved = rule(draft.id ?? (nextId += 1), {
      version: (draft.version ?? 0) + 1,
      side: draft.side,
      charge_type: draft.charge_type,
      account: draft.account,
      item: draft.item,
      item_group: draft.item_group === '' ? null : draft.item_group,
      method: draft.method,
      value: draft.value,
      from_date: isoDate(draft.from_date),
      to_date: draft.to_date === '' ? null : isoDate(draft.to_date),
      is_active: draft.is_active,
      note: draft.note,
      level:
        draft.account !== null
          ? draft.item !== null
            ? 'party_item'
            : 'party'
          : draft.item !== null
            ? 'item'
            : draft.item_group !== ''
              ? 'item_group'
              : 'default',
    });
    const index = store.rows.findIndex((row) => row.id === saved.id);
    if (index >= 0) store.rows[index] = saved;
    else store.rows.push(saved);
    const overlaps = store.overlapsOnNextSave;
    store.overlapsOnNextSave = [];
    return Promise.resolve<RuleSaved>({ row: saved, overlaps });
  });
  vi.mocked(rulesApi.rulesSetActive).mockImplementation((id, isActive) => {
    const index = store.rows.findIndex((row) => row.id === id);
    const found = store.rows[index];
    if (found === undefined) return Promise.reject(new Error('not found'));
    const changed = { ...found, is_active: isActive, version: found.version + 1 };
    store.rows[index] = changed;
    return Promise.resolve<RuleSaved>({ row: changed, overlaps: [] });
  });
  vi.mocked(settingsApi.settingsGetCharges).mockImplementation(() =>
    Promise.resolve(structuredClone(store.charges)),
  );
  vi.mocked(settingsApi.settingsSetCharges).mockImplementation((input) => {
    store.charges = {
      seller_commission: {
        ...store.charges.seller_commission,
        method: input.seller_commission.method,
        value_text: input.seller_commission.value,
        version: input.seller_commission.version + 1,
      },
      buyer_commission: {
        ...store.charges.buyer_commission,
        method: input.buyer_commission.method,
        value_text: input.buyer_commission.value,
        version: input.buyer_commission.version + 1,
      },
      laga_fruit: {
        ...store.charges.laga_fruit,
        method: input.laga_fruit.method,
        value_text: input.laga_fruit.value,
        version: input.laga_fruit.version + 1,
      },
    };
    return Promise.resolve(structuredClone(store.charges));
  });
  vi.mocked(settingsApi.settingsGetOverrides).mockImplementation(() =>
    Promise.resolve({ ...store.overrides }),
  );
  vi.mocked(settingsApi.settingsSetOverrides).mockImplementation((input) => {
    store.overrides = { ...input };
    return Promise.resolve({ ...store.overrides });
  });
  return store;
}

/**
 * One charge of the answer of "Test a rule" for the PRD 4.4 line (20 نگ at 600): who won, at which
 * level, which levels had no rule, and the amount with the typed working the backend sends.
 */
export function testCharge(extra: Partial<ChargeTest> = {}): ChargeTest {
  return {
    side: 'seller',
    charge_type: 'commission',
    winner: {
      level: 'default',
      rule_id: 1,
      method: 'percent',
      value_bps: 600,
      value_paisa: null,
      manual: false,
    },
    skipped: [
      { level: 'line_override', reason: 'no_override' },
      { level: 'party_item', reason: 'no_rule' },
      { level: 'party', reason: 'no_rule' },
      { level: 'item', reason: 'no_rule' },
      { level: 'item_group', reason: 'no_rule' },
    ],
    amount_paisa: money(720),
    working: {
      message_id: 'working-percent',
      args: [
        { name: 'base', value: { kind: 'money', paisa: money(12000) } },
        { name: 'bps', value: { kind: 'percent', bps: 600 } },
        { name: 'result', value: { kind: 'money', paisa: money(720) } },
        { name: 'level', value: { kind: 'level', level: 'default' } },
      ],
      rule_id: 1,
    },
    ...extra,
  };
}

export function testResult(): RuleTest {
  return {
    line_value_paisa: money(12000),
    charges: [
      testCharge(),
      testCharge({
        side: 'buyer',
        winner: {
          level: 'default',
          rule_id: 2,
          method: 'percent',
          value_bps: 600,
          value_paisa: null,
          manual: false,
        },
      }),
      testCharge({
        side: 'buyer',
        charge_type: 'laga',
        winner: {
          level: 'item_group',
          rule_id: 3,
          method: 'per_unit',
          value_bps: null,
          value_paisa: money(8),
          manual: false,
        },
        skipped: [
          { level: 'line_override', reason: 'no_override' },
          { level: 'party_item', reason: 'no_rule' },
          { level: 'party', reason: 'no_rule' },
          { level: 'item', reason: 'no_rule' },
        ],
        amount_paisa: money(160),
        working: {
          message_id: 'working-per-unit',
          args: [
            { name: 'qty', value: { kind: 'count', value: 20 } },
            { name: 'value', value: { kind: 'money', paisa: money(8) } },
            { name: 'result', value: { kind: 'money', paisa: money(160) } },
            { name: 'level', value: { kind: 'level', level: 'item_group' } },
          ],
          rule_id: 3,
        },
      }),
      testCharge({
        side: 'seller',
        charge_type: 'labour',
        winner: null,
        skipped: [
          { level: 'party_item', reason: 'no_rule' },
          { level: 'party', reason: 'no_rule' },
          { level: 'item', reason: 'no_rule' },
          { level: 'item_group', reason: 'no_rule' },
          { level: 'default', reason: 'no_rule' },
        ],
        amount_paisa: money(0),
        working: {
          message_id: 'working-none',
          args: [{ name: 'result', value: { kind: 'money', paisa: money(0) } }],
          rule_id: null,
        },
      }),
    ],
  };
}

/** Makes "Test a rule" answer with `result`, and remembers what it was asked. */
export function fakeRuleTest(result: RuleTest = testResult()) {
  const asked: RuleTestInput[] = [];
  vi.mocked(rulesApi.rulesTest).mockImplementation((input) => {
    asked.push(input);
    return Promise.resolve(result);
  });
  return asked;
}

/** A pointer to a record, as a rule holds its party and item. */
export const pointer = (id: number, code: string, ur: string, en: string): RecordRef => ({
  id,
  code,
  name_ur: ur,
  name_en: en,
});
