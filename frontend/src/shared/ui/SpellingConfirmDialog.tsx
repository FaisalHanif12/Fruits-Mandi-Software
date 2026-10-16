// Asks the user to check the spelling of a name before it is saved (PRD FR-7.66, E49). It shows
// the name with the words made by the spelling rules marked, and two actions: confirm the spelling
// (the word is learned and the save goes on) or go back and fix it (Esc, or the back button).
// The save never goes on without one of the two.

import { Localized } from '@fluent/react';
import { Fragment } from 'react';
import { wordsOf } from '../urdu/text';
import { ConfirmDialog } from './ConfirmDialog';

/** One name shown in the dialog with the words to check. */
export interface SpellingName {
  text: string;
  /** The words of `text` to check. */
  words: readonly string[];
}

interface SpellingConfirmDialogProps {
  open: boolean;
  urdu: SpellingName | null;
  english: SpellingName | null;
  /** Confirm: the spelling is right. */
  onConfirm: () => void;
  /** Back: close the dialog and fix the name. Also Esc. */
  onBack: () => void;
  /** Where focus goes when the dialog has closed (see ConfirmDialog). */
  onCloseFocus?: ((event: Event) => void) | undefined;
}

function MarkedName({ name, lang }: { name: SpellingName; lang: 'ur' | 'en' }) {
  return (
    <bdi lang={lang} dir={lang === 'ur' ? 'rtl' : 'ltr'} className="type-field text-ink">
      {wordsOf(name.text).map((word, index) => (
        <Fragment key={`${word}-${String(index)}`}>
          {index === 0 ? '' : ' '}
          {name.words.includes(word) ? (
            <mark className="rounded-1 border border-warn-border bg-warn-soft px-1 text-warn">
              {word}
            </mark>
          ) : (
            word
          )}
        </Fragment>
      ))}
    </bdi>
  );
}

export function SpellingConfirmDialog({
  open,
  urdu,
  english,
  onConfirm,
  onBack,
  onCloseFocus,
}: SpellingConfirmDialogProps) {
  return (
    <ConfirmDialog
      open={open}
      onOpenChange={(next) => {
        if (!next) onBack();
      }}
      title={<Localized id="translit-spelling-title" />}
      description={
        <>
          <Localized id="translit-spelling-body">
            <span className="block" />
          </Localized>
          {urdu === null ? null : (
            <span className="mt-3 flex flex-col gap-1">
              <Localized id="translit-spelling-urdu-name">
                <span className="type-label" />
              </Localized>
              <MarkedName name={urdu} lang="ur" />
            </span>
          )}
          {english === null ? null : (
            <span className="mt-3 flex flex-col gap-1">
              <Localized id="translit-spelling-english-name">
                <span className="type-label" />
              </Localized>
              <MarkedName name={english} lang="en" />
            </span>
          )}
        </>
      }
      confirmLabel={<Localized id="translit-spelling-confirm" />}
      onConfirm={onConfirm}
      onCloseFocus={onCloseFocus}
    />
  );
}
