// The top of every sheet: the firm at the start, the title of the document at the end, and the
// facts under them (number, date, truck). Every value is data from the backend, already in the
// print language; every label is a Fluent message id resolved in the print language.

import { useLocalization } from '@fluent/react';
import { Code } from '../ui/Code';
import type { PrintData } from './types';

export function PrintHeader({ data }: { data: PrintData }) {
  const { l10n } = useLocalization();
  const { firm } = data;
  // Address and phone are as typed in Settings (PRD FR-7.25); either may be empty.
  const contact = [firm.address, firm.phone].filter((part) => part.trim() !== '');
  return (
    <>
      <header className="print-header">
        <div className="print-firm">
          <p className="print-firm-name">{firm.name}</p>
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
        </div>
        <h1 className="print-title">{l10n.getString(data.title_id)}</h1>
      </header>
      <dl className="print-facts">
        {data.facts.map((fact) => (
          <div key={fact.label_id} className="print-fact">
            <dt>{l10n.getString(fact.label_id)}</dt>
            <dd>
              <Code>{fact.value}</Code>
            </dd>
          </div>
        ))}
      </dl>
    </>
  );
}
