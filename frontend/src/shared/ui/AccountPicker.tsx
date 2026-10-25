// The picker for every account field of the app (PRD FR-7.38 to FR-7.42, FR-10.8, Code Standards
// F40): sellers, buyers, or any account. It is the shared picker with three things added:
//
// - Short codes. A code typed in a seller or buyer field can be just the serial: `1` is 03040001
//   in a buyer field. It is taken on Enter or on leaving the field, never while typing, because `1`
//   may be the start of `12`. While typing, the list shows the matches and the party card previews
//   the top one. The backend resolves the code (found, several, unknown, reserved).
// - The party card under the field as soon as an account is known, or highlighted (FR-7.38).
// - Adding an account on the spot: `F4`, or the button of the unknown-code message, opens the
//   quick add over the screen with the typed code filled in (FR-7.42, FR-10.8). The new account
//   comes back into the field.
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
  /** For a picker in a grid cell (see PickerBase). */
  initialText?: string | undefined;
  startOpen?: boolean | undefined;
  onEnterPicked?: (() => void) | undefined;
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
    balance: row.balance,
  };
}

/** Only digits, up to the length of a full code, can be a code (PRD FR-7.39). */
const CODE_TEXT = /^\d{1,9}$/;

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
  ...field
}: AccountPickerProps) {
  const language = useLanguage();
  const source = useMemo(() => accountSource(scope, controlId ?? null), [scope, controlId]);
  const [note, setNote] = useState<CodeNote | null>(null);
  const [highlight, setHighlight] = useState<AccountHit | null>(null);
  const [adding, setAdding] = useState<string | null>(null);
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
        if (result.status === 'found' && result.account !== null) {
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

  const unknown = note?.result.status === 'unknown';
  // F4 works from the message too, after the field has been left.
  useShortcut(
    SHORTCUTS.addAccount,
    () => {
      if (note !== null) setAdding(note.text);
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
        onAddNew={setAdding}
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
                  setAdding(note.text);
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
          if (!open) setAdding(null);
        }}
        scope={scope}
        typed={adding ?? ''}
        suggestedCode={note?.result.suggested_code ?? null}
        onSaved={(row) => {
          setNote(null);
          onChange(hitOfRow(row));
        }}
      />
    </div>
  );
}
