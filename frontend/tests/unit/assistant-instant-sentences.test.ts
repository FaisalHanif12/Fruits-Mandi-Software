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
  // Plan 59 F3 batch 1: how many Nag, the average rate, expenses, receipts, the cash book, sales by the way
  // they were paid for, a seller's net sale, fruit taken and the capital vouchers.
  {
    id: 'assistant-instant-qty',
    selector: 'role',
    values: ['shop', 'item', 'seller', 'buyer', 'seller-item', 'buyer-item'],
    uses: {
      shop: ['period', 'figure'],
      item: ['name', 'period', 'figure'],
      seller: ['name', 'period', 'figure'],
      buyer: ['name', 'period', 'figure'],
    },
    common: ['name', 'item', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-rate-average',
    selector: 'role',
    values: ['item', 'seller-item'],
    uses: { item: ['item', 'period', 'figure'] },
    common: ['name', 'item', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-expense',
    selector: 'kind',
    values: ['shop', 'head'],
    uses: { shop: ['period', 'figure'] },
    common: ['name', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-receipts',
    selector: 'role',
    values: ['shop', 'collector'],
    uses: { shop: ['period', 'figure'] },
    common: ['name', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-cash-book',
    selector: 'kind',
    values: ['closing', 'in', 'out'],
    common: ['period', 'figure'],
  },
  {
    id: 'assistant-instant-sale-kind',
    selector: 'kind',
    values: ['cash', 'credit'],
    common: ['period', 'figure'],
  },
  {
    id: 'assistant-instant-sale-net',
    selector: '',
    values: [''],
    common: ['name', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-fruit',
    selector: 'role',
    values: ['shop', 'partner', 'guests'],
    uses: { shop: ['period', 'figure'], guests: ['period', 'figure'] },
    common: ['name', 'period', 'figure'],
  },
  {
    id: 'assistant-instant-capital-flow',
    selector: 'kind',
    values: ['in', 'out'],
    common: ['period', 'figure'],
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

describe.each(LANGUAGES)('the instant sentences of plan 59 batch 1 (%s)', (language) => {
  const ARGS = {
    name: 'NAMEMARK',
    item: 'ITEMMARK',
    figure: 'FIGUREMARK',
    period: 'PERIODMARK',
  };
  const say = (id: string, selected: Record<string, string> = {}) =>
    message(language, id, { ...ARGS, ...selected });

  it('the Nag sentences use the glossary Nag, and end with the figure after the party, the item and the period', () => {
    const nag = message(language, 'glossary-nag');
    for (const role of ['item', 'seller', 'buyer', 'seller-item', 'buyer-item', 'shop']) {
      const text = say('assistant-instant-qty', { role });
      expect(text, role).toContain(nag);
      expect(text.indexOf('PERIODMARK'), role).toBeLessThan(text.indexOf('FIGUREMARK'));
    }
    // English puts the item first ("Nag of Banana from Seller Rahim"), Urdu the party first; either way the
    // party and the item both come before the figure.
    for (const role of ['seller-item', 'buyer-item']) {
      const text = say('assistant-instant-qty', { role });
      expect(text.indexOf('NAMEMARK'), role).toBeLessThan(text.indexOf('FIGUREMARK'));
      expect(text.indexOf('ITEMMARK'), role).toBeLessThan(text.indexOf('FIGUREMARK'));
    }
  });

  it('the average rate says average, and not last: it is a different figure from the last rate', () => {
    const average = language === 'ur' ? 'اوسط' : 'Average';
    const last = language === 'ur' ? 'آخری' : 'Last';
    for (const role of ['item', 'seller-item']) {
      const text = say('assistant-instant-rate-average', { role });
      expect(text, role).toContain(average);
      expect(text, role).not.toContain(last);
    }
  });

  it('the expense sentences are the shop expense and never a partner drawing', () => {
    const shop = message(language, 'glossary-dukan-kharcha');
    const drawing = message(language, 'glossary-zati-kharcha');
    for (const kind of ['shop', 'head']) {
      const text = say('assistant-instant-expense', { kind });
      expect(text, kind).toContain(shop);
      expect(text, kind).not.toContain(drawing);
    }
  });

  it('the receipts sentences name the Receipt; the collector one names him', () => {
    const receipt = message(language, 'glossary-wasooli');
    for (const role of ['shop', 'collector']) {
      expect(say('assistant-instant-receipts', { role }), role).toContain(receipt);
    }
    expect(say('assistant-instant-receipts', { role: 'collector' })).toContain('NAMEMARK');
    expect(say('assistant-instant-receipts', { role: 'shop' })).not.toContain('NAMEMARK');
  });

  it('the cash book sentences all name the Cash Book, and say received, paid out or closing', () => {
    const book = message(language, 'glossary-rokar');
    const texts = ['closing', 'in', 'out'].map((kind) =>
      say('assistant-instant-cash-book', { kind }),
    );
    for (const text of texts) expect(text).toContain(book);
    expect(new Set(texts).size).toBe(3);
  });

  it('the sale kind sentences use the glossary words for Cash and Credit and the Sale', () => {
    const sale = message(language, 'glossary-bikri');
    const cash = say('assistant-instant-sale-kind', { kind: 'cash' });
    const credit = say('assistant-instant-sale-kind', { kind: 'credit' });
    expect(cash).toContain(message(language, 'glossary-naqdi'));
    expect(credit).toContain(message(language, 'glossary-udhari'));
    expect(cash).toContain(sale);
    expect(credit).toContain(sale);
  });

  it('the seller net sentence is the Net Sale of the seller, name before figure', () => {
    const text = say('assistant-instant-sale-net');
    expect(text).toContain(message(language, 'glossary-safi-bikri'));
    expect(text.indexOf('NAMEMARK')).toBeLessThan(text.indexOf('FIGUREMARK'));
  });

  it('the fruit sentences are the Fruit Taken with its value; the guests one names the Guests and no partner', () => {
    const fruit = message(language, 'glossary-phal');
    for (const role of ['shop', 'partner', 'guests']) {
      expect(say('assistant-instant-fruit', { role }), role).toContain(fruit);
    }
    expect(say('assistant-instant-fruit', { role: 'guests' })).toContain(
      message(language, 'glossary-mehman'),
    );
    expect(say('assistant-instant-fruit', { role: 'guests' })).not.toContain('NAMEMARK');
    expect(say('assistant-instant-fruit', { role: 'partner' })).toContain('NAMEMARK');
  });

  it('the capital flow sentences are the Capital, put in or taken back, never a drawing', () => {
    const capital = message(language, 'glossary-sarmaya');
    const inText = say('assistant-instant-capital-flow', { kind: 'in' });
    const outText = say('assistant-instant-capital-flow', { kind: 'out' });
    expect(inText).toContain(capital);
    expect(outText).toContain(capital);
    expect(inText).not.toBe(outText);
  });
});

describe('the period in the new Urdu sentences', () => {
  const PERIODS = ['today', 'day', 'range', 'all'] as const;
  const NEW_SENTENCES = [
    [
      'assistant-instant-qty',
      'role',
      ['shop', 'item', 'seller', 'buyer', 'seller-item', 'buyer-item'],
    ],
    ['assistant-instant-rate-average', 'role', ['item', 'seller-item']],
    ['assistant-instant-expense', 'kind', ['shop', 'head']],
    ['assistant-instant-receipts', 'role', ['shop', 'collector']],
    ['assistant-instant-cash-book', 'kind', ['closing', 'in', 'out']],
    ['assistant-instant-sale-kind', 'kind', ['cash', 'credit']],
    ['assistant-instant-fruit', 'role', ['shop', 'partner', 'guests']],
    ['assistant-instant-capital-flow', 'kind', ['in', 'out']],
  ] as const;

  it.each(PERIODS)('period %s is followed by کی or کا in every new line', (shape) => {
    const period = message('ur', 'assistant-instant-period', {
      shape,
      day: 'DAYMARK',
      from: 'FROMMARK',
      to: 'TOMARK',
    });
    for (const [id, selector, values] of NEW_SENTENCES) {
      for (const value of values) {
        const text = message('ur', id, {
          [selector]: value,
          name: 'NAMEMARK',
          item: 'ITEMMARK',
          figure: 'FIGUREMARK',
          period,
        });
        expect(text, `${id}/${value}`).toMatch(new RegExp(`${period} (کی|کا) `));
      }
    }
    const net = message('ur', 'assistant-instant-sale-net', {
      name: 'NAMEMARK',
      figure: 'FIGUREMARK',
      period,
    });
    expect(net).toMatch(new RegExp(`${period} کی `));
  });
});
