// Development page /dev/assistant (plan 49): the question panel over a screen with an unsaved entry,
// and the Settings group "سوال پوچھیں", answered by a stand-in for the backend with the backend's own
// fixtures (tests/helpers/dev-bridge-assistant.ts), so every state can be opened in a plain browser
// tab, used, and photographed in both languages and at every window size. The flags in the hash choose
// what is shown at once (see the stand-in's header). Never shipped: the route exists only when
// import.meta.env.DEV is true (app/router.tsx).

import { Localized } from '@fluent/react';
import { useEffect, useRef, useState } from 'react';
import { useLanguage } from '../../shared/i18n';
import { GalleryLocalization } from './GalleryLocalization';
import { installAssistantDevBridge } from '../../../tests/helpers/dev-bridge-assistant';
import { devValue } from '../../../tests/helpers/dev-bridge';
import { AssistantSettings } from '../../features/assistant';
import { useAssistantRunner } from '../../features/assistant/hooks';
import { useAssistantStore } from '../../features/assistant/store';
import type { QuickChip } from '../../bindings/QuickChip';
import { formNavigationProps, useEscapeDispatcher } from '../../shared/keyboard';
import { Card, DateInput, Field, MoneyInput, QtyInput, TextInput } from '../../shared/ui';
import { AppShell } from '../layout/AppShell';
import { MainNav } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';

const SAMPLE_DATE = '10-10-2026';

/** Opens the panel, and asks or shows what the flags say, once. */
function Scenario() {
  const runner = useAssistantRunner();
  const started = useRef(false);
  useEffect(() => {
    if (started.current) return;
    started.current = true;
    const store = useAssistantStore.getState();
    if (devValue('open') !== null || devValue('tall') !== null) store.openDock();
    if (devValue('tall') !== null) store.setTall(true);
    const ask = devValue('ask');
    const chip = devValue('chip');
    // Two step lines while a slow question works, as the backend's step events would give them.
    if (devValue('steps') !== null) {
      window.setTimeout(() => {
        store.addStep({
          id: 'assistant-step-finding',
          args: [{ name: 'text', value: { type: 'text', text: 'ikram' } }],
        });
        store.addStep({
          id: 'assistant-step-reading-ledger',
          args: [
            {
              name: 'name',
              value: { type: 'names', names: { ur: 'اکرم ٹول ٹیکس', en: 'Akram Toll Tax' } },
            },
          ],
        });
      }, 400);
    }
    if (ask !== null) runner.ask(ask);
    else if (chip !== null) runner.quick(chip as QuickChip, null);
    // The page is drawn twice at start-up in development (the panel forgets its conversation when it
    // goes away), so the scenario may run again.
    return () => {
      started.current = false;
    };
  }, [runner]);
  return null;
}

/** A screen with an unsaved entry in it, which the panel must leave alone. */
function SampleEntry() {
  const [amount, setAmount] = useState('15000');
  const [qty, setQty] = useState('50');
  const [date, setDate] = useState(SAMPLE_DATE);
  const [narration, setNarration] = useState('');
  return (
    <Card title={<Localized id="gallery-section-fields" />}>
      <form {...formNavigationProps} className="grid max-w-reading grid-cols-2 gap-x-5 gap-y-4">
        <Field label={<Localized id="gallery-field-amount" />}>
          {(control) => <MoneyInput {...control} value={amount} onChange={setAmount} />}
        </Field>
        <Field label={<Localized id="gallery-field-qty" />}>
          {(control) => <QtyInput {...control} value={qty} onChange={setQty} />}
        </Field>
        <Field label={<Localized id="gallery-field-date" />}>
          {(control) => <DateInput {...control} value={date} onChange={setDate} />}
        </Field>
        <Field label={<Localized id="gallery-field-narration" />}>
          {(control) => <TextInput {...control} value={narration} onChange={setNarration} />}
        </Field>
      </form>
    </Card>
  );
}

export function AssistantDevPage() {
  // The real app mounts the Esc dispatcher in its frame; this page has no frame, and Esc closes the
  // window (FR-7.18).
  useEscapeDispatcher();
  // The stand-in is installed before the screen asks its first question.
  useState(() => {
    installAssistantDevBridge();
    return true;
  });
  const settings = devValue('screen') === 'settings';
  const language = useLanguage();
  return (
    <AppShell
      nav={<MainNav current={settings ? 'settings' : 'bikri'} onSelect={() => undefined} />}
      moduleMenu={null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onGuide={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      <Scenario />
      {settings ? (
        <AssistantSettings />
      ) : (
        <GalleryLocalization language={language}>
          <SampleEntry />
        </GalleryLocalization>
      )}
    </AppShell>
  );
}
