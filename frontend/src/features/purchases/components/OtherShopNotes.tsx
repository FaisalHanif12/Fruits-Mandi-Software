// The two short texts of a sale in another shop (plan 90): what the qty field says when it is more
// than the line has for another shop, and what the list of lines says when no line has goods for
// another shop. Words only, from Fluent; the numbers come from the backend.

import { Localized } from '@fluent/react';
import { Notice } from '../../../shared/ui';

/** Under the qty field: only `remaining` نگ are left for another shop on the chosen line. */
export function OtherShopQtyError({ remaining }: { remaining: number }) {
  return (
    <Localized id="purchases-qty-above-other" vars={{ remaining: String(remaining) }}>
      <span className="block" />
    </Localized>
  );
}

/** In place of the list of lines: no line has goods left for another shop, and how to set some. */
export function OtherShopNone({ kind }: { kind: 'arrival' | 'purchase' }) {
  return (
    <Notice tone="info">
      <Localized id="purchases-other-goods-none">
        <span className="font-semibold" />
      </Localized>{' '}
      <Localized id={`purchases-other-goods-none-hint-${kind}`}>
        <span />
      </Localized>
    </Notice>
  );
}
