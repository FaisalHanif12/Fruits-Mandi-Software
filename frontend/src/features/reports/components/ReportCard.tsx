// One report on the reports page (PRD FR-19.2, FR-19.14): a small card with its name, its line of help and
// its ID as a quiet badge. Pointing at it, or moving to it with the keyboard, opens its guide card beside
// it (`ReportGuideCard`); clicking it, or Enter, opens the report.

import { useLocalization } from '@fluent/react';
import { Link, useNavigate } from '@tanstack/react-router';
import { FileText } from 'lucide-react';
import type { ReportCatalogEntry } from '../../../bindings/ReportCatalogEntry';
import { Code, HoverDetails } from '../../../shared/ui';
import { helpIdOf } from '../catalog';
import { ReportGuideCard } from './ReportGuideCard';

interface ReportCardProps {
  entry: ReportCatalogEntry;
}

export function ReportCard({ entry }: ReportCardProps) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const title = l10n.getString(entry.title_id);
  const helpId = helpIdOf(entry);
  return (
    <HoverDetails
      label={l10n.getString('report-guide-label', { title })}
      card={
        <ReportGuideCard
          entry={entry}
          onOpen={() => {
            void navigate({ to: '/reports/$id', params: { id: entry.id } });
          }}
        />
      }
    >
      <Link
        to="/reports/$id"
        params={{ id: entry.id }}
        data-nav-item
        className="flex min-h-control items-center gap-3 rounded-3 border border-border bg-surface p-3 transition-calm hover:border-primary hover:bg-primary-soft data-[state=open]:border-primary data-[state=open]:bg-primary-soft"
      >
        <span className="flex size-control shrink-0 items-center justify-center rounded-2 bg-primary-soft text-primary">
          <FileText aria-hidden="true" className="size-icon-md" />
        </span>
        <span className="flex min-w-0 flex-1 flex-col">
          <span className="type-section text-ink">{title}</span>
          {l10n.getBundle(helpId) === null ? null : (
            <span className="type-body text-ink-muted">{l10n.getString(helpId)}</span>
          )}
        </span>
        <span className="shrink-0 self-start text-ink-muted">
          <Code>{entry.id}</Code>
        </span>
      </Link>
    </HoverDetails>
  );
}
