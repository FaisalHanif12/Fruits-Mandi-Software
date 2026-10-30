// Tabs (design system 6, board UsersUr): a row of sections, one open. Built on Radix Tabs, so
// ← and → move between tabs in the direction the user sees, Home and End jump, and the panel is
// linked for screen readers. The open tab is in primary.
//
// With `overflow` the tabs are always ONE row (plan 38): the tabs that do not fit the free width
// go behind a "More" button that opens a menu (arrows and Enter pick one; Esc closes). The open tab
// is always in the row, so it never moves to another line or disappears into the menu. Before the
// row has a width (a test, a window being built) every tab is shown.

import { Localized, useLocalization } from '@fluent/react';
import { ChevronDown } from 'lucide-react';
import { DropdownMenu, Tabs as RadixTabs } from 'radix-ui';
import { useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { useUiLanguage } from '../i18n';

export interface TabItem {
  id: string;
  label: ReactNode;
  content: ReactNode;
  /** The words of the tab as text, for its entry in the "More" menu. Defaults to the label. */
  menuLabel?: string;
}

interface TabsProps {
  items: readonly TabItem[];
  value: string;
  onValueChange: (id: string) => void;
  /** What the row of tabs is, read by screen readers. Comes from Fluent. */
  label: string;
  /**
   * One row always: the tabs that do not fit go behind a "More" button (plan 38). The tabs then also
   * fill the height of their screen, and a panel is given the room that is left.
   */
  overflow?: boolean;
}

const TRIGGER_CLASSES =
  'type-control h-control shrink-0 whitespace-nowrap rounded-t-2 px-4 text-ink-muted transition-calm data-[state=active]:bg-primary-soft data-[state=active]:text-primary';

const MENU_ITEM_CLASSES =
  'type-body flex h-control cursor-pointer items-center rounded-1 px-3 text-ink data-[highlighted]:bg-primary-soft';

/** The space between two tabs, as the row's own gap (`gap-2` is 8 px). */
const TAB_GAP_PX = 8;

/**
 * How many tabs fit the width. `widths` are the widths of the tabs and `moreWidth` the width of
 * the "More" button, all measured. Every tab fits when the total is within the width.
 */
export function tabsThatFit(width: number, widths: readonly number[], moreWidth: number): number {
  if (width <= 0 || widths.length === 0) return widths.length;
  const all = widths.reduce((sum, w) => sum + w, 0) + TAB_GAP_PX * (widths.length - 1);
  if (all <= width) return widths.length;
  let used = moreWidth;
  let count = 0;
  for (const w of widths) {
    if (used + TAB_GAP_PX + w > width) break;
    used += TAB_GAP_PX + w;
    count += 1;
  }
  return Math.max(count, 1);
}

function OverflowList({ items, value, onValueChange, label }: Omit<TabsProps, 'overflow'>) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const boxRef = useRef<HTMLDivElement>(null);
  const rulerRef = useRef<HTMLDivElement>(null);
  const [fit, setFit] = useState(items.length);

  useLayoutEffect(() => {
    const box = boxRef.current;
    const ruler = rulerRef.current;
    if (box === null || ruler === null) return undefined;
    const measure = () => {
      const cells = Array.from(ruler.children) as HTMLElement[];
      const more = cells.at(-1)?.offsetWidth ?? 0;
      const widths = cells.slice(0, -1).map((cell) => cell.offsetWidth);
      setFit(tabsThatFit(box.clientWidth, widths, more));
    };
    measure();
    if (typeof ResizeObserver !== 'function') return undefined;
    const watcher = new ResizeObserver(measure);
    watcher.observe(box);
    return () => {
      watcher.disconnect();
    };
  }, [items, language]);

  // The open tab is always in the row: it takes the place of the last tab that fits.
  const activeIndex = items.findIndex((item) => item.id === value);
  const shownIds = new Set(items.slice(0, fit).map((item) => item.id));
  if (fit < items.length && activeIndex >= fit) {
    const last = items.slice(0, fit).at(-1);
    if (last !== undefined) shownIds.delete(last.id);
    shownIds.add(value);
  }
  const shown = items.filter((item) => shownIds.has(item.id));
  const hidden = items.filter((item) => !shownIds.has(item.id));

  return (
    <div ref={boxRef} className="relative flex items-end gap-2 border-b border-border">
      <RadixTabs.List aria-label={label} className="flex flex-nowrap gap-2">
        {shown.map((item) => (
          <RadixTabs.Trigger key={item.id} value={item.id} className={TRIGGER_CLASSES}>
            {item.label}
          </RadixTabs.Trigger>
        ))}
      </RadixTabs.List>
      {hidden.length === 0 ? null : (
        <DropdownMenu.Root dir={direction}>
          <DropdownMenu.Trigger className="type-control inline-flex h-control shrink-0 items-center gap-2 rounded-t-2 px-4 text-ink-muted transition-calm">
            <Localized id="common-tabs-more" />
            <ChevronDown aria-hidden="true" className="size-icon-md" />
          </DropdownMenu.Trigger>
          <DropdownMenu.Portal>
            <DropdownMenu.Content
              lang={language}
              align="end"
              sideOffset={4}
              aria-label={l10n.getString('common-tabs-more')}
              className="z-50 min-w-nav rounded-2 border border-border bg-surface p-1 shadow-overlay"
            >
              {hidden.map((item) => (
                <DropdownMenu.Item
                  key={item.id}
                  className={MENU_ITEM_CLASSES}
                  onSelect={() => {
                    onValueChange(item.id);
                  }}
                >
                  {item.label}
                </DropdownMenu.Item>
              ))}
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>
      )}
      {/* The ruler: the same tabs and a "More" button, invisible, measured to see what fits. */}
      <div
        ref={rulerRef}
        aria-hidden="true"
        className="pointer-events-none invisible absolute flex gap-2"
      >
        {items.map((item) => (
          <span key={item.id} className={`${TRIGGER_CLASSES} inline-flex items-center`}>
            {item.label}
          </span>
        ))}
        <span className="type-control inline-flex h-control items-center gap-2 px-4">
          <Localized id="common-tabs-more" />
          <ChevronDown className="size-icon-md" />
        </span>
      </div>
    </div>
  );
}

export function Tabs({ items, value, onValueChange, label, overflow = false }: TabsProps) {
  const { direction } = useUiLanguage();
  return (
    <RadixTabs.Root
      value={value}
      onValueChange={onValueChange}
      dir={direction}
      className={`flex flex-col gap-5 ${overflow ? 'min-h-0 flex-1' : ''}`}
    >
      {overflow ? (
        <OverflowList items={items} value={value} onValueChange={onValueChange} label={label} />
      ) : (
        <RadixTabs.List aria-label={label} className="flex flex-wrap gap-2 border-b border-border">
          {items.map((item) => (
            <RadixTabs.Trigger key={item.id} value={item.id} className={TRIGGER_CLASSES}>
              {item.label}
            </RadixTabs.Trigger>
          ))}
        </RadixTabs.List>
      )}
      {items.map((item) => (
        <RadixTabs.Content
          key={item.id}
          value={item.id}
          className={`flex flex-col gap-4 ${overflow ? 'min-h-0 flex-1' : ''}`}
        >
          {item.content}
        </RadixTabs.Content>
      ))}
    </RadixTabs.Root>
  );
}
