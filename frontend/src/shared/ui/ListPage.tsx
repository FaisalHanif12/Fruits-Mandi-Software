// The frame of a list screen (plan 38 G2, G3, G5, Code Standards F12): the name of the screen with
// its help behind an ⓘ, the screen's buttons at the end of the same row, one filter bar, and then the
// list, which takes all the height that is left. The name and the filters stay; the table scrolls
// inside its own area with its heading, its totals row and its page buttons fixed (`PagedList`). The
// same frame on every list, so the same action is in the same place on every screen.
//
//   <ListPage titleId="accounts-title" help={<Localized id="accounts-help"><p /></Localized>}
//             actions={<Button .../>} filters={<FilterBar .../>}>
//     <PagedList .../>
//   </ListPage>

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import { InfoHint } from './InfoHint';

interface ListPageProps {
  /** The Fluent message ID of the screen's name, drawn as the page heading. */
  titleId: string;
  /** What the screen is for, behind an ⓘ next to the name. One line, not a paragraph. */
  help?: ReactNode;
  /** The buttons of the screen, at the end of the name's row (the primary one last). */
  actions?: ReactNode;
  /** The filter bar under the name. */
  filters?: ReactNode;
  /** The list, its notes and its states. It is given all the room that is left. */
  children: ReactNode;
}

export function ListPage({ titleId, help, actions, filters, children }: ListPageProps) {
  return (
    <div className="flex h-full min-h-0 flex-col gap-3 short:gap-2">
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <div className="flex items-center gap-1">
          <Localized id={titleId}>
            <h1 className="type-title" />
          </Localized>
          {help === undefined ? null : <InfoHint>{help}</InfoHint>}
        </div>
        {actions === undefined ? null : <div className="flex flex-wrap gap-2">{actions}</div>}
      </div>
      {filters === undefined ? null : <div className="shrink-0">{filters}</div>}
      <div className="flex min-h-0 flex-1 flex-col gap-3">{children}</div>
    </div>
  );
}
