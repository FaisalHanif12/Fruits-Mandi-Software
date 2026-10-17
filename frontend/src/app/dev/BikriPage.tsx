// Development page /dev/bikri (plans 12 and 13): the real بکری screens inside the app shell,
// answered by a stand-in for the backend with the numbers of Event 1 (PRD 4.4), so they can be
// opened in a plain browser tab and photographed in both languages and at every window size. The
// flags in the hash choose the screen: none is the sale book; `?id=41` a saved بکری; `?today` the
// list; `?pukhta` the register; `?offer` adds the start-up offer. Never shipped: the route exists
// only when import.meta.env.DEV is true (app/router.tsx).

import { Localized } from '@fluent/react';
import { useState } from 'react';
import { devFlag, installDevBridge, seedEvent1 } from '../../../tests/helpers/dev-bridge';
import { paisaForTest } from '../../../tests/helpers/paisa';
import { BikriDraftOffer } from '../../features/bikri/draft-offer';
import { BikriBookPage } from '../../features/bikri/pages/BikriBookPage';
import { TodayBikriPage } from '../../features/bikri/pages/TodayBikriPage';
import { PrintingTab } from '../../features/settings/components/PrintingTab';
import { PukhtaRegisterPage } from '../../features/pukhta_khata/pages/PukhtaRegisterPage';
import { useLanguage } from '../../shared/i18n';
import { Button, CashConfirmDialog } from '../../shared/ui';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { ModuleMenu } from '../layout/ModuleMenu';
import { GalleryLocalization } from './GalleryLocalization';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';
/** The sample cash after the entry shown in the confirm box: -5,000 rupees. */
const SAMPLE_CASH_AFTER = paisaForTest(-500_000);

export function BikriPage() {
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installDevBridge();
    return true;
  });
  const [cashBox, setCashBox] = useState(false);
  const language = useLanguage();
  return (
    <AppShell
      nav={<MainNav current="bikri" onSelect={() => undefined} />}
      moduleMenu={
        <ModuleMenu
          current={devFlag('today') ? 'today' : devFlag('pukhta') ? 'pukhta' : 'bikri-book'}
          onSelect={() => undefined}
        />
      }
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      {devFlag('cash') ? (
        // Plan 13 opens this box on save; here a button opens it so it can be photographed.
        <div className="mb-3">
          {/* The words of this page are in the development bundle, not the app's. */}
          <GalleryLocalization language={language}>
            <Button
              onClick={() => {
                setCashBox(true);
              }}
            >
              <Localized id="bikripage-open-cash-box" />
            </Button>
          </GalleryLocalization>
          <CashConfirmDialog
            open={cashBox}
            onOpenChange={setCashBox}
            cashAfter={SAMPLE_CASH_AFTER}
            onConfirm={() => undefined}
          />
        </div>
      ) : null}
      {devFlag('offer') ? <BikriDraftOffer /> : null}
      {devFlag('printing') ? (
        <PrintingTab />
      ) : devFlag('today') ? (
        <TodayBikriPage />
      ) : devFlag('pukhta') ? (
        <PukhtaRegisterPage />
      ) : (
        <BikriBookPage initialDraft={devFlag('empty') ? undefined : seedEvent1} />
      )}
    </AppShell>
  );
}
