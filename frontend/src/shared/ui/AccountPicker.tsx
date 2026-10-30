// The picker for every account field of the app (PRD FR-7.38 to FR-7.42, FR-10.8, Code Standards
// F40): sellers, buyers, or any account. It is the shared picker with three things added:
//
// - Short codes. A code typed in a seller or buyer field can be just the running number: `1` is
//   0021 in a buyer field (plan 32, R5). It is taken on Enter or on leaving the field, never while typing, because `1`
//   may be the start of `12`. While typing, the list shows the matches and the party card previews
//   the top one. The backend resolves the code (found, several, unknown, reserved). A code has no
//   fixed length (plan 32), so any run of digits is sent.
// - The party card under the field as soon as an account is known, or highlighted (FR-7.38).
// - Adding an account on the spot (plan 41, PRD FR-7.69): the add row that ends the open list, `F4`,
//   Enter on a name that matches nothing, or the button of the unknown-code message open the quick
//   add over the screen with what was typed filled in (FR-7.42, FR-10.8). The new account is put in
//   the field. The quick add has a link to the full Index form for fields it does not ask.
//
// Pickers never convert English letters to Urdu (F41): the text is searched as typed.

import { Localized } from '@fluent/react';
import { useEffect, useMemo, useRef, useState } from 'react';
import {
  accountSource,
  accountsResolveCode,
  type AccountHit,
  type AccountRow,
  type AccountScope,
  type ResolvedCode,
} from '../accounts';
import { SHORTCUTS, useShortcut } from '../keyboard';
import { displayName, useLanguage } from '../i18n';
import { useAddRecord, type AddRecordKind } from './add-record';
import { Balance } from './Balance';
import { Button } from './Button';
import type { FieldControlProps } from './Field';
import { PartyCard } from './PartyCard';
import { PickerBase, type PickerValue } from './PickerBase';
import { QuickAddAccount } from './QuickAddAccount';

interface AccountPickerProps extends FieldControlProps {
  /** Which accounts the field takes: sellers, buyers, or every account. */
  scope: AccountScope;
  value: PickerValue | null;
  /** The account chosen, or null when the field was emptied. */
  onChange: (account: AccountHit | null) => void;
  placeholder?: string | undefined;
  /** Show the party card under the field. Default: yes (FR-7.38). */
  showCard?: boolean | undefined;
  /** The control whose accounts the field takes, for scope `control` (for example ادھار رقم). */
  controlId?: number | undefined;
  /**
   * Which account the party card would show now: the one the list highlights, else the chosen
   * one. A screen that draws the card itself (the sale lines grid, where the picker is a cell)
   * turns `showCard` off and reads this.
   */
  onCardAccount?: ((accountId: number | null) => void) | undefined;
  /** False where adding an account from the field would not make sense, for example a filter. */
  canAdd?: boolean | undefined;
  /** For a picker in a grid cell (see PickerBase). */
  initialText?: string | undefined;
  startOpen?: boolean | undefined;
  onEnterPicked?: (() => void) | undefined;
  /**
   * A person in the dead amount may be offered and picked (PRD FR-10.10): only the receipt field and
   * the ادھار رقم واپسی field say yes. He shows with the tag "ڈوبی".
   */
  includeDead?: boolean | undefined;
}

/** What the screen says about a code that did not give one account. */
interface CodeNote {
  result: ResolvedCode;
  text: string;
}

/** The text an account's row of the picker shows after the name: the father or the firm, the city. */
function hitOfRow(row: AccountRow): AccountHit {
  return {
    id: row.id,
    code: row.code,
    account_type: row.account_type,
    control_id: row.control.id,
    name_ur: row.name_ur,
    name_en: row.name_en,
    ...(row.tag === undefined ? {} : { tag: row.tag }),
    extra_ur: [row.father_or_firm_ur, row.city?.name_ur ?? '']
      .filter((part) => part !== '')
      .join('، '),
    extra_en: [row.father_or_firm_en, row.city?.name_en ?? '']
      .filter((part) => part !== '')
      .join(', '),
    is_active: row.is_active,
    is_dead: false,
    balance: row.balance,
  };
}

/** Only digits can be a code (PRD FR-7.39). Codes grow without a limit (plan 32, R2). */
const CODE_TEXT = /^\d{1,18}$/;

/** The word of each kind of field in the add row's and the empty list's messages, and its form. */
const ADD: Record<AccountScope, { noun: string; kind: AddRecordKind | null }> = {
  seller: { noun: 'seller', kind: 'seller' },
  lot_seller: { noun: 'seller', kind: 'seller' },
  buyer: { noun: 'buyer', kind: 'buyer' },
  any: { noun: 'account', kind: 'any' },
  // The accounts of one control have their own forms (money lent, separate khata).
  control: { noun: 'account', kind: null },
};

const UNKNOWN_ID: Record<AccountScope, string> = {
  seller: 'accountpicker-unknown-seller',
  // The seller field of a lot also offers اپنا مال; an unknown code is still an unknown seller.
  lot_seller: 'accountpicker-unknown-seller',
  buyer: 'accountpicker-unknown-buyer',
  any: 'accountpicker-unknown-any',
  control: 'accountpicker-unknown-any',
};

export function AccountPicker({
  scope,
  value,
  onChange,
  showCard = true,
  controlId,
  onCardAccount,
  onEnterPicked,
  canAdd = true,
  includeDead = false,
  ...field
}: AccountPickerProps) {
  const language = useLanguage();
  const source = useMemo(
    () => accountSource(scope, controlId ?? null, includeDead),
    [scope, controlId, includeDead],
  );
  const [note, setNote] = useState<CodeNote | null>(null);
  const [highlight, setHighlight] = useState<AccountHit | null>(null);
  // The quick add that is open: the text it starts with. Who is waiting for the new account is kept
  // in a ref, so the form closing after a save calls it once.
  const [adding, setAdding] = useState<string | null>(null);
  const waiting = useRef<((added: AccountHit | null) => void) | null>(null);
  const addRecord = useAddRecord();
  const add = ADD[scope];
  // True while a code that Enter sent to the backend is being answered: when it is found, the
  // grid cell goes on to the next cell as it does after any pick by Enter (PRD FR-7.14).
  const enterPending = useRef(false);

  // A code typed and then left (or Enter with no match) is asked of the backend once.
  const resolve = (text: string) => {
    if (!CODE_TEXT.test(text)) {
      enterPending.current = false;
      return;
    }
    void accountsResolveCode(scope, controlId ?? null, text)
      .then((result) => {
        const byEnter = enterPending.current;
        enterPending.current = false;
        // A person in the dead amount comes back as "inactive" with his account; only the fields that
        // take him (the receipt, the ادھار رقم واپسی) pick him by his code (PRD FR-10.10).
        const deadPick =
          includeDead && result.status === 'inactive' && result.account?.is_dead === true;
        if ((result.status === 'found' || deadPick) && result.account !== null) {
          setNote(null);
          onChange(result.account);
          if (byEnter) onEnterPicked?.();
        } else {
          setNote({ result, text });
        }
      })
      .catch(() => {
        enterPending.current = false;
      });
  };

  // Opens the quick add. `done` is told which account was saved, or null when the form closed
  // without one (Esc). The message path (`null`) puts a saved account in the field itself.
  function openQuickAdd(text: string, done: ((added: AccountHit | null) => void) | null) {
    waiting.current =
      done ??
      ((hit) => {
        if (hit === null) return;
        setNote(null);
        onChange(hit);
      });
    setAdding(text);
  }
  function finishQuickAdd(hit: AccountHit | null) {
    const done = waiting.current;
    waiting.current = null;
    setAdding(null);
    done?.(hit);
  }

  // The full Index form, for fields the quick add does not ask (plan 41, R3). The quick add closes
  // and the picker waits for the Index form instead.
  const fullForm =
    addRecord !== null && add.kind !== null
      ? (text: string) => {
          const done = waiting.current;
          waiting.current = null;
          setAdding(null);
          const kind = add.kind;
          if (kind === null) return;
          addRecord({
            kind,
            text,
            form: 'full',
            onClosed: (added) => {
              if (added === null) {
                done?.(null);
                return;
              }
              // The account the form saved, as the list shows it.
              accountsResolveCode(scope, controlId ?? null, added.code).then(
                (result) => {
                  done?.(result.status === 'found' ? result.account : null);
                },
                () => {
                  done?.(null);
                },
              );
            },
          });
        }
      : undefined;

  const unknown = note?.result.status === 'unknown';
  // F4 works from the message too, after the field has been left.
  useShortcut(
    SHORTCUTS.addAccount,
    () => {
      if (note !== null) openQuickAdd(note.text, null);
    },
    { enabled: unknown && adding === null },
  );

  const cardId = highlight?.id ?? value?.id ?? null;
  const cardRef = useRef(onCardAccount);
  useEffect(() => {
    cardRef.current = onCardAccount;
  });
  useEffect(() => {
    cardRef.current?.(cardId);
  }, [cardId]);
  return (
    <div className="flex flex-col gap-2">
      <PickerBase<AccountHit>
        {...field}
        source={source}
        value={value}
        onChange={(hit) => {
          setNote(null);
          onChange(hit);
        }}
        onHighlight={setHighlight}
        onLeave={resolve}
        onEnterNoMatch={(text) => {
          enterPending.current = true;
          resolve(text);
        }}
        onEnterPicked={onEnterPicked}
        resolveOnEnter={(text) => {
          // A code goes whole to the backend; anything else waits for the answer to its text.
          if (!CODE_TEXT.test(text)) return false;
          enterPending.current = true;
          resolve(text);
          return true;
        }}
        onTyping={() => {
          setNote(null);
        }}
        addNoun={canAdd ? add.noun : undefined}
        onAddRecord={(text, done) => {
          openQuickAdd(text, done);
        }}
        renderEnd={(hit) => (
          <span className="flex items-baseline gap-3">
            {scope === 'any' ? (
              <Localized id={`accounts-type-${hit.account_type}`}>
                <span className="type-chip rounded-1 bg-surface-sunken px-2 text-ink-muted" />
              </Localized>
            ) : null}
            {hit.balance === null ? null : <Balance balance={hit.balance} />}
          </span>
        )}
      />
      {note === null ? null : (
        <div role="status" className="flex flex-col items-start gap-2 type-body text-ink">
          {note.result.status === 'several' ? (
            <>
              <Localized id="accountpicker-several">
                <p />
              </Localized>
              <ul className="flex flex-wrap gap-2">
                {note.result.candidates.map((candidate) => (
                  <li key={candidate.id}>
                    <Button
                      onClick={() => {
                        setNote(null);
                        onChange(candidate);
                      }}
                    >
                      <Localized id={`accounts-type-${candidate.account_type}`} />
                      <bdi dir="ltr">{candidate.code}</bdi>
                      {displayName(candidate, language)}
                    </Button>
                  </li>
                ))}
              </ul>
            </>
          ) : note.result.status === 'reserved' ? (
            <Localized id="accountpicker-reserved">
              <p className="text-due" />
            </Localized>
          ) : note.result.status === 'inactive' ? (
            // The account exists but is hidden, so a new entry cannot use it (PRD FR-7.42).
            <Localized
              id="accountpicker-inactive"
              vars={{
                name:
                  note.result.account === null ? '' : displayName(note.result.account, language),
                code: note.result.account?.code ?? note.text,
              }}
            >
              <p className="text-due" />
            </Localized>
          ) : (
            <>
              <Localized id={UNKNOWN_ID[scope]}>
                <p className="text-due" />
              </Localized>
              <Button
                shortcut={SHORTCUTS.addAccount}
                onClick={() => {
                  openQuickAdd(note.text, null);
                }}
              >
                <Localized id="accountpicker-add" />
              </Button>
            </>
          )}
        </div>
      )}
      {showCard ? <PartyCard accountId={cardId} /> : null}
      <QuickAddAccount
        open={adding !== null}
        onOpenChange={(open) => {
          if (!open) finishQuickAdd(null);
        }}
        scope={scope}
        typed={adding ?? ''}
        suggestedCode={note?.result.suggested_code ?? null}
        onSaved={(row) => {
          finishQuickAdd(hitOfRow(row));
        }}
        onFullForm={fullForm}
        onPickExisting={(match) => {
          // A look-alike that is already there is the one he means: take it, save nothing.
          void accountsResolveCode(scope, controlId ?? null, match.code).then((result) => {
            if (result.status === 'found') finishQuickAdd(result.account);
          });
        }}
      />
    </div>
  );
}
