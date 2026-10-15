// The chart of accounts in three levels with live balances (PRD FR-10.1): heads, controls and
// accounts. One tab stop, like any tree: ↑ ↓ move, Home and End jump, Page Up and Page Down move
// a page, the arrow that points at the children opens a row (or goes to its first child) and the
// arrow that points at the parent closes it (or goes to the parent), in both directions (FR-7.13).
// Enter opens a head or a control, and opens an account. Only the rows in view are drawn
// (NFR-SPD-18), because a control can hold thousands of accounts. Nothing is worked out here:
// every balance and its side come from the backend.

import { Localized } from '@fluent/react';
import { ChevronRight } from 'lucide-react';
import { useEffect, useId, useLayoutEffect, useMemo, useState } from 'react';
import { tokenPx } from '../../../shared/hooks';
import { displayName, useLanguage } from '../../../shared/i18n';
import { treeKeyHandler } from '../../../shared/keyboard';
import { Balance, Code } from '../../../shared/ui';
import type { ChartHead } from '../../../shared/accounts';
import { headKeys, horizontalMove, visibleRows, type TreeRow } from '../chart-tree';

const FALLBACK_ROW_PX = 44;
/** What is drawn when the height of the tree cannot be read (tests have no layout). */
const FALLBACK_ROWS = 60;
const OVERSCAN_ROWS = 4;
const PAGE_ROWS = 10;
/** How far each level is moved in, in rows of the spacing scale (design tokens). */
const INDENT_CLASS: Record<TreeRow['level'], string> = { 1: 'ps-3', 2: 'ps-5', 3: 'ps-6' };

interface ChartTreeProps {
  heads: readonly ChartHead[];
  /** Enter or a double click on an account. */
  onOpenAccount: (id: number) => void;
}

export function ChartTree({ heads, onOpenAccount }: ChartTreeProps) {
  const language = useLanguage();
  const treeId = useId();
  const [expanded, setExpanded] = useState<ReadonlySet<string>>(() => headKeys(heads));
  const [activeKey, setActiveKey] = useState<string | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportPx, setViewportPx] = useState(0);
  const [rowPx] = useState(() => tokenPx('--control-h', FALLBACK_ROW_PX));
  const [box, setBox] = useState<HTMLDivElement | null>(null);

  const rows = useMemo(() => visibleRows(heads, expanded), [heads, expanded]);
  const activeIndex = Math.max(
    rows.findIndex((row) => row.key === activeKey),
    0,
  );

  // The height of the tree, read from the page and kept up to date when the window changes.
  useLayoutEffect(() => {
    if (box === null || typeof ResizeObserver === 'undefined') return undefined;
    // The observer reports the height once it starts watching, and again when it changes.
    const observer = new ResizeObserver(() => {
      setViewportPx(box.clientHeight);
    });
    observer.observe(box);
    return () => {
      observer.disconnect();
    };
  }, [box]);

  // Keep the active row in view while the arrows move through a long tree.
  useEffect(() => {
    if (box === null || box.clientHeight === 0) return;
    const top = activeIndex * rowPx;
    const bottom = top + rowPx;
    if (top < box.scrollTop) box.scrollTo({ top });
    else if (bottom > box.scrollTop + box.clientHeight) {
      box.scrollTo({ top: bottom - box.clientHeight });
    }
  }, [activeIndex, box, rowPx]);

  const visibleCount = viewportPx === 0 ? FALLBACK_ROWS : Math.ceil(viewportPx / rowPx) + 1;
  const first = viewportPx === 0 ? 0 : Math.max(Math.floor(scrollTop / rowPx) - OVERSCAN_ROWS, 0);
  const last = Math.min(first + visibleCount + 2 * OVERSCAN_ROWS, rows.length);

  const toggle = (key: string, open: boolean) => {
    setExpanded((current) => {
      const next = new Set(current);
      if (open) next.add(key);
      else next.delete(key);
      return next;
    });
  };

  const activateIndex = (index: number) => {
    const row = rows[index];
    if (row !== undefined) setActiveKey(row.key);
  };

  const onKeyDown = treeKeyHandler({
    count: rows.length,
    active: activeIndex,
    pageSize: PAGE_ROWS,
    onMove: activateIndex,
    onHorizontal: (action) => {
      const move = horizontalMove(rows, activeIndex, action);
      if (move.to === 'none') return false;
      if (move.to === 'toggle') toggle(move.key, move.open);
      else if (move.to === 'index') activateIndex(move.index);
      return true;
    },
    onEnter: () => {
      const row = rows[activeIndex];
      if (row === undefined) return;
      if (row.kind === 'account') onOpenAccount(row.id);
      else toggle(row.key, !row.expanded);
    },
  });

  const active = rows[activeIndex];
  return (
    <div
      ref={setBox}
      role="tree"
      tabIndex={0}
      aria-labelledby={`${treeId}-label`}
      aria-activedescendant={active === undefined ? undefined : `${treeId}-${active.key}`}
      onKeyDown={onKeyDown}
      onScroll={(event) => {
        setScrollTop(event.currentTarget.scrollTop);
      }}
      className="h-full min-h-tree overflow-y-auto rounded-3 border border-border bg-surface"
    >
      <Localized id="accounts-tree-label">
        <span id={`${treeId}-label`} className="sr-only" />
      </Localized>
      {first > 0 ? <div aria-hidden="true" style={{ height: first * rowPx }} /> : null}
      {rows.slice(first, last).map((row, offset) => {
        const index = first + offset;
        const isActive = index === activeIndex;
        return (
          <div
            key={row.key}
            id={`${treeId}-${row.key}`}
            role="treeitem"
            aria-level={row.level}
            aria-expanded={row.expandable ? row.expanded : undefined}
            aria-selected={isActive}
            aria-setsize={rows.length}
            aria-posinset={index + 1}
            onClick={() => {
              setActiveKey(row.key);
              if (row.expandable) toggle(row.key, !row.expanded);
            }}
            onDoubleClick={() => {
              if (row.kind === 'account') onOpenAccount(row.id);
            }}
            style={{ height: rowPx }}
            className={`flex cursor-pointer items-center gap-3 border-b border-border pe-3 type-body ${INDENT_CLASS[row.level]} ${isActive ? 'bg-row-current' : ''} ${row.inactive ? 'text-ink-muted' : 'text-ink'} ${row.kind === 'head' ? 'font-semibold' : ''}`}
          >
            {row.expandable ? (
              <ChevronRight
                aria-hidden="true"
                className={`size-icon-md shrink-0 transition-calm ${row.expanded ? 'rotate-90' : 'rtl:rotate-180'}`}
              />
            ) : (
              <span aria-hidden="true" className="size-icon-md shrink-0" />
            )}
            <Code>{row.code}</Code>
            <span className="min-w-0 flex-1 truncate">{displayName(row, language)}</span>
            {row.inactive ? (
              <Localized id="accounts-status-inactive">
                <span className="type-chip rounded-1 bg-surface-sunken px-2 text-ink-muted" />
              </Localized>
            ) : null}
            <Balance balance={row.balance} />
          </div>
        );
      })}
      {last < rows.length ? (
        <div aria-hidden="true" style={{ height: (rows.length - last) * rowPx }} />
      ) : null}
    </div>
  );
}
