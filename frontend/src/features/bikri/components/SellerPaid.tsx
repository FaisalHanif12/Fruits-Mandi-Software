// A seller's money paid on the spot, in his summary (plan 91, PRD FR-12.23 v1.53, FR-12.3, C6, C8): one field, "موقع پر
// ادائیگی", beside the advance tick, and the lines of his account after this بکری: پچھلا بقایا, آج کی صافی بکری,
// موقع پر ادائیگی, بکری کی باقی رقم، کھاتے میں جمع, and نیا بقایا, each with its side, and
// the working of the new balance behind its ⓘ. When the amount is more than the seller may take without a question,
// the balance will grow and an amber line says so; the save asks "is this entry correct?" (the box on the page).
//
// The amount typed is spread by the backend over his lots' نقد, which are written into the lots' own fields
// (`useSellerPaidSpread`); the field shows the sum of those نقد as the backend worked it out. Every figure here is the
// backend's (CLAUDE.md rule 1): the lines that are 0 are left out by comparing the text a figure is written as, never
// the number.

import { Localized } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import { useState, type ReactNode } from 'react';
import type { Working } from '../../../bindings/Working';
import { formatPaisa, useGrouping, type Paisa } from '../../../shared/format';
import type { PickedRecord } from '../../../shared/grid';
import { displayName, useLanguage } from '../../../shared/i18n';
import { Balance, CalcInfo, Field, MoneyInput, Num, fieldError } from '../../../shared/ui';
import { cashText, type SellerPaidSpreading } from '../hooks/useSellerPaidSpread';
import type { SellerPaidLines } from '../../../bindings/SellerPaidLines';

interface SellerPaidProps {
  seller: PickedRecord;
  /** What the preview says about this seller, or undefined while it cannot work it out. */
  lines: SellerPaidLines | undefined;
  spreading: SellerPaidSpreading;
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

export function SellerPaid({ seller, lines, spreading, tick }: SellerPaidProps) {
  const grouping = useGrouping();
  const language = useLanguage();
  const name = displayName(seller, language);
  // What is being typed, until the field loses focus; then the field shows the sum the backend worked out.
  const [typed, setTyped] = useState<string | null>(null);
  const value = typed ?? (lines === undefined ? '' : cashText(lines.paid_paisa));
  // A figure is "nothing" when the text it is written as is 0: no amount is compared as a number here.
  const isNothing = (paisa: Paisa) => formatPaisa(paisa, { grouping }) === '0';
  return (
    <section
      aria-label={name}
      data-seller-paid={seller.id}
      className="flex flex-col gap-2 rounded-2 border border-border bg-surface-sunken p-3"
    >
      <Localized id="bikri-seller-paid-title" vars={{ name }}>
        <h3 className="type-label text-ink" />
      </Localized>
      {tick}
      <div
        onBlur={() => {
          setTyped(null);
        }}
      >
        <Field
          label={<Localized id="bikri-seller-paid-field" />}
          info={
            <Localized id="bikri-seller-paid-hint">
              <p />
            </Localized>
          }
          error={fieldError(spreading.errors[seller.id], 'seller_paid[0].amount')}
        >
          {(props) => (
            <MoneyInput
              {...props}
              value={value}
              onChange={(text) => {
                setTyped(text);
                spreading.spread(seller.id, text);
              }}
            />
          )}
        </Field>
      </div>
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
