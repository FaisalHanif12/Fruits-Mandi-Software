// Development page /dev/rokar (plan 17): the screens of the روکڑ area inside the app shell, answered
// by a stand-in for the backend with the books of the PRD worked day (tests/helpers/cashbook-books.ts),
// so they can be opened in a plain browser tab and photographed in both languages and at every window
// size. The flags in the hash choose the screen: `?screen=book|expense|fruit|lent|khata|general|capital|statement|overview|day`, `&type=...` the
// type of an entry screen, `&empty`, `&cash`, `&repeat` (see dev-bridge-rokar.ts). Never shipped: the
// route exists only when import.meta.env.DEV is true (app/router.tsx).

import { useState, type ReactElement } from 'react';
import { devValue, installDevBridge } from '../../../tests/helpers/dev-bridge';
import { CashbookPage } from '../../features/cashbook/pages/CashbookPage';
import { ExpensePage } from '../../features/cashbook/pages/ExpensePage';
import { KhataPage } from '../../features/cashbook/pages/KhataPage';
import { GeneralPage } from '../../features/cashbook/pages/GeneralPage';
import { LentPage } from '../../features/cashbook/pages/LentPage';
import { DayPage } from '../../features/day/pages/DayPage';
import { FruitPage } from '../../features/fruit/pages/FruitPage';
import { CapitalPage } from '../../features/partners/pages/CapitalPage';
import { PartnersOverviewPage } from '../../features/partners/pages/PartnersOverviewPage';
import { StatementPage } from '../../features/partners/pages/StatementPage';
import { AppShell } from '../layout/AppShell';
import { MainNav, type MainNavId } from '../layout/MainNav';
import { RokarMenu, type RokarMenuId } from '../layout/ModuleMenu';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

interface DevScreen {
  /** The item of the main menu that is current. */
  main: MainNavId;
  /** The item of the side menu of the روکڑ area, or null when the screen has no side menu. */
  id: RokarMenuId | null;
  page: ReactElement;
}

function screenOf(name: string | null): DevScreen {
  switch (name) {
    case 'general':
      return { main: 'rokar', id: 'general', page: <GeneralPage /> };
    case 'capital':
      return { main: 'partners', id: null, page: <CapitalPage /> };
    case 'statement':
      return { main: 'partners', id: null, page: <StatementPage /> };
    case 'overview':
      return { main: 'partners', id: null, page: <PartnersOverviewPage /> };
    case 'day':
      return { main: 'dashboard', id: null, page: <DayPage /> };
    case 'expense':
      return { main: 'rokar', id: 'expense', page: <ExpensePage /> };
    case 'lent':
      return { main: 'rokar', id: 'lent', page: <LentPage /> };
    case 'khata':
      return { main: 'rokar', id: 'khata', page: <KhataPage /> };
    case 'fruit':
      return { main: 'rokar', id: 'expense', page: <FruitPage /> };
    default:
      return { main: 'rokar', id: 'cashbook', page: <CashbookPage /> };
  }
}

export function RokarPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDevBridge();
    return true;
  });
  const { main, id, page } = screenOf(devValue('screen'));
  return (
    <AppShell
      nav={<MainNav current={main} onSelect={() => undefined} />}
      moduleMenu={id === null ? null : <RokarMenu current={id} onSelect={() => undefined} />}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      {page}
    </AppShell>
  );
}
