// The titles and the words of the party slips (plan 78 A2, A3, A6, PRD FR-19.12, FR-19.15), in Urdu and in
// English: a slip for one party carries his name in its title ("اکرم کی روزانہ پرچی", "Day slip: Akram"), a
// seller's bill has that title in the header of its own sheet and no second one inside, a buyer's slip stays
// several to a page, the party's code (not his name again) is the line under the title, and a slip that names
// no one keeps the plain title. The words and figures are the backend's: this file checks only where the
// layout puts them.

import { render } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { DaySlipsPrintData } from '../../src/bindings/DaySlipsPrintData';
import type { Language } from '../../src/bindings/Language';
import type { PrintReportParty } from '../../src/bindings/PrintReportParty';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { DaySlipsDocument, slipTitle } from '../../src/features/reports/print/DaySlipsDocument';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { daySlipsData } from '../helpers/day-slips';
import { message, visible } from '../helpers/render';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.style.removeProperty(FOOTER_PROPERTY);
});

/** The first slip of a list; a test that has none is wrong. */
function firstOf<T>(list: readonly T[]): T {
  const [first] = list;
  if (first === undefined) throw new Error('the print has no slip');
  return first;
}

const PARTY: PrintReportParty = {
  short_code: '0',
  code: '0020',
  name: 'Akram Toll Tax',
  tag: null,
  person_key: null,
};
const NAME_UR = 'اکرم ٹول ٹیکس';

/** The slip of the day for one party, named in its title the way the backend sends a one-party print. */
function named(
  data: ReportPrintData,
  language: Language,
  titleId: string,
  tag: string | null = null,
): ReportPrintData {
  return {
    ...data,
    party: { ...PARTY, name: language === 'ur' ? NAME_UR : PARTY.name, tag },
    named_title_id: titleId,
  };
}

function draw(data: DaySlipsPrintData, variant: 'parchi' | 'seller-bills') {
  useLanguageStore.setState({ language: data.frame.language });
  const result = render(<DaySlipsDocument data={data} variant={variant} />);
  return result.container;
}

describe.each(['en', 'ur'] as const)('the title of a party slip (%s)', (language) => {
  const m = (id: string, vars: Record<string, string> = {}) => message(language, id, vars);
  const name = language === 'ur' ? NAME_UR : PARTY.name;

  it('is written with the party’s name in his own words: "Day slip: Akram Toll Tax"', () => {
    const base = daySlipsData(language, 'parchi');
    const buyer = named(firstOf(base.buyers), language, 'print-buyer-day-slip-title');
    const page = draw({ ...base, buyers: [buyer] }, 'parchi');
    const title = page.querySelector('.print-slip-title');
    expect(visible(title?.textContent)).toBe(m('print-buyer-day-slip-title', { name }));
    // The name is in the title, so the line under it holds his code and not his name a second time.
    const facts = [...page.querySelectorAll('.print-fact dd')].map((dd) => visible(dd.textContent));
    expect(facts).toContain('0');
    expect(facts.some((fact) => fact.includes(name))).toBe(false);
  });

  it('puts the tag after the name', () => {
    const base = daySlipsData(language, 'parchi');
    const buyer = named(firstOf(base.buyers), language, 'print-buyer-day-slip-title', 'K');
    const page = draw({ ...base, buyers: [buyer] }, 'parchi');
    expect(visible(page.querySelector('.print-slip-title')?.textContent)).toBe(
      m('print-buyer-day-slip-title', { name: `${name} K` }),
    );
  });

  it('is left out when the backend names no one: the slip keeps its plain heading and its facts', () => {
    const real = daySlipsData(language, 'parchi');
    // The real slip now names its party; this one is the same slip with no name given.
    const base = {
      ...real,
      buyers: real.buyers.map((buyer) => ({ ...buyer, party: null, named_title_id: null })),
    };
    const page = draw(base, 'parchi');
    expect(page.querySelector('.print-slip-title')).toBeNull();
    const facts = [...page.querySelectorAll('.print-fact dd')].map((dd) => visible(dd.textContent));
    expect(facts.some((fact) => fact.includes(name))).toBe(true);
  });

  it('is a function of the id and the name only: slipTitle gives the words, or null for no name', () => {
    const base = daySlipsData(language, 'seller-bills');
    const seller = named(firstOf(base.sellers), language, 'print-seller-bill-named-title');
    const getString = (id: string, vars: Record<string, string>) => message(language, id, vars);
    expect(slipTitle(seller, getString)).toBe(m('print-seller-bill-named-title', { name }));
    expect(slipTitle({ named_title_id: null, party: PARTY }, getString)).toBeNull();
    expect(
      slipTitle({ named_title_id: 'print-seller-bill-named-title', party: null }, getString),
    ).toBeNull();
  });

  it('a seller’s bill has the title in the header of its own sheet, once, one sheet for each seller', () => {
    const base = daySlipsData(language, 'seller-bills');
    const first = named(firstOf(base.sellers), language, 'print-seller-bill-named-title');
    const second = named({ ...firstOf(base.sellers) }, language, 'print-seller-bill-named-title');
    const page = draw(
      { ...base, sellers: [first, { ...second, party: { ...PARTY, name: 'Rahim' } }] },
      'seller-bills',
    );
    const sheets = [...page.querySelectorAll('.print-sheet')];
    expect(sheets).toHaveLength(2);
    expect(visible(sheets[0]?.querySelector('.print-header')?.textContent)).toContain(
      m('print-seller-bill-named-title', { name }),
    );
    // The title is in the header, so the slip inside does not write it again.
    expect(page.querySelector('.print-slip-title')).toBeNull();
    expect(visible(sheets[1]?.querySelector('.print-header')?.textContent)).toContain(
      m('print-seller-bill-named-title', { name: 'Rahim' }),
    );
  });

  it('a buyer’s slip says "Cash Paid In" and a seller’s says نقد ادائیگی (Cash Paid), with the sides written for the reader', () => {
    const buyers = draw(daySlipsData(language, 'parchi'), 'parchi');
    const buyerLabels = [...buyers.querySelectorAll('.print-slip-lines th')].map((th) =>
      visible(th.textContent),
    );
    expect(buyerLabels).toContain(m('report-r35-paid-today'));
    expect(visible(buyers.textContent)).toContain(m('print-slip-side-dr'));
    const sellers = draw(daySlipsData(language, 'seller-bills'), 'seller-bills');
    const sellerLabels = [...sellers.querySelectorAll('.print-slip-lines th')].map((th) =>
      visible(th.textContent),
    );
    expect(sellerLabels).toContain(m('report-r35-paid-today-seller'));
    // The seller's cash paid is the glossary term نقد ادائیگی (plan 95, PRD FR-12.52); the buyer's line keeps its own words.
    expect(m('report-r35-paid-today-seller')).toBe(m('glossary-naqd-adaigi'));
    expect(m('report-r35-paid-today')).not.toBe(m('glossary-naqd-adaigi'));
    expect(sellerLabels).toContain(m('report-r35-today-left-seller'));
    expect(sellerLabels).not.toContain(m('report-r35-paid-today'));
    expect(visible(sellers.textContent)).toContain(m('print-slip-side-cr'));
    // The ledger's book words are not on a slip. The one line whose label is Umar's own wording ("... credited to
    // his account", PRD FR-12.23 v1.53) is set aside: it uses the plain verb, never the side of a balance.
    const withoutRest = sellers.cloneNode(true) as HTMLElement;
    for (const row of withoutRest.querySelectorAll('.print-slip-lines tr')) {
      if (visible(row.querySelector('th')?.textContent) === m('report-r35-sale-rest')) row.remove();
    }
    expect(visible(withoutRest.textContent)).not.toContain(m('report-side-cr'));
  });
});
