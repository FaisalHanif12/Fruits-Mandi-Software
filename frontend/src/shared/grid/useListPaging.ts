// What a list grid needs to turn the page with Page Up and Page Down (PRD FR-7.17, plan 38 R1): the
// key asks for the next or the previous page, and focus goes to the first row of the page that
// arrives. Used by `PagedList`, and by a screen that draws its grid itself.

import { useState } from 'react';
import type { PagerState } from '../hooks';
import type { ListPaging } from './ListGrid';

export function useListPaging(pager: PagerState): ListPaging {
  // Counts the page turns made with the keys, so the grid puts focus on the first row of the new page.
  const [turns, setTurns] = useState(0);
  return {
    onPage: (direction) => {
      const target = direction === 'next' ? pager.page + 1 : pager.page - 1;
      if (target < 1 || target > pager.pageCount) return;
      setTurns((count) => count + 1);
      pager.goTo(target);
    },
    focusFirst: turns,
    ready: !pager.busy,
  };
}
