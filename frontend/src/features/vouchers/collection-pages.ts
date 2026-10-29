// The pages of the اگراہی فہرست joined into one list (PRD FR-15.6, FR-7.61, plan 38 B1). The round is
// a data-entry list: the Owner types an amount for every buyer and saves them together, so the
// screen needs every buyer of the filter. The backend counts buyers down the groups, so a group can
// continue on the next page; its total is the whole group's, so the first one is kept. Pure: no
// amount is worked out here.

import type { CollectionGroup } from '../../bindings/CollectionGroup';
import type { CollectionList } from '../../bindings/CollectionList';

/**
 * Joins the pages of one list: the groups in the order they came, a group that continues on the next
 * page getting its buyers added. Returns undefined for no page at all.
 */
export function joinCollectionPages(pages: readonly CollectionList[]): CollectionList | undefined {
  const first = pages[0];
  if (first === undefined) return undefined;
  const groups: CollectionGroup[] = [];
  for (const page of pages) {
    for (const group of page.groups) {
      const index = groups.findIndex(
        (known) => known.collector?.id === group.collector?.id && known.city?.id === group.city?.id,
      );
      const known = groups[index];
      if (known === undefined) groups.push({ ...group, buyers: [...group.buyers] });
      else groups[index] = { ...known, buyers: [...known.buyers, ...group.buyers] };
    }
  }
  const buyers = groups.reduce((count, group) => count + group.buyers.length, 0);
  return { ...first, groups, offset: 0, limit: buyers };
}
