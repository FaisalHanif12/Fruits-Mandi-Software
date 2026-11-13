// The quiet card that holds the filters of a screen (plan 75, item 5): a white panel with a thin border
// and a little room inside, so the search box, the dates and the ticks read as one tool above the list
// and not as loose fields on the page. It adds no behaviour: it is a `div`, and everything a `div`
// takes (a role, a label, the keys of a form, a ref) passes through to it.

import type { ComponentPropsWithRef } from 'react';

export function FilterCard({ className = '', ...rest }: ComponentPropsWithRef<'div'>) {
  return <div {...rest} className={`rounded-3 border border-border bg-surface p-3 ${className}`} />;
}
