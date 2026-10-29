// The page buttons under every list that loads page by page (PRD FR-19.11, plan 38 R1): the rows on
// this page ("26 to 50 of 340"), the rows per page (25, 50 or 100), "Page 2 of 14", and First,
// Previous, Next and Last. Built once here; no screen draws its own. The numbers are the backend's
// count of the whole filter. A list that fits one page of 25 shows only its count, so a short list
// is not cluttered. Page Up and Page Down in the table do the same as Previous and Next
// (`ListGrid`, PRD FR-7.17).

import { Localized, useLocalization } from '@fluent/react';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';
import { useId } from 'react';
import { DEFAULT_PAGE_SIZE, PAGE_SIZES, type PageSize, type PagerState } from '../hooks';
import { Button, Num, RUNNING_NUMBER, Select } from '../ui';

interface PagerProps {
  pager: PagerState;
  /**
   * The short form for a place inside a list, such as the page row under the accounts of an opened
   * control in the chart: the count and the Previous and Next buttons only.
   */
  compact?: boolean;
}

// The number is a figure inside the Urdu screen: isolated, in Inter (Code Standards F25).
const SIZE_OPTIONS = PAGE_SIZES.map((size) => ({
  value: String(size),
  label: <Num>{String(size)}</Num>,
}));

interface Step {
  key: string;
  labelId: string;
  icon: typeof ChevronLeft;
  disabled: boolean;
  to: number;
}

/** The icon buttons that turn the page. The arrows turn round in right-to-left (F26). */
function StepButtons({ steps, pager }: { steps: readonly Step[]; pager: PagerState }) {
  const { l10n } = useLocalization();
  return (
    <div className="flex gap-2">
      {steps.map(({ key, labelId, icon: Icon, disabled, to }) => (
        <Button
          key={key}
          iconOnly
          disabled={disabled}
          onClick={() => {
            pager.goTo(to);
          }}
        >
          <Icon aria-hidden="true" className="size-icon-md rtl:rotate-180" />
          <span className="sr-only">{l10n.getString(labelId)}</span>
        </Button>
      ))}
    </div>
  );
}

export function Pager({ pager, compact = false }: PagerProps) {
  const { l10n } = useLocalization();
  const sizeId = useId();
  const { page, pageCount, pageSize, total, from, to } = pager;
  // One page of the smallest size holds the whole filter: nothing to turn.
  const turns = total > DEFAULT_PAGE_SIZE;
  const all: Step[] = [
    {
      key: 'first',
      labelId: 'common-pager-first',
      icon: ChevronsLeft,
      disabled: page <= 1,
      to: 1,
    },
    {
      key: 'previous',
      labelId: 'common-pager-previous',
      icon: ChevronLeft,
      disabled: page <= 1,
      to: page - 1,
    },
    {
      key: 'next',
      labelId: 'common-pager-next',
      icon: ChevronRight,
      disabled: page >= pageCount,
      to: page + 1,
    },
    {
      key: 'last',
      labelId: 'common-pager-last',
      icon: ChevronsRight,
      disabled: page >= pageCount,
      to: pageCount,
    },
  ];
  const steps = compact
    ? all.filter((step) => step.key === 'previous' || step.key === 'next')
    : all;
  return (
    <nav
      aria-label={l10n.getString('common-pager-label')}
      aria-busy={pager.busy}
      className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2"
    >
      <Localized
        id="common-pager-range"
        vars={{ from: String(from), to: String(to), total: String(total) }}
        elems={RUNNING_NUMBER}
      >
        <p className="type-body text-ink-muted" />
      </Localized>
      {compact && turns ? <StepButtons steps={steps} pager={pager} /> : null}
      {!compact && turns ? (
        <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
          <div className="flex items-center gap-2">
            <label htmlFor={sizeId} className="type-body text-ink-muted">
              <Localized id="common-pager-size" />
            </label>
            <div className="w-pager-size">
              <Select<string>
                id={sizeId}
                value={String(pageSize)}
                options={SIZE_OPTIONS}
                onChange={(value) => {
                  pager.setPageSize(Number(value) as PageSize);
                }}
              />
            </div>
          </div>
          <Localized
            id="common-pager-page"
            vars={{ page: String(page), pages: String(pageCount) }}
            elems={RUNNING_NUMBER}
          >
            <p className="type-body text-ink" />
          </Localized>
          <StepButtons steps={steps} pager={pager} />
        </div>
      ) : null}
    </nav>
  );
}
