// The audit log (PRD FR-21.3, NFR-SEC-5): every change with who, when, what, and the old and new values.
// Read only: it is not edited or deleted from the app. It sits in Settings as its own tab. The filters
// and the open entry are kept above the screen (store.ts), so a language switch, which builds the grid
// again, loses nothing (Code Standards F39). A filter change asks the backend once, 250 ms after the
// last key (F35). A refusal that points at a filter box shows under that box and the rows stay.
//
// Four states: loading, an error with a retry, nothing found (with a way out), and the data.

import { Localized } from '@fluent/react';
import { useEffect, useMemo, useState, type ReactNode } from 'react';
import type { AuditPage } from '../../../bindings/AuditPage';
import type { UiError } from '../../../bindings/UiError';
import { uiErrorOf } from '../../../shared/api';
import { useDebouncedValue, useListWindow } from '../../../shared/hooks';
import { useEscape } from '../../../shared/keyboard';
import { useScreenMemory } from '../../../shared/screen-state';
import { InfoHint, ListWindowBar, LoadingState, PartnerLockedBox } from '../../../shared/ui';
import { useDayStatus } from '../../../app/use-day-status';
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
  // The log shows 30 days at a time, the latest first (PRD FR-19.11, v1.57, plan 95 item 18), unless the Owner types
  // dates of his own in the boxes: those win, and the bar is not shown.
  const dayStatus = useDayStatus();
  const today = dayStatus.data?.today;
  const listWindow = useListWindow({ kind: 'days', source: { kind: 'audit' }, today });
  // Without the day (its read failed) the log asks for every entry, as before the window.
  const custom = dayStatus.isError || settled.from.trim() !== '' || settled.to.trim() !== '';
  const wire = useMemo(() => {
    const typed = filtersToWire(settled);
    return custom ? typed : { ...typed, from: listWindow.range.from, to: listWindow.range.to };
  }, [settled, custom, listWindow.range.from, listWindow.range.to]);
  const query = useAuditPages(wire, true, custom || listWindow.ready);
  const options = useAuditOptions();

  // The page last shown, kept while a half-typed filter is refused. They are state set while
  // rendering, which React allows when the value really changed.
  const [kept, setKept] = useState<{ pages: AuditPage[] } | null>(null);
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

  // The log scrolls as one page: the table is as tall as its rows, so the page buttons and the 30-day bar stand under
  // it and never over a row (plan 100 D; the grid that filled the room left over ran under the pager).
  return (
    <div className="no-scrollbar relative flex min-h-0 flex-1 flex-col gap-3 overflow-auto pe-corner">
      <div className="flex shrink-0 items-center gap-1">
        <Localized id="audit-title">
          <h2 className="type-section" />
        </Localized>
        <InfoHint>
          <Localized id="audit-help">
            <p />
          </Localized>
          <Localized id="audit-open-hint">
            <p className="text-ink-muted" />
          </Localized>
        </InfoHint>
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
          {query.data?.pages.some((page) => page.partners_locked) === true ? (
            <PartnerLockedBox />
          ) : null}
          <AuditResults
            query={query}
            kept={filterRefusal === null ? null : kept}
            filtering={!isUnfiltered(filters)}
            onClear={clear}
            onOpen={open}
            focusRowId={focusRowId}
            onFocusRow={setFocusRowId}
          />
          {custom ? null : <ListWindowBar window={listWindow} />}
        </>
      ) : (
        <AuditDetail
          row={opened}
          onBack={() => {
            open(null);
          }}
        />
      )}
      {/* The band of the round assistant button: the Settings tab that holds this log runs down through it
          (`fill-to-corner`), so the page buttons end above it (Umar, 12-11-2026). */}
      <div aria-hidden="true" className="h-corner shrink-0" />
    </div>
  );
}
