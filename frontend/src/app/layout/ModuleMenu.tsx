// The second menu of the بکری area (PRD FR-5.3) and of the روکڑ area (PRD 5, plan 17): the items, in order,
// and the tabs that show them. Since v1.56 (plan 93) they are not a side panel but a row of tabs in the title row
// of the screen (`shared/ui/SectionTabs`): one component for every section, so the same row looks and works the same
// in both. The shell gives the row to the screen through `SectionTabsProvider`; a screen says where its title row
// is with `TitleOrTabs` and builds nothing of its own.

import { useLocalization } from '@fluent/react';
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  BookText,
  BookUser,
  Coins,
  FileSignature,
  Handshake,
  History,
  NotebookPen,
  NotebookTabs,
  Package,
  Receipt,
  Wallet,
  type LucideIcon,
} from 'lucide-react';
import { SectionTabs } from '../../shared/ui';

export type ModuleMenuId =
  'bikri-book' | 'receipt' | 'payment' | 'advance' | 'today' | 'party-ledgers' | 'stock';

interface ModuleEntry {
  id: ModuleMenuId;
  labelId: string;
  icon: LucideIcon;
}

/** PRD FR-5.3, in this order. پارٹی کھاتے sits above باقی مال (PRD FR-12.43). */
export const BIKRI_MENU: readonly ModuleEntry[] = [
  { id: 'bikri-book', labelId: 'layout-module-bikri-book', icon: NotebookPen },
  { id: 'receipt', labelId: 'layout-module-receipt', icon: ArrowDownToLine },
  { id: 'payment', labelId: 'layout-module-payment', icon: ArrowUpFromLine },
  { id: 'advance', labelId: 'layout-module-advance', icon: Coins },
  { id: 'today', labelId: 'layout-module-today', icon: History },
  { id: 'party-ledgers', labelId: 'layout-module-party-ledgers', icon: BookUser },
  { id: 'stock', labelId: 'layout-module-stock', icon: Package },
];

/** The side menu of the روکڑ area (PRD 5, Architecture 5): the cash book and the entry screens. */
export type RokarMenuId = 'cashbook' | 'expense' | 'lent' | 'khata' | 'general' | 'voucher';

/**
 * PRD 5 (the روکڑ branch), in this order. پھل (fruit taken) comes next to خرچہ with plan 17d, and
 * is one more entry here and one more route. جنرل is the fifth screen (plan 17).
 */
export const ROKAR_MENU: readonly { id: RokarMenuId; labelId: string; icon: LucideIcon }[] = [
  { id: 'cashbook', labelId: 'layout-rokar-cashbook', icon: Wallet },
  { id: 'expense', labelId: 'layout-rokar-expense', icon: Receipt },
  { id: 'lent', labelId: 'layout-rokar-lent', icon: Handshake },
  { id: 'khata', labelId: 'layout-rokar-khata', icon: NotebookTabs },
  { id: 'general', labelId: 'layout-rokar-general', icon: BookText },
  { id: 'voucher', labelId: 'layout-rokar-voucher', icon: FileSignature },
];

interface SectionRowProps<Id extends string> {
  entries: readonly { id: Id; labelId: string; icon: LucideIcon }[];
  /** The Fluent message ID of the row's name, read by screen readers. */
  labelId: string;
  current: Id;
  onSelect: (id: Id) => void;
}

function SectionRow<Id extends string>({
  entries,
  labelId,
  current,
  onSelect,
}: SectionRowProps<Id>) {
  const { l10n } = useLocalization();
  return (
    <SectionTabs
      tabs={entries.map((entry) => ({
        id: entry.id,
        label: l10n.getString(entry.labelId),
        icon: entry.icon,
      }))}
      label={l10n.getString(labelId)}
      current={current}
      onSelect={onSelect}
    />
  );
}

interface ModuleTabsProps {
  current: ModuleMenuId;
  onSelect: (id: ModuleMenuId) => void;
}

/** The tabs of the بکری area (PRD FR-5.3). */
export function ModuleTabs({ current, onSelect }: ModuleTabsProps) {
  return (
    <SectionRow
      entries={BIKRI_MENU}
      labelId="layout-module-label"
      current={current}
      onSelect={onSelect}
    />
  );
}

interface RokarTabsProps {
  current: RokarMenuId;
  onSelect: (id: RokarMenuId) => void;
}

/** The tabs of the روکڑ area (PRD 5, PRD 16). */
export function RokarTabs({ current, onSelect }: RokarTabsProps) {
  return (
    <SectionRow
      entries={ROKAR_MENU}
      labelId="layout-rokar-label"
      current={current}
      onSelect={onSelect}
    />
  );
}
