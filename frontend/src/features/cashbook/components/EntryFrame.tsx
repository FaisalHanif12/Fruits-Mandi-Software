// The frame the three entry screens of the روکڑ area share (PRD FR-7.67, FR-7.9, FR-6.7, Code
// Standards F12, F27): the title, the type list, the buttons, the refusal of a save, the leave
// guard, the new-entry prompt and the cash question. The screen gives it the form, what to show
// below the form, and how to save; the keys come from here, so the three screens work the same.
//
// - Keys: F10 or Ctrl+S saves, F2 starts a new entry (asking first if there is typed work), Esc
//   goes back to the cash book (asking first if there is unsaved work). Enter and the arrows come
//   from shared/keyboard (F27).
// - Everything typed is text in the screen's state, never in this frame, so a language switch
//   loses nothing (F39).
// - The cash question (PRD FR-6.7) is the shared box: Yes saves, No goes back to the amount.

import { Localized, useLocalization } from '@fluent/react';
import { useNavigate } from '@tanstack/react-router';
import { Plus } from 'lucide-react';
import type { ReactNode, RefObject } from 'react';
import { useRef, useState } from 'react';
import { LeaveGuard } from '../../../app/LeaveGuard';
import type { UiError } from '../../../bindings/UiError';
import type { Paisa } from '../../../shared/format';
import { useRefusalIntoView } from '../../../shared/hooks';
import { SHORTCUTS, useEscape, useFormNavigation, useShortcut } from '../../../shared/keyboard';
import {
  Button,
  CashConfirmDialog,
  ErrorNotice,
  Field,
  InfoHint,
  SaveChangesPrompt,
  SegmentedControl,
} from '../../../shared/ui';

/** The cash question of the save flow, as the frame needs it. */
export interface CashFlowView {
  ask: boolean;
  cashAfter: Paisa | null;
  confirmCash: () => void;
  /** The box closed. True after a "no", when the cursor goes back to the amount. */
  onCashOpenChange: (open: boolean) => boolean;
}

export interface TypeChoice<T extends string> {
  value: T;
  /** The Fluent message ID of the type's name. */
  labelId: string;
}

interface EntryFrameProps<T extends string> {
  /** The Fluent message IDs of the title and the help line. */
  titleId: string;
  helpId: string;
  /** The Fluent message ID of the type list's name, read by screen readers. */
  typeLabelId?: string | undefined;
  /** The tab strip of the area (خرچہ and پھل), drawn under the title (plan 38 G7), not above it. */
  tabs?: ReactNode;
  /** The type list. A screen with only one kind of entry (the جنرل screen) has none. */
  types?: readonly TypeChoice<T>[] | undefined;
  type?: T | undefined;
  onType?: ((type: T) => void) | undefined;
  /** The address of this screen: a change of type stays on it, so it is not "leaving". */
  path: string;
  /** Something has been typed, so leaving or starting a new entry asks first (PRD FR-7.9). */
  filled: boolean;
  /**
   * Save stays switched off while this is true. Default: while nothing is typed. The جنرل screen
   * keeps it off until the backend says the entry is balanced (PRD 16.4).
   */
  saveBlocked?: boolean | undefined;
  /** Where Esc goes when the entry is left. Default: the cash book. */
  leaveTo?: string | undefined;
  /** A save is on its way. */
  busy: boolean;
  /** Saves the form. Resolves true when it was saved. */
  onSave: () => Promise<boolean>;
  /** Starts a new, empty entry of the same type (the user already agreed to lose what he typed). */
  onNew: () => void;
  flow: CashFlowView;
  /** What the backend refused, until the draft changes. */
  error: UiError | null;
  /** The field paths the form shows next to their inputs, so no problem is told twice. */
  fieldsShown: readonly string[];
  formRef: RefObject<HTMLFormElement | null>;
  /** More buttons next to New and Save, for example "repeat last month". */
  actions?: ReactNode;
  /** The fields and the notes under them. */
  children: ReactNode;
  /**
   * Content that sits between the form and the list and is not part of the form, such as the line
   * grid of the جنرل screen. It has its own keys, so the form's arrow keys never see it.
   */
  between?: ReactNode;
  /** The list under the form. */
  below: ReactNode;
}

export function EntryFrame<T extends string>({
  titleId,
  helpId,
  tabs,
  typeLabelId,
  types,
  type,
  onType,
  path,
  filled,
  saveBlocked,
  leaveTo = '/rokar',
  busy,
  onSave,
  onNew,
  flow,
  error,
  fieldsShown,
  formRef,
  actions,
  children,
  between,
  below,
}: EntryFrameProps<T>) {
  const { l10n } = useLocalization();
  const navigate = useNavigate();
  const rootRef = useRef<HTMLDivElement>(null);
  const [askNew, setAskNew] = useState(false);
  useRefusalIntoView(rootRef, error);

  const saveNow = () => {
    void onSave();
  };
  const navigation = useFormNavigation(saveNow);
  useShortcut(SHORTCUTS.save, saveNow);

  const requestNew = () => {
    if (filled) setAskNew(true);
    else onNew();
  };
  useShortcut(SHORTCUTS.newEntry, requestNew, { enabled: !askNew });
  useEscape('leave', () => {
    void navigate({ to: leaveTo });
  });

  return (
    <div ref={rootRef} className="flex max-w-page flex-col gap-3 short:gap-2">
      <LeaveGuard
        dirty={filled}
        onSave={onSave}
        allowNavigation={(next) => next.pathname === path}
      />
      <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-2">
        {/* The name first, with its line of help behind an ⓘ, so the name and the buttons are one
            row (plan 38 G5, G7). */}
        <div className="flex items-center gap-1">
          <Localized id={titleId}>
            <h1 className="sr-only normal:not-sr-only normal:type-title" />
          </Localized>
          <div className="hidden normal:block">
            <InfoHint>
              <Localized id={helpId}>
                <p />
              </Localized>
            </InfoHint>
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {actions}
          <Button shortcut={SHORTCUTS.newEntry} onClick={requestNew}>
            <Plus aria-hidden="true" className="size-icon-md" />
            <Localized id="cashbook-new" />
          </Button>
          <Button
            variant="primary"
            shortcut={SHORTCUTS.save}
            busy={busy}
            disabled={saveBlocked ?? !filled}
            onClick={saveNow}
          >
            <Localized id="layout-save-save" />
          </Button>
        </div>
      </div>
      {tabs}
      {types === undefined ||
      type === undefined ||
      onType === undefined ||
      typeLabelId === undefined ? null : (
        <Field label={<Localized id={typeLabelId} />} hideLabel>
          {() => (
            <div>
              <SegmentedControl<T>
                options={types.map((choice) => ({
                  value: choice.value,
                  label: <Localized id={choice.labelId} />,
                }))}
                value={type}
                groupLabel={l10n.getString(typeLabelId)}
                onChange={onType}
              />
            </div>
          )}
        </Field>
      )}
      <form
        ref={formRef}
        {...navigation}
        onSubmit={(event) => {
          event.preventDefault();
        }}
        className="flex flex-col gap-3 short:gap-2"
      >
        {error === null ? null : <ErrorNotice error={error} fieldsShown={fieldsShown} />}
        {children}
      </form>
      {between}
      {below}
      {flow.cashAfter === null ? null : (
        <CashConfirmDialog
          open={flow.ask}
          onOpenChange={(open) => {
            if (flow.onCashOpenChange(open)) {
              // After a "no" the cursor is on the amount (PRD FR-6.7).
              window.requestAnimationFrame(() => {
                formRef.current
                  ?.querySelector<HTMLInputElement>('input[inputmode="numeric"]')
                  ?.focus();
              });
            }
          }}
          cashAfter={flow.cashAfter}
          onConfirm={flow.confirmCash}
          onCloseFocus={(event) => {
            // Focus goes where the screen puts it, not back to the button.
            event.preventDefault();
          }}
        />
      )}
      <SaveChangesPrompt
        open={askNew}
        onOpenChange={setAskNew}
        onSave={() => {
          void onSave().then((saved) => {
            if (saved) setAskNew(false);
          });
        }}
        onLeave={() => {
          setAskNew(false);
          onNew();
        }}
      />
    </div>
  );
}
