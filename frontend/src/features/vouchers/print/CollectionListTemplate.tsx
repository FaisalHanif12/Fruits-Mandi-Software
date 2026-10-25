// The collection list on paper (اگراہی فہرست, PRD FR-15.6, FR-7.61, FR-19.13): the buyers planned for
// a day, by collector and then by city, each with his short code, name with tag, phone, dues and
// oldest open bill, an empty column for the collector's note, a total for every group and the total
// OF THIS LIST under the last group. It is NOT the shop's total اگراہی (R20): the list holds only
// the buyers whose collection day is this weekday. It sits under the shared header and footer, and
// lays out what the backend sent: every number, date and total is already formatted, and the
// template adds nothing up (CLAUDE.md rule 1).
//
// On the 80 mm roll there is no room for eight columns: each buyer is two lines.

import { useLocalization } from '@fluent/react';
import type { Ref } from 'react';
import type { CollectionPrintData } from '../../../bindings/CollectionPrintData';
import type { CollectionPrintGroup } from '../../../bindings/CollectionPrintGroup';
import type { CollectionPrintRow } from '../../../bindings/CollectionPrintRow';
import {
  LAYOUT_IDS,
  PrintHeader,
  PrintName,
  PrintRoot,
  PrintScreenFooter,
} from '../../../shared/print';
import { Code } from '../../../shared/ui/Code';
import { Num } from '../../../shared/ui/Num';

interface CollectionListTemplateProps {
  data: CollectionPrintData;
  /** True in the hidden print window (see PrintRoot). */
  applyRoot?: boolean;
  ref?: Ref<HTMLDivElement>;
}

/** The ids of the words of this print, defined in print.ftl in both languages. */
const WORDS = {
  collector: 'print-collection-no-collector',
  city: 'print-collection-no-city',
  groupTotal: 'print-collection-group-total',
  buyers: 'print-collection-buyers',
} as const;

/** The heading of a group: who collects, and in which city. A missing one has its own words. */
function GroupHeading({ group }: { group: CollectionPrintGroup }) {
  const { l10n } = useLocalization();
  return (
    <>
      <bdi dir="auto">{group.collector ?? l10n.getString(WORDS.collector)}</bdi>
      {' · '}
      <bdi dir="auto">{group.city ?? l10n.getString(WORDS.city)}</bdi>
    </>
  );
}

function BuyerName({ row }: { row: CollectionPrintRow }) {
  return <PrintName name={row.name} tag={row.tag} />;
}

function GroupTable({ group }: { group: CollectionPrintGroup }) {
  const { l10n } = useLocalization();
  return (
    <section className="print-collection-group">
      <h2 className="print-collection-group-title">
        <GroupHeading group={group} />
      </h2>
      <table className="print-table print-collection">
        <thead>
          <tr>
            <th scope="col" className="print-num print-col-serial">
              {l10n.getString('print-collection-col-serial')}
            </th>
            <th scope="col" className="print-num print-col-code">
              {l10n.getString('print-collection-col-code')}
            </th>
            <th scope="col" className="print-col-party">
              {l10n.getString('print-collection-col-party')}
            </th>
            <th scope="col" className="print-num print-col-num">
              {l10n.getString('print-collection-col-dues')}
            </th>
            <th scope="col" className="print-num print-col-date">
              {l10n.getString('print-collection-col-oldest')}
            </th>
            <th scope="col" className="print-num print-col-days">
              {l10n.getString('print-collection-col-days')}
            </th>
            <th scope="col" className="print-col-note">
              {l10n.getString('print-collection-col-note')}
            </th>
          </tr>
        </thead>
        <tbody>
          {group.rows.map((row) => (
            <tr key={row.serial}>
              <td className="print-num print-col-serial">
                <Num>{row.serial}</Num>
              </td>
              <td className="print-num print-col-code">
                <Code>{row.code}</Code>
              </td>
              <td>
                <BuyerName row={row} />
                {row.phone === '' ? null : (
                  <div className="print-collection-phone">
                    <Code>{row.phone}</Code>
                  </div>
                )}
              </td>
              <td className="print-num print-col-num">
                <Num>{row.dues}</Num>
              </td>
              <td className="print-num print-col-date">
                {row.oldest_bill_date === null ? null : <Num>{row.oldest_bill_date}</Num>}
              </td>
              <td className="print-num print-col-days">
                {row.oldest_days === null ? null : <Num>{row.oldest_days}</Num>}
              </td>
              <td className="print-col-note" />
            </tr>
          ))}
          <tr className="print-amount-subtotal">
            <th scope="row" colSpan={3}>
              {l10n.getString(WORDS.groupTotal)}
            </th>
            <td className="print-num print-col-num">
              <Num>{group.total}</Num>
            </td>
            <td colSpan={3} />
          </tr>
        </tbody>
      </table>
    </section>
  );
}

/** The roll: a buyer is his serial, name and dues on one line, then his code, phone and age. */
function GroupCompact({ group }: { group: CollectionPrintGroup }) {
  const { l10n } = useLocalization();
  return (
    <section className="print-collection-group">
      <h2 className="print-collection-group-title">
        <GroupHeading group={group} />
      </h2>
      <ul className="print-compact">
        {group.rows.map((row) => (
          <li key={row.serial} className="print-compact-line">
            <div className="print-compact-row">
              <span className="print-compact-name">
                <Num>{row.serial}</Num> <BuyerName row={row} />
              </span>
              <Num>{row.dues}</Num>
            </div>
            <div className="print-compact-sub">
              <Code>{row.code}</Code>
              {row.phone === '' ? null : (
                <>
                  {' · '}
                  <Code>{row.phone}</Code>
                </>
              )}
              {row.oldest_bill_date === null ? null : (
                <>
                  {' · '}
                  <Num>{row.oldest_bill_date}</Num>
                </>
              )}
              {row.oldest_days === null ? null : (
                <>
                  {' · '}
                  <Num>{row.oldest_days}</Num>
                </>
              )}
            </div>
          </li>
        ))}
        <li className="print-compact-line print-amount-subtotal">
          <div className="print-compact-row">
            <span>{l10n.getString(WORDS.groupTotal)}</span>
            <Num>{group.total}</Num>
          </div>
        </li>
      </ul>
    </section>
  );
}

function Body({ data }: { data: CollectionPrintData }) {
  const { l10n } = useLocalization();
  if (data.groups.length === 0) {
    return <p className="print-empty">{l10n.getString(LAYOUT_IDS.empty)}</p>;
  }
  return (
    <>
      {data.groups.map((group, index) =>
        data.paper === 'mm80' ? (
          <GroupCompact key={index} group={group} />
        ) : (
          <GroupTable key={index} group={group} />
        ),
      )}
      <table className="print-amounts print-collection-totals">
        <tbody>
          <tr>
            <th scope="row">{l10n.getString(WORDS.buyers)}</th>
            <td className="print-num">
              <Num>{data.buyer_count}</Num>
            </td>
          </tr>
          <tr className="print-amount-total">
            <th scope="row">{l10n.getString(data.total.label_id)}</th>
            <td className="print-num">
              <Num>{data.total.amount}</Num>
            </td>
          </tr>
        </tbody>
      </table>
    </>
  );
}

export function CollectionListTemplate({
  data,
  applyRoot = false,
  ref,
}: CollectionListTemplateProps) {
  return (
    <PrintRoot
      frame={data}
      paper={data.paper}
      applyRoot={applyRoot}
      {...(ref === undefined ? {} : { ref })}
    >
      <section className="print-sheet">
        <PrintHeader header={data.header} />
        <Body data={data} />
        {/* On paper the footer is the page margin (print.css). This line is for the preview. */}
        <PrintScreenFooter footer={data.footer} />
      </section>
    </PrintRoot>
  );
}
