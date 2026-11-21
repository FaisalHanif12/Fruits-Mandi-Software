// The headline of the capital card and its net line (PRD FR-16.14, plan 87). The headline names the figure, the
// partners' total capital, says that both sides are equal (✓) or that they are not, and says the day it is as of.
// The net line closes the two blocks. Both figures are the backend's, unsigned, written on the physical right edge
// of the card (NFR-USE-19, F17): when the shop holds less than it owes the backend says `owe` and the figure carries
// the word "short", never a minus sign. Nothing is worked out here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import { CircleAlert, CircleCheck } from 'lucide-react';
import { isoToDisplayDate, type Paisa } from '../../../../shared/format';
import { Num, StatusChip } from '../../../../shared/ui';
import type { CapitalSide } from '../../capital-types';

interface NetFigureProps {
  net: Paisa;
  side: CapitalSide;
}

/** The figure and, when the shop holds less than it owes, the word that says so. */
function NetFigure({ net, side }: NetFigureProps) {
  return (
    <>
      <Num inherit paisa={net} />
      {side === 'owe' ? (
        <Localized id="cashbook-capital-net-owe">
          <span className="type-label" />
        </Localized>
      ) : null}
    </>
  );
}

interface CapitalHeadlineProps extends NetFigureProps {
  /**
   * `equal`: the backend says the two sides are equal and nothing stands apart (✓). `unequal`: it says they are not.
   * `none`: an opening difference stands apart from the partners' lines, so the warning below tells the story and
   * the headline claims nothing (the backend's advice: the ✓ only when there is no difference).
   */
  mark: 'equal' | 'unequal' | 'none';
  /** The day the card is as of, `YYYY-MM-DD`. */
  asOfDate: string;
}

export function CapitalHeadline({ net, side, mark, asOfDate }: CapitalHeadlineProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 rounded-3 border border-border bg-surface-sunken p-4">
      <div className="flex min-w-0 flex-col gap-2">
        <Localized id="cashbook-capital-headline">
          <h3 className="type-section text-ink" />
        </Localized>
        <div className="flex flex-wrap items-center gap-2">
          {mark === 'equal' ? (
            <StatusChip tone="primary" icon={CircleCheck}>
              <Localized id="cashbook-capital-equal" />
            </StatusChip>
          ) : null}
          {mark === 'unequal' ? (
            <StatusChip tone="due" icon={CircleAlert}>
              <Localized id="cashbook-capital-unequal" />
            </StatusChip>
          ) : null}
          <Localized id="cashbook-capital-as-of" vars={{ date: isoToDisplayDate(asOfDate) }}>
            <span className="type-label text-ink-muted" />
          </Localized>
        </div>
      </div>
      <div className="value-col type-figure text-ink">
        <NetFigure net={net} side={side} />
      </div>
    </div>
  );
}

/** صافی: what is had less what is owed, under the two blocks. */
export function CapitalNet({ net, side }: NetFigureProps) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-2 border border-border bg-surface-sunken px-3 py-2">
      <Localized id="cashbook-capital-net-label">
        <span className="type-field text-ink" />
      </Localized>
      <div className="value-col type-field text-ink">
        <NetFigure net={net} side={side} />
      </div>
    </div>
  );
}
