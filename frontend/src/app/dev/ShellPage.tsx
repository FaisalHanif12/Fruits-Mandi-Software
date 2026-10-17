// Development page /dev/shell (plan 04): the app shell with the main menu, the بکری side menu,
// the top bar, the status bar and a small form, to try the arrow keys, Ctrl+K, text size and the
// language switch in both layouts. Sample values only. Never shipped: the route exists only when
// import.meta.env.DEV is true (app/router.tsx).

import { Localized } from '@fluent/react';
import { useState } from 'react';
import { SHORTCUTS, formNavigationProps } from '../../shared/keyboard';
import {
  Button,
  Card,
  DateInput,
  Field,
  MoneyInput,
  QtyInput,
  SaveChangesPrompt,
  TextInput,
} from '../../shared/ui';
import { useLanguage } from '../../shared/i18n';
import { GalleryLocalization } from './GalleryLocalization';
import { AppShell } from '../layout/AppShell';
import { MainNav, type MainNavId } from '../layout/MainNav';
import { SAMPLE_USER } from './sample';
import { ModuleMenu, type ModuleMenuId } from '../layout/ModuleMenu';

const SAMPLE_DATE = '10-10-2026';

function SampleForm() {
  const [narration, setNarration] = useState('');
  const [amount, setAmount] = useState('15000');
  const [date, setDate] = useState(SAMPLE_DATE);
  const [qty, setQty] = useState('50');
  const [promptOpen, setPromptOpen] = useState(false);
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
        <div>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            onClick={() => {
              setPromptOpen(true);
            }}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </form>
      <SaveChangesPrompt
        open={promptOpen}
        onOpenChange={setPromptOpen}
        onSave={() => undefined}
        onLeave={() => undefined}
      />
    </Card>
  );
}

function Content() {
  const [main, setMain] = useState<MainNavId>('bikri');
  const [module, setModule] = useState<ModuleMenuId>('bikri-book');
  return (
    <AppShell
      nav={<MainNav current={main} onSelect={setMain} />}
      moduleMenu={main === 'bikri' ? <ModuleMenu current={module} onSelect={setModule} /> : null}
      userName={SAMPLE_USER}
      onLock={() => undefined}
      onSignOut={() => undefined}
      onOpenAccount={() => undefined}
      onOpenBikri={() => undefined}
      onOpenVoucher={() => undefined}
      date={SAMPLE_DATE}
      day="open"
    >
      <SampleForm />
    </AppShell>
  );
}

export function ShellPage() {
  const language = useLanguage();
  return (
    <GalleryLocalization language={language}>
      <Content />
    </GalleryLocalization>
  );
}
