// A row of buttons that is always ONE line (plan 93 T6, Umar 22-11-2026): the dashboard's quick actions. The buttons keep
// their natural width, never stretched; the ones that do not fit the width go behind a "More" button at the end of the
// row, which opens a menu (arrows and Enter pick one, Esc closes). It is the same arithmetic and the same "More" button
// as the tabs of a section (`itemsThatFit`, `common-tabs-more`). Before the row has a width (a test, a window being
// built) every button is shown.
//
// The row wants nothing from its parent but its width: an in-flow ruler with no height holds every button, invisible
// and inert, and is measured to see what fits. A button of the ruler is a copy of the shown one, so the row never
// guesses a width.

import { Localized, useLocalization } from '@fluent/react';
import { ChevronDown } from 'lucide-react';
import { DropdownMenu } from 'radix-ui';
import { Fragment, useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { useUiLanguage } from '../i18n';
import { radixCollisionPadding } from './dropdown-position';
import { itemsThatFit } from './overflow-fit';

export interface OverflowItem {
  id: string;
  /** The words of the item in the "More" menu, already in the app language. */
  label: string;
  /** The button as it stands in the row (a shared `Button`). */
  node: ReactNode;
  /** What the item does, from the "More" menu. The button of the row has its own click. */
  onSelect: () => void;
}

interface OverflowRowProps {
  items: readonly OverflowItem[];
  /** The id of the heading that names the group, read by screen readers. */
  'aria-labelledby': string;
}

/** The space between two buttons, as the row's own gap (`gap-2` is 8 px). */
const GAP_PX = 8;

/** The look of the "More" button: a plain button of the shared kit (secondary), with a chevron. */
const MORE_CLASSES =
  'inline-flex h-control shrink-0 items-center gap-2 rounded-2 border border-border-control bg-surface px-4 type-control text-ink transition-calm';

export function OverflowRow({ items, 'aria-labelledby': labelledBy }: OverflowRowProps) {
  const { l10n } = useLocalization();
  const { language, direction } = useUiLanguage();
  const boxRef = useRef<HTMLDivElement>(null);
  const rulerRef = useRef<HTMLDivElement>(null);
  const moreRulerRef = useRef<HTMLSpanElement>(null);
  const [fit, setFit] = useState(items.length);

  useLayoutEffect(() => {
    const box = boxRef.current;
    const ruler = rulerRef.current;
    if (box === null || ruler === null) return undefined;
    const measure = () => {
      const widths = (Array.from(ruler.children) as HTMLElement[]).map((cell) => cell.offsetWidth);
      const more = moreRulerRef.current?.offsetWidth ?? 0;
      setFit(itemsThatFit(box.clientWidth, widths, more, GAP_PX));
    };
    measure();
    if (typeof ResizeObserver !== 'function') return undefined;
    const watcher = new ResizeObserver(measure);
    watcher.observe(box);
    return () => {
      watcher.disconnect();
    };
  }, [items, language]);

  const shown = items.slice(0, fit);
  const hidden = items.slice(fit);

  return (
    <div ref={boxRef} role="group" aria-labelledby={labelledBy} className="flex min-w-0 flex-col">
      {/* The ruler: every button and a "More" button, invisible and inert, with no height. */}
      <div
        aria-hidden="true"
        inert
        className="pointer-events-none invisible relative h-0 overflow-hidden whitespace-nowrap"
      >
        <div ref={rulerRef} className="flex w-max gap-2">
          {items.map((item) => (
            <div key={item.id} className="shrink-0">
              {item.node}
            </div>
          ))}
        </div>
        <span ref={moreRulerRef} className={`${MORE_CLASSES} absolute`}>
          <Localized id="common-tabs-more" />
          <ChevronDown className="size-icon-md" />
        </span>
      </div>
      <div className="flex min-w-0 items-center gap-2">
        {shown.map((item) => (
          <Fragment key={item.id}>{item.node}</Fragment>
        ))}
        {hidden.length === 0 ? null : (
          <DropdownMenu.Root dir={direction}>
            <DropdownMenu.Trigger className={`${MORE_CLASSES} data-[state=open]:bg-surface-sunken`}>
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
                {hidden.map((item) => (
                  <DropdownMenu.Item
                    key={item.id}
                    onSelect={item.onSelect}
                    className="type-body flex h-control cursor-pointer items-center rounded-1 px-3 text-ink data-[highlighted]:bg-primary-soft"
                  >
                    {item.label}
                  </DropdownMenu.Item>
                ))}
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
        )}
      </div>
    </div>
  );
}
