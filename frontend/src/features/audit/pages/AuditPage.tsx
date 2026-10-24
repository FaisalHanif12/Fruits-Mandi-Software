// The audit log (PRD FR-21.3, NFR-SEC-5): every change with who, when, what, and the old and new values.
// Read only: it is not edited or deleted from the app. It sits in Settings as its own tab. The filters
// and the open entry are kept above the screen (store.ts), so a language switch, which builds the grid
// again, loses nothing (Code Standards F39). A filter change asks the backend once, 250 ms after the
// last key (F35). A refusal that points at a filter box shows under that box and the rows stay.
//
// Four states: loading, an error with a retry, nothing found (with a way out), and the data.

import { Localized } from '@fluent/react';
import type { InfiniteData } from '@tanstack/react-query';
import { useEffect, useMemo, useState, type ReactNode } from 'react';
import type { AuditPage } from '../../../bindings/AuditPage';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useDebouncedValue } from '../../../shared/hooks';
import { useEscape } from '../../../shared/keyboard';
import { useScreenMemory } from '../../../shared/screen-state';
import { LoadingState } from '../../../shared/ui';
import { AuditDetail } from '../components/AuditDetail';
import { AuditFilterBar } from '../components/AuditFilterBar';
import { AuditResults } from '../components/AuditResults';
import { filtersToWire, isUnfiltered } from '../filters';
import { useAuditOptions, useAuditPages } from '../hooks';
import { AUDIT_SCREEN, auditFromRemembered, rememberedAudit } from '../remember';
import { useAuditStore } from '../store';

/** How long the filters rest before the backend is asked (Code Standards F35). */
const FILTER_SETTLE_MS = 250;

/** True when the refusal points at a filter box: the box shows it and the rows stay. */
function isFilterRefusal(error: UiError): boolean {
  return (
    error.errors.length > 0 &&
    error.errors.every(
      (item) => typeof item.field === 'string' && item.field.startsWith('filters.'),
    )
  );
}

/**
 * The filters the Owner last used are put back once when the log first opens, and each change is
 * kept (PRD FR-7.45). The log is drawn once they are back, so it never flashes empty boxes.
 */
function RememberedAuditFilters({ children }: { children: ReactNode }) {
  const { ready, saved, remember } = useScreenMemory(AUDIT_SCREEN);
  const hydrated = useAuditStore((now) => now.hydrated);
  const filters = useAuditStore((now) => now.filters);
  useEffect(() => {
    if (ready) useAuditStore.getState().hydrate(auditFromRemembered(saved));
  }, [ready, saved]);
  useEffect(() => {
    if (hydrated) remember(rememberedAudit(filters));
  }, [hydrated, filters, remember]);
  if (!hydrated) return <LoadingState />;
  return <>{children}</>;
}

export function AuditLog() {
  return (
    <RememberedAuditFilters>
      <AuditLogBody />
    </RememberedAuditFilters>
  );
}

function AuditLogBody() {
  const filters = useAuditStore((now) => now.filters);
  const opened = useAuditStore((now) => now.opened);
  const focusRowId = useAuditStore((now) => now.focusRowId);
  const { change, clear, open, setFocusRowId } = useAuditStore.getState();
  const settled = useDebouncedValue(filters, FILTER_SETTLE_MS);
  const wire = useMemo(() => filtersToWire(settled), [settled]);
  const query = useAuditPages(wire);
  const options = useAuditOptions();

  // The pages last shown, kept while a half-typed filter is refused. They are state set while
  // rendering, which React allows when the value really changed.
  const [kept, setKept] = useState<InfiniteData<AuditPage> | null>(null);
  if (query.data !== undefined && !query.isPlaceholderData && query.data !== kept) {
    setKept(query.data);
  }
  const refusal = query.isError ? uiErrorOf(query.error) : null;
  const filterRefusal = refusal !== null && isFilterRefusal(refusal) ? refusal : null;

  useEscape(
    'leave',
    () => {
      open(null);
    },
    opened !== null,
  );

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <Localized id="audit-title">
          <h2 className="type-section" />
        </Localized>
        <Localized id="audit-help">
          <p className="type-body text-ink-muted" />
        </Localized>
      </div>
      {opened === null ? (
        <>
          <AuditFilterBar
            state={filters}
            onChange={change}
            options={options.data}
            optionsFailed={options.isError}
            error={filterRefusal}
          />
          <AuditResults
            query={query}
            kept={filterRefusal === null ? null : kept}
            filtering={!isUnfiltered(filters)}
            onClear={clear}
            onOpen={open}
            focusRowId={focusRowId}
            onFocusRow={setFocusRowId}
          />
        </>
      ) : (
        <AuditDetail
          row={opened}
          onBack={() => {
            open(null);
          }}
        />
      )}
    </div>
  );
}
