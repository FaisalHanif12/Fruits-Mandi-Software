// The lot of a later sale as one strip (plan 38, later sale): who sent it, what it is, and how much
// is left, on one line, so the new lines grid under it is in the first screen. Every other figure of
// the lot (the day received, the days waiting, written off, taken as fruit) is behind an ⓘ, in the
// same `LotFacts` the stock list uses. All figures are the backend's (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import type { StockRow } from '../../../../bindings/StockRow';
import { displayName, useLanguage } from '../../../../shared/i18n';
import { InfoHint, Num } from '../../../../shared/ui';
import { stockSellerName } from '../../stock-columns';
import { LotFacts } from './StockLotCard';

function Pair({ label, children }: { label: ReactNode; children: ReactNode }) {
  return (
    <span className="flex items-baseline gap-2 type-body">
      <span className="text-ink-muted">{label}</span>
      <span className="text-ink">{children}</span>
    </span>
  );
}

export function LotStrip({ row }: { row: StockRow }) {
  const language = useLanguage();
  return (
    <section className="flex flex-wrap items-baseline gap-x-6 gap-y-1 rounded-3 border border-border bg-surface px-3 py-2">
      <Localized id="bikri-later-lot-title">
        <h2 className="type-label text-ink" />
      </Localized>
      <Pair label={<Localized id="glossary-beopari" />}>{stockSellerName(row, language)}</Pair>
      <Pair label={<Localized id="glossary-qism-maal" />}>
        {displayName(row.item, language)} {displayName(row.packing, language)}
      </Pair>
      <Pair label={<Localized id="bikri-col-qty-received" />}>
        <Num>{row.qty_received}</Num>
      </Pair>
      <Pair label={<Localized id="bikri-saved-qty-sold" />}>
        <Num>{row.qty_sold}</Num>
      </Pair>
      <Pair label={<Localized id="bikri-col-balance" />}>
        <Num>{row.balance}</Num>
      </Pair>
      <InfoHint>
        <LotFacts row={row} />
      </InfoHint>
    </section>
  );
}
