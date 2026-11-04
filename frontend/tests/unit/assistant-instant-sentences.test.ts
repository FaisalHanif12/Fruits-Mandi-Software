// The sentences of an instant answer (PRD FR-7.90, plan 55 Section 10): the backend writes an answer to a
// simple figure question from these Fluent messages, with no model call, and fills the arguments itself.
// Each message must read in both languages for every value of its selector, place every argument, and
// hold no digit: a figure, a name or a date arrives as a reference that the app checks and replaces.

import { describe, expect, it } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { message } from '../helpers/render';

/** One message: its id, the selector it reads and the values of that selector, and the other arguments. */
interface Sentence {
  id: string;
  selector: string;
  values: readonly string[];
  /** The arguments each value needs, by value; a value not listed here needs only `common`. */
  uses?: Readonly<Record<string, readonly string[]>>;
  common: readonly string[];
}

const SENTENCES: readonly Sentence[] = [
  {
    id: 'assistant-instant-period',
    selector: 'shape',
    values: ['today', 'day', 'range', 'all'],
    uses: { today: [], day: ['day'], range: ['from', 'to'], all: [] },
    common: [],
  },
  {
    id: 'assistant-instant-sale',
    selector: 'role',
    values: ['shop', 'item', 'seller', 'buyer'],
    uses: { shop: ['period', 'figure'] },
    common: ['name', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-sale-count',
    selector: 'role',
    values: ['shop', 'seller'],
    uses: { shop: ['period', 'figure'] },
    common: ['name', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-commission',
    selector: 'role',
    values: ['shop', 'item', 'seller', 'buyer'],
    uses: { shop: ['period', 'figure'] },
    common: ['name', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-laga',
    selector: 'role',
    values: ['shop', 'item', 'buyer'],
    uses: { shop: ['period', 'figure'] },
    common: ['name', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-party',
    selector: 'kind',
    values: ['balance', 'payable', 'advance'],
    common: ['name', 'figure'],
  },
  {
    id: 'assistant-instant-shop',
    selector: 'kind',
    values: ['cash', 'buyers-dues', 'total-dues', 'payable', 'advances', 'dead', 'lent'],
    common: ['figure'],
  },
  {
    id: 'assistant-instant-stock',
    selector: 'role',
    values: ['shop', 'item'],
    uses: { shop: ['qty', 'lots'] },
    common: ['name', 'qty', 'lots'],
  },
  {
    id: 'assistant-instant-partner-total',
    selector: '',
    values: [''],
    common: ['name', 'figure'],
  },
];

/** A marker for each argument: letters only, so a digit in the text can only come from the message. */
const MARKERS: Readonly<Record<string, string>> = {
  name: 'NAMEMARK',
  figure: 'FIGUREMARK',
  period: 'PERIODMARK',
  day: 'DAYMARK',
  from: 'FROMMARK',
  to: 'TOMARK',
  qty: 'QTYMARK',
  lots: 'LOTSMARK',
};

describe.each(LANGUAGES)('the instant answer sentences (%s)', (language) => {
  const textOf = (sentence: Sentence, value: string): string => {
    const args: Record<string, string> =
      sentence.selector === '' ? {} : { [sentence.selector]: value };
    for (const name of sentence.common) args[name] = MARKERS[name] ?? name;
    args['day'] = MARKERS['day'] ?? '';
    args['from'] = MARKERS['from'] ?? '';
    args['to'] = MARKERS['to'] ?? '';
    return message(language, sentence.id, args);
  };

  it.each(SENTENCES)('$id reads for every value, with its arguments in place', (sentence) => {
    for (const value of sentence.values) {
      const text = textOf(sentence, value);
      expect(text.trim(), `${sentence.id}/${value}`).not.toBe('');
      // The arguments the value needs are all in the text.
      const needed = sentence.uses?.[value] ?? sentence.common;
      for (const name of needed) {
        expect(text, `${sentence.id}/${value} lacks ${name}`).toContain(MARKERS[name] ?? name);
      }
      // No digit, month name or number word is written in a sentence: the answer checks refuse them.
      expect(text, `${sentence.id}/${value}`).not.toMatch(/[0-9۰-۹٠-٩]/);
    }
  });

  it.each(SENTENCES)('$id says something different for each value', (sentence) => {
    const texts = sentence.values.map((value) => textOf(sentence, value));
    expect(new Set(texts).size).toBe(texts.length);
  });

  it('a figure sentence never writes a side word: the figure carries its own', () => {
    for (const sentence of SENTENCES) {
      if (!['assistant-instant-party', 'assistant-instant-shop'].includes(sentence.id)) continue;
      for (const value of sentence.values) {
        const text = textOf(sentence, value);
        expect(text).not.toMatch(/\b(debit|credit)\b/i);
        expect(text).not.toContain('بنام');
        expect(text).not.toContain('جمع');
      }
    }
  });
});
