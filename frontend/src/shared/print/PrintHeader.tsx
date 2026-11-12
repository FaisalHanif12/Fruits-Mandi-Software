// The top of every printed sheet (PRD FR-19.13, Section 21 Firm): the firm name and the mandi, the
// firm's address and phone, the trade mark in a box, the people of the firm (up to six: the owners
// first, then the munshi, each with his role word and, when he has one, his phone), then the title of
// the document and the facts under it (number, day, filters). It is ONE header for
// every print of the app, the built ones and the reports of D5: it is given the header the backend
// built (`PrintHeader` of the bindings, from `reports_print_*` or `reports_print_header`) and draws it.
// A contact or a trade mark that is not set is not there: no empty line, no empty box. Every value
// is data in the print language; every label is a Fluent message id resolved in the print language.

import { useLocalization } from '@fluent/react';
import { Fragment, type ReactNode } from 'react';
import { Code } from '../ui/Code';
import { PrintText } from './PrintText';
import { LAYOUT_IDS, type FirmRole, type PrintHeaderData } from './types';

/** The Fluent message of the word before a person's name in the header. */
const ROLE_ID: Record<FirmRole, string> = {
  owner: 'print-role-owner',
  munshi: 'print-role-munshi',
};

export function PrintHeader({ header }: { header: PrintHeaderData }) {
  const { l10n } = useLocalization();
  const { firm } = header;
  const parts: ReactNode[] = [];
  if (firm.address.trim() !== '') parts.push(<PrintText text={firm.address} />);
  if (firm.phone.trim() !== '') parts.push(<Code>{firm.phone}</Code>);
  // Address and phone are as typed in Settings (PRD FR-7.25); either may be empty. The phone is a
  // code (`Code`) and any number inside the address is a number (`PrintText`), so no digit of this
  // line is drawn in the font of the words (NFR-URD-14).
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
          {firm.mandi === '' ? null : (
            <p className="print-firm-line">
              <PrintText text={firm.mandi} />
            </p>
          )}
          {parts.length === 0 ? null : (
            <p className="print-firm-line">
              {parts.map((part, index) => (
                <Fragment key={index}>
                  {index > 0 ? ' · ' : null}
                  {part}
                </Fragment>
              ))}
            </p>
          )}
          {header.contacts.length === 0 ? null : (
            <ul className="print-contacts">
              {header.contacts.map((person, index) => (
                <li key={index} className="print-firm-line">
                  {/* The backend sends the owners first, then the munshi; the word of the role is ours. */}
                  <span className="print-contact-role">{l10n.getString(ROLE_ID[person.role])}</span>{' '}
                  <PrintText text={person.name} />
                  {/* A person with no phone is the name only: no empty gap, no dash. */}
                  {person.phone === '' ? null : (
                    <>
                      {' '}
                      <Code>{person.phone}</Code>
                    </>
                  )}
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
                {/* A fact with Urdu letters in it is a party's name with his code or a note: its words are in the
                    text font (Gulzar, not the Inter of a code, which has no Urdu letters) and every number
                    in it is a number (`PrintText`). Any other fact is a date, a number, a phone, a truck
                    number or an English name: one code, written left to right (Urdu check of the prints). */}
                {/\p{Script=Arabic}/u.test(fact.value) ? (
                  <PrintText text={fact.value} />
                ) : (
                  <Code>{fact.value}</Code>
                )}
              </dd>
            </div>
          ))}
        </dl>
      )}
    </>
  );
}
