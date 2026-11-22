// A seller's account after this بکری, in his summary (plan 91, PRD FR-12.23 v1.53, FR-12.3, C6, C8; plan 95 item 8, PRD
// FR-12.52): the tick "پیشگی اس بکری سے کاٹیں" beside the lines of his account: پچھلا بقایا, آج کی صافی بکری,
// موقع پر ادائیگی, بکری کی باقی رقم، کھاتے میں جمع, and نیا بقایا, each with its side, and the working of the new
// balance behind its ⓘ. There is no box to type an amount here any more (Umar, 23-11-2026): cash to the seller is typed
// only in the نقد ادائیگی of his lots, and موقع پر ادائیگی is the sum of those, worked out by the backend. When it is more
// than the seller may take without a question, the balance will grow and an amber line says so; the save asks "is this
// entry correct?" (the box on the page).
//
// Every figure here is the backend's (CLAUDE.md rule 1): the lines that are 0 are left out by comparing the text a figure
// is written as, never the number.

import { Localized } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import type { ReactNode } from 'react';
import type { Working } from '../../../bindings/Working';
import { formatPaisa, useGrouping, type Paisa } from '../../../shared/format';
import type { PickedRecord } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Balance, CalcInfo, Num } from '../../../shared/ui';
import type { SellerPaidLines } from '../../../bindings/SellerPaidLines';

interface SellerPaidProps {
  seller: PickedRecord;
  /** What the preview says about this seller, or undefined while it cannot work it out. */
  lines: SellerPaidLines | undefined;
  /** The seller's advance tick, shown above the field when he has an advance open (plan 78 C3). */
  tick?: ReactNode;
}

interface LineProps {
  labelId: string;
  working?: Working;
  children: ReactNode;
  emphasis?: boolean;
}

function Line({ labelId, working, children, emphasis = false }: LineProps) {
  return (
    <div
      className={`flex items-center justify-between gap-3 border-b border-border py-1 ${emphasis ? 'type-label' : 'type-body'} text-ink`}
    >
      <Localized id={labelId}>
        <dt />
      </Localized>
      <dd className="value-col">
        {children}
        {working === undefined ? null : <CalcInfo working={working} inCell />}
      </dd>
    </div>
  );
}

export function SellerPaid({ seller, lines, tick }: SellerPaidProps) {
  const grouping = useGrouping();
  const language = useLanguage();
  const name = displayName(seller, language);
  // A figure is "nothing" when the text it is written as is 0: no amount is compared as a number here.
  const isNothing = (paisa: Paisa) => formatPaisa(paisa, { grouping }) === '0';
  return (
    <section
      aria-label={name}
      data-seller-paid={seller.id}
      className="flex min-w-0 flex-col gap-2 rounded-2 border border-border bg-surface-sunken p-3"
    >
      <Localized id="bikri-seller-paid-title" vars={{ name }}>
        <h3 className="type-label text-ink" />
      </Localized>
      {tick}
      {lines === undefined ? null : (
        <>
          {lines.needs_confirm ? (
            <p role="status" className="flex items-center gap-2 type-body text-warn">
              <TriangleAlert aria-hidden="true" className="size-icon-sm shrink-0" />
              <Localized id="bikri-seller-paid-above">
                <span />
              </Localized>
            </p>
          ) : null}
          <dl className="flex flex-col" aria-label={name}>
            <Line labelId="bikri-seller-paid-previous">
              <Balance
                balance={{ amount_paisa: lines.previous_paisa, side: lines.previous_side }}
              />
            </Line>
            <Line labelId="bikri-seller-paid-net">
              <Num paisa={lines.credit_paisa} />
            </Line>
            <Line labelId="bikri-seller-paid-paid">
              <Num paisa={lines.paid_paisa} />
            </Line>
            <Line labelId="bikri-seller-paid-rest">
              <Num paisa={lines.carry_paisa} />
            </Line>
            {isNothing(lines.advance_paisa) ? null : (
              // The advance typed on his lots, a debit of its own: it is in the new balance (PRD FR-12.52).
              <Line labelId="bikri-seller-paid-advance">
                <Num paisa={lines.advance_paisa} />
              </Line>
            )}
            {isNothing(lines.over_paid_paisa) ? null : (
              <Line labelId="bikri-seller-paid-over">
                <Num paisa={lines.over_paid_paisa} />
              </Line>
            )}
            {isNothing(lines.increase_owed_paisa) ? null : (
              <Line labelId="bikri-seller-paid-increase">
                <Num paisa={lines.increase_owed_paisa} />
              </Line>
            )}
            <Line labelId="bikri-seller-paid-new" working={lines.working} emphasis>
              <Balance balance={{ amount_paisa: lines.new_paisa, side: lines.new_side }} />
            </Line>
          </dl>
        </>
      )}
    </section>
  );
}
