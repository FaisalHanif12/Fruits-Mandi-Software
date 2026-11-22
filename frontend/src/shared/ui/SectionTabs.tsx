// The tabs of a section along the top of its screens (PRD FR-5.3, v1.56, plan 93): the second menu of بکری and
// روکڑ, as one row in the title row of the screen. Each tab has its icon and a short label; the open one is a filled
// pill in primary and is the page title (the screen keeps its <h1> for screen readers). The tabs are always ONE row.
// The row wants the width of ALL its tabs, so when the screen's own buttons do not fit beside them the buttons go
// to the row under (the screen's title row wraps) and the tabs keep the whole first row; only when the tabs alone do
// not fit the width do the last ones go behind a "More" button, with the open tab always in the row. Before the row
// has a width (a test, a window being built) every tab is shown.
//
// Keys (PRD FR-7.17): ← → move between the tabs in the reading direction, Enter opens, ↓ moves into the screen. Only
// the open tab is a Tab stop; the arrows reach the others (roving focus). The row marks itself `data-section-tabs`, so
// the arrow into a screen never lands on a tab.

import { Localized, useLocalization } from '@fluent/react';
import { ChevronDown, type LucideIcon } from 'lucide-react';
import { DropdownMenu } from 'radix-ui';
import { useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { useUiLanguage } from '../i18n';
import { useRovingRowKeys } from '../keyboard';
import { radixCollisionPadding } from './dropdown-position';
import { itemsThatFit } from './overflow-fit';
import { useSectionTabs } from './section-tabs-context';

export interface SectionTab<Id extends string> {
  id: Id;
  /** The words of the tab, already in the app language. */
  label: string;
  icon: LucideIcon;
}

interface SectionTabsProps<Id extends string> {
  tabs: readonly SectionTab<Id>[];
  current: Id;
  onSelect: (id: Id) => void;
  /** What the row is, read by screen readers ("بکری کا مینو"). Comes from Fluent. */
  label: string;
}

/** The space between two tabs, as the row's own gap (`gap-1` is 4 px). */
const TAB_GAP_PX = 4;

const TAB_CLASSES =
  'type-control inline-flex h-control-compact shrink-0 items-center gap-2 whitespace-nowrap rounded-2 px-3 transition-calm';

/**
 * How many tabs fit the width (the arithmetic is `itemsThatFit`, with the gap of this row). `widths` are the measured
 * widths of the tabs and `moreWidth` the width of the "More" button.
 */
export function sectionTabsThatFit(
  width: number,
  widths: readonly number[],
  moreWidth: number,
): number {
  return itemsThatFit(width, widths, moreWidth, TAB_GAP_PX);
}

function TabLook({ tab, active }: { tab: SectionTab<string>; active: boolean }) {
  const Icon = tab.icon;
  return (
    <>
      <Icon
        aria-hidden="true"
        className={`size-icon-md shrink-0 ${active ? 'text-on-primary' : 'text-ink-muted'}`}
      />
      <span>{tab.label}</span>
    </>
  );
}

const lookOf = (active: boolean) =>
  `${TAB_CLASSES} ${active ? 'bg-primary text-on-primary' : 'text-ink hover:bg-surface-sunken'}`;

export function SectionTabs<Id extends string>({
  tabs,
  current,
  onSelect,
  label,
}: SectionTabsProps<Id>) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const onKeyDown = useRovingRowKeys();
  const boxRef = useRef<HTMLElement>(null);
  const rulerRef = useRef<HTMLDivElement>(null);
  const moreRulerRef = useRef<HTMLSpanElement>(null);
  const [fit, setFit] = useState(tabs.length);

  useLayoutEffect(() => {
    const box = boxRef.current;
    const ruler = rulerRef.current;
    if (box === null || ruler === null) return undefined;
    const measure = () => {
      const widths = (Array.from(ruler.children) as HTMLElement[]).map((cell) => cell.offsetWidth);
      const more = moreRulerRef.current?.offsetWidth ?? 0;
      setFit(sectionTabsThatFit(box.clientWidth, widths, more));
    };
    measure();
    if (typeof ResizeObserver !== 'function') return undefined;
    const watcher = new ResizeObserver(measure);
    watcher.observe(box);
    return () => {
      watcher.disconnect();
    };
  }, [tabs, language]);

  // The open tab is always in the row: it takes the place of the last tab that fits.
  const activeIndex = tabs.findIndex((tab) => tab.id === current);
  const shownIds = new Set(tabs.slice(0, fit).map((tab) => tab.id));
  if (fit < tabs.length && activeIndex >= fit) {
    const last = tabs.slice(0, fit).at(-1);
    if (last !== undefined) shownIds.delete(last.id);
    shownIds.add(current);
  }
  const shown = tabs.filter((tab) => shownIds.has(tab.id));
  const hidden = tabs.filter((tab) => !shownIds.has(tab.id));

  // The row is as wide as ALL its tabs want to be, whatever is shown now (the ruler below is in the flow, with no
  // height), and it gives way when the room is less. So a screen's title row can tell whether the tabs and the
  // screen's own buttons fit side by side: if not, the buttons go to the row under the tabs and the tabs keep the whole
  // first row. "More" shows only when the tabs alone do not fit the width (PRD FR-5.3, v1.56).
  return (
    <nav
      ref={boxRef}
      aria-label={label}
      data-section-tabs=""
      onKeyDown={onKeyDown}
      className="flex min-w-0 shrink flex-col"
    >
      <div
        aria-hidden="true"
        className="pointer-events-none invisible relative h-0 overflow-hidden whitespace-nowrap"
      >
        <div ref={rulerRef} className="flex w-max gap-1">
          {tabs.map((tab) => (
            <span key={tab.id} className={TAB_CLASSES}>
              <TabLook tab={tab} active={false} />
            </span>
          ))}
        </div>
        <span ref={moreRulerRef} className={`${TAB_CLASSES} absolute`}>
          <Localized id="common-tabs-more" />
          <ChevronDown className="size-icon-md" />
        </span>
      </div>
      <div className="flex min-w-0 items-center gap-1">
        {shown.map((tab) => {
          const active = tab.id === current;
          return (
            <button
              key={tab.id}
              type="button"
              data-nav-item
              aria-current={active ? 'page' : undefined}
              // Only the open tab is a Tab stop; the arrows reach the rest.
              tabIndex={active ? 0 : -1}
              onClick={() => {
                onSelect(tab.id);
              }}
              className={lookOf(active)}
            >
              <TabLook tab={tab} active={active} />
            </button>
          );
        })}
        {hidden.length === 0 ? null : (
          <DropdownMenu.Root dir={direction}>
            <DropdownMenu.Trigger
              data-nav-item
              className={`${lookOf(false)} data-[state=open]:bg-surface-sunken`}
            >
              <Localized id="common-tabs-more" />
              <ChevronDown aria-hidden="true" className="size-icon-md" />
            </DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                lang={language}
                align="end"
                sideOffset={4}
                collisionPadding={radixCollisionPadding()}
                aria-label={l10n.getString('common-tabs-more')}
                className="z-50 min-w-nav rounded-2 border border-border bg-surface p-1 shadow-overlay"
              >
                {hidden.map((tab) => {
                  const Icon = tab.icon;
                  return (
                    <DropdownMenu.Item
                      key={tab.id}
                      onSelect={() => {
                        onSelect(tab.id);
                      }}
                      className="type-body flex h-control cursor-pointer items-center gap-3 rounded-1 px-3 text-ink data-[highlighted]:bg-primary-soft"
                    >
                      <Icon aria-hidden="true" className="size-icon-md shrink-0 text-ink-muted" />
                      {tab.label}
                    </DropdownMenu.Item>
                  );
                })}
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
        )}
      </div>
    </nav>
  );
}

interface TitleOrTabsProps {
  /** The title in words, kept for screen readers as the <h1> when the tabs stand in for it. */
  title: ReactNode;
  /** The screen's own visible title, shown when it belongs to no section. */
  children: ReactNode;
}

/**
 * The place of a screen's title in its title row. In a section the row of tabs stands in for the title (the open
 * tab IS the title, no extra row of height) and the <h1> is kept for screen readers; outside a section the
 * screen's own title shows as before (PRD FR-5.3, v1.56).
 */
export function TitleOrTabs({ title, children }: TitleOrTabsProps) {
  const tabs = useSectionTabs();
  if (tabs === null) return <>{children}</>;
  return (
    <>
      {tabs}
      <h1 className="sr-only">{title}</h1>
    </>
  );
}
