// The icon in front of a line of the attention list, one for each kind (PRD FR-7.44). An icon never
// stands alone: the words of the line say the same (NFR-USE-12). A switch, so each kind is a plain
// component (the icons are drawn at one size, 20 px, in buttons and lists).

import {
  AlarmClock,
  ArchiveRestore,
  CalendarClock,
  CloudOff,
  CreditCard,
  FileClock,
  HandCoins,
  PackageOpen,
  Tag,
  WalletCards,
} from 'lucide-react';
import type { AttentionKind } from '../../../bindings/AttentionKind';

const CLASSES = 'size-icon-md shrink-0 text-warn';

export function KindIcon({ kind }: { kind: AttentionKind }) {
  switch (kind) {
    case 'drafts':
      return <FileClock aria-hidden="true" className={CLASSES} />;
    case 'open_days':
      return <CalendarClock aria-hidden="true" className={CLASSES} />;
    case 'stock_waiting':
      return <PackageOpen aria-hidden="true" className={CLASSES} />;
    case 'over_credit_limit':
      return <CreditCard aria-hidden="true" className={CLASSES} />;
    case 'dues_old':
      return <WalletCards aria-hidden="true" className={CLASSES} />;
    case 'payables_old':
      return <HandCoins aria-hidden="true" className={CLASSES} />;
    case 'backup_old':
      return <CloudOff aria-hidden="true" className={CLASSES} />;
    case 'loans_overdue':
      return <AlarmClock aria-hidden="true" className={CLASSES} />;
    case 'takes_overdue':
      return <ArchiveRestore aria-hidden="true" className={CLASSES} />;
    case 'names_missing':
      return <Tag aria-hidden="true" className={CLASSES} />;
  }
}
