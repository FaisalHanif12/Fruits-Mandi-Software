// The canned data of the own-lot tests has the shape of the backend's real answers (plan 16b, T4b),
// and every working the backend can send has its words in both languages. The backend wrote each
// real answer in its own golden test (tests/fixtures/*.json, copied, never edited). This compares
// the keys and kinds at every level, so a test that passes on canned data cannot hide a field the
// real backend does not send, or one it sends and the canned data forgot.

import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { CANCEL_DONE } from '../helpers/bikri-plan16-real';
import * as own from '../helpers/bikri-own-real';
import { e34OwnLotPart, e34OwnStockRow } from '../helpers/trips-data';
import { message } from '../helpers/render';

type Shape = string | null | Shape[] | { [key: string]: Shape };

/** The keys and kinds of a value at every level; a list is shown by its first element. */
function shapeOf(value: unknown): Shape {
  if (value === null || value === undefined) return null;
  if (Array.isArray(value)) return value.length === 0 ? [] : [shapeOf(value[0])];
  if (typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, item]) => [key, shapeOf(item)]),
    );
  }
  return typeof value;
}

/** The first difference between two shapes, or null. A null or an empty list can hold anything. */
function firstDifference(a: Shape, b: Shape, path = ''): string | null {
  if (a === null || b === null) return null;
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b)) return `${path}: list against not a list`;
    const [x, y] = [a[0], b[0]];
    return x === undefined || y === undefined ? null : firstDifference(x, y, `${path}[]`);
  }
  if (typeof a === 'string' || typeof b === 'string') {
    return a === b ? null : `${path}: ${JSON.stringify(a)} against ${JSON.stringify(b)}`;
  }
  const keys = new Set([...Object.keys(a), ...Object.keys(b)]);
  for (const key of [...keys].sort()) {
    const [x, y] = [a[key], b[key]];
    if (x === undefined) return `${path}.${key}: the real answer has no such field`;
    if (y === undefined) return `${path}.${key}: the canned data has no such field`;
    const inner = firstDifference(x, y, `${path}.${key}`);
    if (inner !== null) return inner;
  }
  return null;
}

const same = (realValue: unknown, canned: unknown) =>
  firstDifference(shapeOf(realValue), shapeOf(canned));

describe('the canned data of the own-lot tests has the real shape', () => {
  it('the part of a lot of trip goods in a preview, with its two workings', () => {
    const real = own.OWN_PREVIEW_E34.lots[0]?.own;
    expect(real).not.toBeNull();
    expect(same(real, e34OwnLotPart())).toBeNull();
    // The names of the workings are the ones the backend sends.
    expect(real?.cost_working.message_id).toBe(e34OwnLotPart().cost_working.message_id);
    expect(real?.result_working.message_id).toBe(e34OwnLotPart().result_working.message_id);
    expect(real?.credits[0]?.working.message_id).toBe(
      e34OwnLotPart().credits[0]?.working.message_id,
    );
  });

  it('a row of trip goods in باقی مال, and the totals next to the rows', () => {
    expect(same(own.STOCK_OWN_ROWS.own_rows[0], e34OwnStockRow())).toBeNull();
    expect(own.STOCK_OWN_ROWS.own_balance_total).toBe(5);
    expect(own.STOCK_OWN_ROWS.own_cost_left_paisa).toBe(260_000);
  });

  it('the cancel of a بکری with trip goods answers like any other cancel', () => {
    expect(same(own.OWN_CANCEL, CANCEL_DONE)).toBeNull();
  });

  it('the real preview of an own lot has the keys of a preview of any other lot', () => {
    expect(
      same(own.OWN_PREVIEW_E34.lots[0], {
        ...own.OWN_PREVIEW_NOT_FULLY_SOLD.lots[0],
      }),
    ).toBeNull();
    // A lot that is not trip goods says so, with a null `own`.
    expect(own.OWN_GET_E34.lots[0]?.purchase).toEqual({
      id: 1,
      number: 1,
      trip_id: 1,
      trip_number: 1,
      trip_code: '0070',
    });
  });
});

describe('every working the backend can send has its words in both languages', () => {
  /** Every `message_id` in every real answer the backend wrote. */
  function workingIds(): Map<string, Set<string>> {
    const found = new Map<string, Set<string>>();
    const walk = (value: unknown) => {
      if (Array.isArray(value)) value.forEach(walk);
      else if (typeof value === 'object' && value !== null) {
        const record = value as Record<string, unknown>;
        const id = record['message_id'];
        const args = record['args'];
        if (typeof id === 'string' && Array.isArray(args)) {
          const names = found.get(id) ?? new Set<string>();
          for (const arg of args) {
            const name = (arg as { name?: unknown }).name;
            if (typeof name === 'string') names.add(name);
          }
          found.set(id, names);
        }
        Object.values(record).forEach(walk);
      }
    };
    const dir = path.resolve(__dirname, '../fixtures');
    for (const file of readdirSync(dir).filter((name) => name.endsWith('.json'))) {
      walk(JSON.parse(readFileSync(path.join(dir, file), 'utf8')));
    }
    return found;
  }

  it.each(LANGUAGES)('%s: no working is left without a message', (language) => {
    const missing: string[] = [];
    for (const [id, names] of workingIds()) {
      // A per-نگ charge of a lot names the quantity received: the screen reads its "-received" form.
      const wanted = id.endsWith('per-unit') && names.has('qty_received') ? `${id}-received` : id;
      try {
        // Every value the backend gives is passed, so a variable the message uses and the
        // backend does not send shows as an open placeholder.
        const args = Object.fromEntries([...names].map((name) => [name, '1']));
        const text = message(language, wanted, args);
        if (/[{}$]/.test(text)) missing.push(`${wanted}: ${text}`);
      } catch {
        missing.push(`${wanted}: no message`);
      }
    }
    expect(missing).toEqual([]);
  });
});
