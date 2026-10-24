// The first-run guide (PRD FR-7.12): a small card at the end of the screen, over a corner of it, with
// the words of the step that is open and Back, Next and Skip. Next opens the screen the next step is
// about, so the Owner sees what the words say. It is not a dialog that blocks the screen: it can be
// left open while he looks around, and Esc closes it. The words are Fluent; the first-run guide
// starts once (see use-guide.ts), and the user menu shows it again.

import { Localized } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { useEffect, useRef } from 'react';
import { SHORTCUTS, useEscape } from '../../shared/keyboard';
import { Button, Code, Num } from '../../shared/ui';
import { useGuideStore } from './guide-store';
import { GUIDE_STEPS, type GuideStep } from './steps';

/** Opens the screen of a step. */
function useGoToStep() {
  const navigate = useNavigate();
  return (step: GuideStep) => {
    if (step.type === 'receipt')
      void navigate({ to: '/bikri/voucher', search: { type: 'receipt' } });
    else void navigate({ to: step.to });
  };
}

export function GuideCard() {
  const index = useGuideStore((state) => state.step);
  const open = useGuideStore((state) => state.open);
  const close = useGuideStore((state) => state.close);
  const goTo = useGoToStep();
  const next = useRef<HTMLButtonElement>(null);
  useEscape('overlay', close, index !== null);
  // Focus is on Next when a step opens, so Enter goes on.
  useEffect(() => {
    if (index !== null) next.current?.focus();
  }, [index]);
  if (index === null) return null;
  const step = GUIDE_STEPS[index];
  if (step === undefined) return null;
  const last = index === GUIDE_STEPS.length - 1;
  const move = (to: number) => {
    const target = GUIDE_STEPS[to];
    if (target === undefined) return;
    open(to);
    goTo(target);
  };
  return (
    <section
      aria-labelledby="guide-title"
      data-guide-step={step.id}
      className="fixed bottom-statusbar end-6 z-30 mb-4 flex w-full max-w-dialog flex-col gap-3 rounded-3 border border-border bg-surface p-5 text-ink shadow-overlay"
    >
      <p className="type-body text-ink-muted">
        <Localized
          id="guide-progress"
          vars={{ step: index + 1, total: GUIDE_STEPS.length }}
          elems={{ num: <Num /> }}
        >
          <span />
        </Localized>
      </p>
      <Localized id={`guide-step-${step.id}-title`}>
        <h2 id="guide-title" className="type-section" />
      </Localized>
      <Localized
        id={`guide-step-${step.id}-body`}
        vars={{ key: SHORTCUTS.save.label }}
        elems={{ key: <Code /> }}
      >
        <p className="type-body" />
      </Localized>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Button onClick={close}>
          <Localized id="guide-skip" />
        </Button>
        <div className="flex gap-2">
          {index === 0 ? null : (
            <Button
              onClick={() => {
                move(index - 1);
              }}
            >
              <Localized id="guide-back" />
            </Button>
          )}
          <Button
            variant="primary"
            ref={next}
            onClick={() => {
              if (last) close();
              else move(index + 1);
            }}
          >
            <Localized id={last ? 'guide-done' : 'guide-next'} />
          </Button>
        </div>
      </div>
    </section>
  );
}
