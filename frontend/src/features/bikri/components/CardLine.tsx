// One row of a card in the summary: a label and a figure at the end, with the working of the figure behind its ⓘ. The
// seller cards (`SellerPaid`) and the garden and trip cards (`VentureCards`) draw their rows with it, so they look the same
// (PRD FR-12.52, plan 97 item 5). The figure is the backend's; nothing is worked out here (CLAUDE.md rule 1).

import { Localized } from '@fluent/react';
import type { ReactNode } from 'react';
import type { Working } from '../../../bindings/Working';
import { CalcInfo } from '../../../shared/ui';

interface CardLineProps {
  labelId: string;
  working?: Working;
  children: ReactNode;
  emphasis?: boolean;
}

export function CardLine({ labelId, working, children, emphasis = false }: CardLineProps) {
  return (
    <div
      className={`flex items-center justify-between gap-3 border-b border-border py-1 ${emphasis ? 'type-label' : 'type-body'} text-ink`}
    >
      <Localized id={labelId}>
        <dt />
      </Localized>
      <dd className="value-col">
        {children}
        {working === undefined ? null : <CalcInfo working={working} inCell />}
      </dd>
    </div>
  );
}
