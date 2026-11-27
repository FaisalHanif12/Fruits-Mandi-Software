// The second menu of the بکری area (PRD FR-5.3) and of the روکڑ area (PRD 5, plan 17): the items, in order,
// and the tabs that show them. Since v1.56 (plan 93) they are not a side panel but a row of tabs in the title row
// of the screen (`shared/ui/SectionTabs`): one component for every section, so the same row looks and works the same
// in both. Since v1.57 (plan 95) وصولی, ادائیگی and پیشگی are three tabs of روکڑ, not of بکری. The shell gives the row to the screen through `SectionTabsProvider`; a screen says where its title row
// is with `TitleOrTabs` and builds nothing of its own.

import { useLocalization } from '@fluent/react';
import {
  ArrowDownToLine,
  ArrowUpFromLine,
  BookText,
  BookUser,
  Coins,
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

export type ModuleMenuId = 'bikri-book' | 'party-ledgers' | 'today' | 'stock';

interface ModuleEntry {
  id: ModuleMenuId;
  labelId: string;
  icon: LucideIcon;
}

/**
 * PRD FR-5.3 (v1.57), in this order: بکری بک, پارٹی کھاتے (PRD FR-12.43), پچھلی بکری, باقی مال. The tabs
 * that do not fit a narrow window go behind "More" (`SectionTabs`).
 */
export const BIKRI_MENU: readonly ModuleEntry[] = [
  { id: 'bikri-book', labelId: 'layout-module-bikri-book', icon: NotebookPen },
  { id: 'party-ledgers', labelId: 'layout-module-party-ledgers', icon: BookUser },
  { id: 'today', labelId: 'layout-module-today', icon: History },
  { id: 'stock', labelId: 'layout-module-stock', icon: Package },
];

/**
 * The tabs of the روکڑ area (PRD 5, FR-5.3 v1.57). وصولی, ادائیگی and پیشگی are three tabs that open the one
 * voucher screen with that type already chosen (PRD FR-7.67, FR-15.10); the others are screens of their own.
 */
export type RokarMenuId =
  'cashbook' | 'receipt' | 'payment' | 'advance' | 'expense' | 'lent' | 'khata' | 'general';

/**
 * PRD 5 (the روکڑ branch), in this order: the cash book, the three voucher types, then خرچہ, ادھار رقم,
 * الگ کھاتہ and جنرل. پھل (fruit taken) is a tab of خرچہ (plan 17d), not an item here.
 */
export const ROKAR_MENU: readonly { id: RokarMenuId; labelId: string; icon: LucideIcon }[] = [
  { id: 'cashbook', labelId: 'layout-rokar-cashbook', icon: Wallet },
  { id: 'receipt', labelId: 'layout-rokar-receipt', icon: ArrowDownToLine },
  { id: 'payment', labelId: 'layout-rokar-payment', icon: ArrowUpFromLine },
  { id: 'advance', labelId: 'layout-rokar-advance', icon: Coins },
  { id: 'expense', labelId: 'layout-rokar-expense', icon: Receipt },
  { id: 'lent', labelId: 'layout-rokar-lent', icon: Handshake },
  { id: 'khata', labelId: 'layout-rokar-khata', icon: NotebookTabs },
  { id: 'general', labelId: 'layout-rokar-general', icon: BookText },
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
