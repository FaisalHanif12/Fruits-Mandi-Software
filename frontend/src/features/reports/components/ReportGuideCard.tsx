// The guide card of one report (plan 75 T7, PRD FR-19.14): what the Owner sees when he points at a report
// on the reports page. It tells him, before he opens the report, what it shows, its columns, its filters,
// what can be done with it and where to print, and when to use it. Seven parts: its name with its ID and
// its group; what it shows; what you can do (screen, print, PDF, Excel, and the special print of R1, R5
// and R35); when to use it; its columns; its filters; and a button that opens it. The words come first and
// the lists after them, so the answer to "where do I print this?" is in view without scrolling when the
// window is short (the card scrolls inside itself, and its button stays at the bottom).
//
// Everything comes from the catalogue the page already has and from Fluent: the columns and the filters
// are the report's own (the catalogue's `column_title_ids` and `filters`), so the card never names one the
// report has not, and opening it needs no call to the backend. A part whose words are missing is left
// out, so a report the backend adds before its words are written still gets the card it can (the same
// rule as the one-line help of the small card).

import { Localized, useLocalization } from '@fluent/react';
import {
  ArrowRight,
  FileDown,
  FileSpreadsheet,
  Files,
  Monitor,
  Printer,
  type LucideIcon,
} from 'lucide-react';
import type { ReactNode } from 'react';
import type { ReportCatalogEntry } from '../../../bindings/ReportCatalogEntry';
import { Button, Code, Notice } from '../../../shared/ui';
import {
  columnTitleIdsOf,
  detailIdOf,
  filterLabelIdsOf,
  specialPrintIdOf,
  whenIdOf,
} from '../catalog';

interface ReportGuideCardProps {
  entry: ReportCatalogEntry;
  /** Opens the report, the same as a click on its small card. */
  onOpen: () => void;
}

/** One labelled part of the card. The label is quiet, the words under it are the content. */
function Part({ label, children }: { label: ReactNode; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-1 border-t border-border pt-3">
      <dt className="type-label text-ink-muted">{label}</dt>
      <dd className="type-body text-ink">{children}</dd>
    </div>
  );
}

/** One thing that can be done with a report: an icon and a few words. */
function Action({ icon: Icon, id }: { icon: LucideIcon; id: string }) {
  return (
    <li className="flex items-center gap-2">
      <Icon aria-hidden="true" className="size-icon-md shrink-0 text-ink-muted" />
      <Localized id={id} />
    </li>
  );
}

/** A short list of words as small chips (columns, filters). */
function Chips({ words }: { words: readonly string[] }) {
  return (
    <ul className="flex flex-wrap gap-2">
      {words.map((word, index) => (
        <li
          // The same word can come twice (two columns that read alike), so the place is part of the key.
          key={`${word}-${index.toString()}`}
          className="rounded-1 bg-surface-sunken px-2 type-chip text-ink"
        >
          {word}
        </li>
      ))}
    </ul>
  );
}

export function ReportGuideCard({ entry, onOpen }: ReportGuideCardProps) {
  const { l10n } = useLocalization();
  const known = (id: string) => l10n.getBundle(id) !== null;
  const words = (ids: readonly string[]) => ids.filter(known).map((id) => l10n.getString(id));

  const detailId = detailIdOf(entry);
  const whenId = whenIdOf(entry);
  const columns = words(columnTitleIdsOf(entry));
  const filters = words(filterLabelIdsOf(entry));
  const specialId = specialPrintIdOf(entry);

  return (
    <div className="flex flex-col gap-3 p-4">
      <div className="flex items-center gap-3">
        <span className="shrink-0 rounded-1 bg-primary-soft px-2 text-primary">
          <Code>{entry.id}</Code>
        </span>
        <div className="flex min-w-0 flex-col">
          <h3 className="type-section text-ink">{l10n.getString(entry.title_id)}</h3>
          <p className="type-body text-ink-muted">{l10n.getString(entry.group_title_id)}</p>
        </div>
      </div>

      <dl className="flex flex-col gap-3">
        {known(detailId) ? (
          <Part label={<Localized id="report-guide-shows" />}>{l10n.getString(detailId)}</Part>
        ) : null}

        <Part label={<Localized id="report-guide-actions" />}>
          <ul className="flex flex-col gap-2">
            <li>
              <ul className="flex flex-wrap gap-x-4 gap-y-1">
                <Action icon={Monitor} id="report-guide-action-screen" />
                <Action icon={Printer} id="report-guide-action-print" />
                <Action icon={FileDown} id="report-guide-action-pdf" />
                <Action icon={FileSpreadsheet} id="report-guide-action-excel" />
              </ul>
            </li>
            {specialId !== null && known(specialId) ? (
              <li className="flex items-center gap-2 rounded-2 bg-primary-soft px-2 text-primary">
                <Files aria-hidden="true" className="size-icon-md shrink-0" />
                {l10n.getString(specialId)}
              </li>
            ) : null}
          </ul>
        </Part>

        {known(whenId) ? (
          <Part label={<Localized id="report-guide-when" />}>{l10n.getString(whenId)}</Part>
        ) : null}

        {columns.length === 0 ? null : (
          <Part label={<Localized id="report-guide-columns" />}>
            <Chips words={columns} />
          </Part>
        )}

        <Part label={<Localized id="report-guide-filters" />}>
          <div className="flex flex-col gap-2">
            {filters.length === 0 ? (
              <Localized id="report-guide-filters-none" />
            ) : (
              <Chips words={filters} />
            )}
            {entry.party_required ? (
              <Notice tone="info">
                <Localized id="report-guide-party-required" />
              </Notice>
            ) : null}
          </div>
        </Part>
      </dl>

      {/* It stays at the bottom edge while a tall card scrolls, so the way in is always in view. */}
      <div className="sticky -mx-4 -mb-4 bottom-0 flex justify-end border-t border-border bg-surface px-4 py-3">
        <Button variant="primary" onClick={onOpen}>
          <Localized id="report-guide-open" />
          <ArrowRight aria-hidden="true" className="size-icon-md rtl:rotate-180" />
        </Button>
      </div>
    </div>
  );
}
