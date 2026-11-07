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
    values: ['shop', 'item', 'seller', 'seller-own', 'buyer'],
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
  // The sentences of a figure on one item for one party, and the last rate of an item (PRD FR-7.90).
  {
    id: 'assistant-instant-commission-buyer-item',
    selector: '',
    values: [''],
    common: ['name', 'item', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-commission-seller-item-own',
    selector: '',
    values: [''],
    common: ['name', 'item', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-commission-seller-item',
    selector: '',
    values: [''],
    common: ['name', 'item', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-laga-buyer-item',
    selector: '',
    values: [''],
    common: ['name', 'item', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-last-rate-item',
    selector: '',
    values: [''],
    common: ['item', 'period', 'figure', 'date'],
  },
  {
    id: 'assistant-instant-last-rate-buyer-item',
    selector: '',
    values: [''],
    common: ['name', 'item', 'period', 'figure', 'date'],
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
  {
    id: 'assistant-instant-partner-capital',
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
  item: 'ITEMMARK',
  date: 'DATEMARK',
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

  it('the commission of the shop or an item says it includes Laga; the buyer line does not', () => {
    const laga = message(language, 'glossary-laga');
    const args = { name: 'NAMEMARK', figure: 'FIGUREMARK', period: 'PERIODMARK' };
    // The total on a seller's goods includes Laga too; what is charged to a seller or a buyer does not.
    for (const role of ['shop', 'item', 'seller']) {
      expect(message(language, 'assistant-instant-commission', { role, ...args })).toContain(laga);
    }
    for (const role of ['seller-own', 'buyer']) {
      expect(message(language, 'assistant-instant-commission', { role, ...args })).not.toContain(
        laga,
      );
    }
  });

  it('the total on a seller goods says it holds the line above, so the two figures cannot be added', () => {
    const args = { name: 'NAMEMARK', figure: 'FIGUREMARK', period: 'PERIODMARK' };
    const own = message(language, 'assistant-instant-commission', { role: 'seller-own', ...args });
    const total = message(language, 'assistant-instant-commission', { role: 'seller', ...args });
    expect(total).toContain(language === 'ur' ? 'اوپر والی لائن سمیت' : 'including the line above');
    expect(own).not.toContain(language === 'ur' ? 'اوپر والی لائن' : 'line above');
    // The two lines are about the same seller and period, and differ in what the figure is.
    for (const text of [own, total]) {
      expect(text).toContain('NAMEMARK');
      expect(text).toContain('PERIODMARK');
    }
    expect(own).not.toBe(total);
  });

  it('the item sentences place the item after the party, and only the seller total includes Laga', () => {
    const laga = message(language, 'glossary-laga');
    const args = { name: 'NAMEMARK', item: 'ITEMMARK', figure: 'FIGUREMARK', period: 'PERIODMARK' };
    const buyer = message(language, 'assistant-instant-commission-buyer-item', args);
    const own = message(language, 'assistant-instant-commission-seller-item-own', args);
    const total = message(language, 'assistant-instant-commission-seller-item', args);
    for (const text of [buyer, own, total]) {
      expect(text.indexOf('NAMEMARK')).toBeLessThan(text.indexOf('ITEMMARK'));
      expect(text.indexOf('ITEMMARK')).toBeLessThan(text.indexOf('FIGUREMARK'));
    }
    expect(total).toContain(laga);
    expect(total).toContain(language === 'ur' ? 'اوپر والی لائن سمیت' : 'including the line above');
    expect(buyer).not.toContain(laga);
    expect(own).not.toContain(laga);
    expect(own).not.toContain(language === 'ur' ? 'اوپر والی لائن' : 'line above');
  });

  it('the last rate sentences end with the figure, then the date', () => {
    const args = {
      name: 'NAMEMARK',
      item: 'ITEMMARK',
      figure: 'FIGUREMARK',
      period: 'PERIODMARK',
      date: 'DATEMARK',
    };
    for (const id of [
      'assistant-instant-last-rate-item',
      'assistant-instant-last-rate-buyer-item',
    ]) {
      const text = message(language, id, args);
      expect(text.indexOf('FIGUREMARK'), id).toBeLessThan(text.indexOf('DATEMARK'));
      expect(text.indexOf('ITEMMARK'), id).toBeLessThan(text.indexOf('FIGUREMARK'));
    }
  });

  it('the capital sentence is the partner and his capital with the figure', () => {
    const text = message(language, 'assistant-instant-partner-capital', {
      name: 'NAMEMARK',
      figure: 'FIGUREMARK',
    });
    expect(text).toContain(message(language, 'glossary-sarmaya'));
    expect(text.indexOf('NAMEMARK')).toBeLessThan(text.indexOf('FIGUREMARK'));
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

describe('the word order of the period in the sentences', () => {
  const PERIODS = ['today', 'day', 'range', 'all'] as const;
  /** Every sale, commission and Laga sentence puts the period before کی or کا, whatever the period is. */
  const WITH_PERIOD = [
    ['assistant-instant-sale', ['shop', 'item', 'seller', 'buyer']],
    ['assistant-instant-sale-count', ['shop', 'seller']],
    ['assistant-instant-commission', ['shop', 'item', 'seller', 'seller-own', 'buyer']],
    ['assistant-instant-laga', ['shop', 'item', 'buyer']],
  ] as const;

  it.each(PERIODS)(
    'period %s is followed by کی or کا in every sale, commission and Laga line',
    (shape) => {
      const period = message('ur', 'assistant-instant-period', {
        shape,
        day: 'DAYMARK',
        from: 'FROMMARK',
        to: 'TOMARK',
      });
      for (const [id, roles] of WITH_PERIOD) {
        for (const role of roles) {
          const text = message('ur', id, { role, name: 'NAMEMARK', figure: 'FIGUREMARK', period });
          expect(text, `${id}/${role}`).toMatch(new RegExp(`${period} (کی|کا) `));
        }
      }
    },
  );

  it('the buyer line never has two "سے" in a row around the period (a range has its own سے)', () => {
    const range = message('ur', 'assistant-instant-period', {
      shape: 'range',
      from: 'FROMMARK',
      to: 'TOMARK',
    });
    for (const id of ['assistant-instant-commission', 'assistant-instant-laga']) {
      const text = message('ur', id, {
        role: 'buyer',
        name: 'NAMEMARK',
        figure: 'FIGUREMARK',
        period: range,
      });
      expect(text).not.toMatch(/سے\s+FROMMARK/);
      expect(text).toContain('NAMEMARK سے لیا گیا');
    }
  });

  it.each(PERIODS)(
    'period %s is followed by کا in every new item and last-rate line (Urdu)',
    (shape) => {
      const period = message('ur', 'assistant-instant-period', {
        shape,
        day: 'DAYMARK',
        from: 'FROMMARK',
        to: 'TOMARK',
      });
      const ids = [
        'assistant-instant-commission-buyer-item',
        'assistant-instant-commission-seller-item-own',
        'assistant-instant-commission-seller-item',
        'assistant-instant-laga-buyer-item',
        'assistant-instant-last-rate-item',
        'assistant-instant-last-rate-buyer-item',
      ];
      for (const id of ids) {
        const text = message('ur', id, {
          name: 'NAMEMARK',
          item: 'ITEMMARK',
          figure: 'FIGUREMARK',
          period,
          date: 'DATEMARK',
        });
        expect(text, id).toMatch(new RegExp(`${period} کا `));
      }
    },
  );

  it.each(PERIODS)('in English the period comes after the noun, for period %s', (shape) => {
    const period = message('en', 'assistant-instant-period', {
      shape,
      day: 'DAYMARK',
      from: 'FROMMARK',
      to: 'TOMARK',
    });
    const text = message('en', 'assistant-instant-commission', {
      role: 'shop',
      name: 'NAMEMARK',
      figure: 'FIGUREMARK',
      period,
    });
    expect(text).toBe(`Commission (with ${message('en', 'glossary-laga')}) ${period}: FIGUREMARK`);
  });
});
