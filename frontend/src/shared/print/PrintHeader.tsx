// The top of every printed sheet (PRD FR-19.13, Section 21 Firm): the firm name with its phones under it
// (up to four, plan 100 H), the mandi, the firm's address, the trade mark in a box, the people of the
// firm (up to eight: the owners first, then the munshi, each with his role word and, when he has one, his phone), then the title of
// the document and the facts under it (number, day, filters). The firm's picture (a fruit crate, PRD
// FR-19.13, plan 76) is at the physical left edge and the people of the firm at the physical right
// edge, in both print languages, with the firm name between them (the order in print.css). It is ONE header for
// every print of the app, the built ones and the reports of D5: it is given the header the backend
// built (`PrintHeader` of the bindings, from `reports_print_*` or `reports_print_header`) and draws it.
// A contact or a trade mark that is not set is not there: no empty line, no empty box. Every value
// is data in the print language; every label is a Fluent message id resolved in the print language.

import { useLocalization } from '@fluent/react';
import { Code } from '../ui/Code';
import { PrintFruitMark } from './PrintFruitMark';
import { PrintText } from './PrintText';
import { LAYOUT_IDS, type FirmRole, type PrintHeaderData } from './types';

/** The Fluent message of the word before a person's name in the header. */
const ROLE_ID: Record<FirmRole, string> = {
  owner: 'print-role-owner',
  munshi: 'print-role-munshi',
};

interface PrintHeaderProps {
  header: PrintHeaderData;
  /**
   * The title as words, when the document's own title names a party ("<name> کا بل", plan 78 A2, A3): the
   * caller resolves the message with the party's name. Left out, the title is the message of the header.
   */
  title?: string | undefined;
}

/** The titles that take the party's name (`$name`): the print window must give it, or the title reads wrong. */
export const NAMED_TITLE_IDS: ReadonlySet<string> = new Set([
  'print-buyer-day-slip-title',
  'print-seller-day-slip-title',
  'print-seller-bill-named-title',
  'print-statement-named-title',
]);

/**
 * The people of the firm, the owners first and then the munshi, each with his role word and his phone (Section 21).
 * `showRole` false leaves the role word out, where the place already says it (the owners under the firm name on a
 * parchi).
 */
export function PrintContacts({
  contacts,
  showRole = true,
}: {
  contacts: PrintHeaderData['contacts'];
  showRole?: boolean;
}) {
  const { l10n } = useLocalization();
  if (contacts.length === 0) return null;
  return (
    <ul className="print-contacts">
      {contacts.map((person, index) => (
        <li key={index} className="print-firm-line">
          {/* The backend sends the owners first, then the munshi; the word of the role is ours. */}
          {showRole ? (
            <>
              <span className="print-contact-role">
                {l10n.getString(ROLE_ID[person.role])}
              </span>{' '}
            </>
          ) : null}
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
  );
}

/** The firm's phones on one line under its name, each whole (a phone is one code); nothing when it has none. */
function PrintFirmPhones({ phones }: { phones: readonly string[] }) {
  if (phones.length === 0) return null;
  return (
    <p className="print-firm-line print-firm-phones">
      {phones.map((phone, index) => (
        <Code key={index}>{phone}</Code>
      ))}
    </p>
  );
}

/**
 * The top band of a buyer slip (PRD FR-19.15, plan 79 item 7, plan 100 H): the fruit picture on one side;
 * in the middle the firm name with the owners' names and phones under it; on the other side the munshi's
 * name with his phone; under the three, the firm's phones across the slip. The same three zones and the same edges as the header of every print
 * (`print-zones`): the two sides are equally wide, so the firm name is in the middle of the slip, with or
 * without a munshi. Smaller than the header of a page, so a slip keeps its size.
 */
export function PrintSlipBand({ header }: { header: PrintHeaderData }) {
  const owners = header.contacts.filter((person) => person.role === 'owner');
  const staff = header.contacts.filter((person) => person.role !== 'owner');
  return (
    <div className="print-slip-head">
      <div className="print-zones print-slip-band">
        <div className="print-slip-side print-slip-picture">
          <PrintFruitMark />
        </div>
        <div className="print-firm">
          <p className="print-firm-name">{header.firm.name}</p>
          <PrintContacts contacts={owners} showRole={false} />
        </div>
        <div className="print-slip-side print-slip-people">
          <PrintContacts contacts={staff} />
        </div>
      </div>
      {/* The firm's own phones have the whole width of the slip, two to a line, so they never run into a side. */}
      <PrintFirmPhones phones={header.firm.phones} />
    </div>
  );
}

export function PrintHeader({ header, title }: PrintHeaderProps) {
  const { l10n } = useLocalization();
  const { firm } = header;
  // The address and the phones are as typed in Settings (PRD FR-7.25); either may be missing. A phone is a
  // code (`Code`) and any number inside the address is a number (`PrintText`), so no digit of these
  // lines is drawn in the font of the words (NFR-URD-14).
  return (
    <>
      <header className="print-header">
        <div className="print-zones">
          <PrintFruitMark />
          <div className="print-firm">
            <div className="print-firm-top">
              <p className="print-firm-name">{firm.name}</p>
              {header.trade_mark === null ? null : (
                <p className="print-trade-mark" aria-label={l10n.getString(LAYOUT_IDS.tradeMark)}>
                  <bdi dir="auto">{header.trade_mark}</bdi>
                </p>
              )}
            </div>
            <PrintFirmPhones phones={firm.phones} />
            {firm.mandi === '' ? null : (
              <p className="print-firm-line">
                <PrintText text={firm.mandi} />
              </p>
            )}
            {firm.address.trim() === '' ? null : (
              <p className="print-firm-line">
                <PrintText text={firm.address} />
              </p>
            )}
          </div>
          <PrintContacts contacts={header.contacts} />
        </div>
        <h1 className="print-title">
          {title === undefined ? l10n.getString(header.title_id) : <PrintText text={title} />}
        </h1>
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
