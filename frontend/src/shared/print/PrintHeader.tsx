// The top of every printed sheet (PRD FR-19.13, Section 21 Firm): the firm name and the mandi, the
// firm's address and phone, up to three contact persons with their phones, the trade mark in a box,
// then the title of the document and the facts under it (number, day, filters). It is ONE header for
// every print of the app, the built ones and the reports of D5: it is given the header the backend
// built (`PrintHeader` of the bindings, from `reports_print_*` or `reports_print_header`) and draws it.
// A contact or a trade mark that is not set is not there: no empty line, no empty box. Every value
// is data in the print language; every label is a Fluent message id resolved in the print language.

import { useLocalization } from '@fluent/react';
import { Code } from '../ui/Code';
import { LAYOUT_IDS, type PrintHeaderData } from './types';

export function PrintHeader({ header }: { header: PrintHeaderData }) {
  const { l10n } = useLocalization();
  const { firm } = header;
  // Address and phone are as typed in Settings (PRD FR-7.25); either may be empty.
  const contact = [firm.address, firm.phone].filter((part) => part.trim() !== '');
  return (
    <>
      <header className="print-header">
        <div className="print-firm">
          <div className="print-firm-top">
            <p className="print-firm-name">{firm.name}</p>
            {header.trade_mark === null ? null : (
              <p className="print-trade-mark" aria-label={l10n.getString(LAYOUT_IDS.tradeMark)}>
                <bdi dir="auto">{header.trade_mark}</bdi>
              </p>
            )}
          </div>
          {firm.mandi === '' ? null : <p className="print-firm-line">{firm.mandi}</p>}
          {contact.length === 0 ? null : (
            <p className="print-firm-line">
              {contact.map((part, index) => (
                <span key={part}>
                  {index > 0 ? ' · ' : null}
                  <bdi dir="auto">{part}</bdi>
                </span>
              ))}
            </p>
          )}
          {header.contacts.length === 0 ? null : (
            <ul className="print-contacts">
              {header.contacts.map((person, index) => (
                <li key={index} className="print-firm-line">
                  <bdi dir="auto">{person.name}</bdi>{' '}
                  {person.phone === '' ? null : <Code>{person.phone}</Code>}
                </li>
              ))}
            </ul>
          )}
        </div>
        <h1 className="print-title">{l10n.getString(header.title_id)}</h1>
      </header>
      {header.facts.length === 0 ? null : (
        <dl className="print-facts">
          {header.facts.map((fact) => (
            <div key={fact.label_id} className="print-fact">
              <dt>{l10n.getString(fact.label_id)}</dt>
              <dd>
                <Code>{fact.value}</Code>
              </dd>
            </div>
          ))}
        </dl>
      )}
    </>
  );
}
