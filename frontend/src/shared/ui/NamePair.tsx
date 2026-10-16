// Every name has an Urdu and an English form (PRD FR-7.24, Code Standards F41). The user types
// one; the other fills by itself, and he only checks it.
//
// - Typing English letters in the Urdu field turns them into Urdu (RomanUrduInput), and the
//   English name is exactly what was typed with capital first letters: "tariq mehmood" gives
//   "Tariq Mehmood". Urdu typed with a keyboard gets an English suggestion from the backend.
// - Typing the English name gives an Urdu suggestion.
// - A field the user has typed in is never overwritten again. A pair that arrives already filled
//   (an existing record) is never overwritten at all.
// - In the English app the English name comes first and is the one to type; in the Urdu app the
//   Urdu name comes first. The fields themselves work the same.
//
// A word the app made only from its spelling rules (not from the word list or a learned pair) is
// marked in amber, "check the spelling", under its name (PRD FR-7.66). The marks live in the value
// of the pair (`spelling`), so they sit in the screen's state with the names. Before the save, the
// screen asks the user to confirm or fix them (useSpellingFlow).
//
// Wrong-script problems show at the field at once (a hint, not trusted: the backend checks every
// name again on save, ERR_NAME_UR_SCRIPT and ERR_NAME_EN_SCRIPT). The pair never saves anything.

import { Localized } from '@fluent/react';
import { useEffect, useRef, useState, type ReactNode } from 'react';
import type { Kind } from '../../bindings/Kind';
import { useLanguage } from '../i18n';
import { SHORTCUTS } from '../keyboard';
import { scriptHintId } from '../text/scriptHint';
import { translitSuggest } from '../urdu/api';
import {
  NO_MARKS,
  isInitial,
  rulesOnlyWords,
  wordsStillIn,
  withMark,
  type NameWithMarks,
} from '../urdu/spelling';
import { capitalizeWords, wordsOf, withoutTrailingLatinRun } from '../urdu/text';
import type { TypingMode } from '../urdu/typing-mode';
import type { Suggestions } from '../../bindings/Suggestions';
import { Field } from './Field';
import { RomanUrduInput, type ConvertedWord, type ListExtra } from './RomanUrduInput';
import { SpellingMark } from './SpellingMark';
import { SuggestInput } from './SuggestInput';
import { TextInput } from './TextInput';

/** The two names, and the words of them that the user has to check (PRD FR-7.66). */
export type NamePairValue = NameWithMarks;

/** A name worked out by the backend, and the words in it made by the spelling rules alone. */
interface Worked {
  text: string;
  rulesWords: string[];
}

/**
 * The best spelling of the whole name, and the words in it made by the spelling rules alone. Null
 * when there is no spelling at all, because then the name cannot be worked out right now.
 */
function workedFrom(answer: Suggestions): Worked | null {
  const best = answer.items[0];
  if (best === undefined || best.text === '') return null;
  return { text: best.text, rulesWords: rulesOnlyWords(best) };
}

/** `value` with the marks of one name replaced by `rulesWords`, leaving out confirmed words. */
function withMarksFor(
  value: NamePairValue,
  script: 'urdu' | 'english',
  rulesWords: readonly string[],
): NamePairValue {
  const marks = value.spelling ?? NO_MARKS;
  return {
    ...value,
    spelling: { ...marks, [script]: rulesWords.filter((word) => !marks.confirmed.includes(word)) },
  };
}

interface NamePairProps {
  value: NamePairValue;
  onChange: (next: NamePairValue) => void;
  urduLabel: ReactNode;
  englishLabel: ReactNode;
  /** The backend's message for the Urdu name, from fieldError(). Shown instead of the hint. */
  urduError?: ReactNode;
  englishError?: ReactNode;
  /** What a learned word is about. Default: name. */
  kind?: Kind | undefined;
  /**
   * Mark words made by the spelling rules alone (PRD FR-7.66). Default: yes. Free text that is
   * not a name, such as a standard narration, turns it off.
   */
  checkSpelling?: boolean | undefined;
  /**
   * The pair opens with one name already typed (for example the text typed in a picker before
   * `F4`): the other name is filled from it once, at open, like after typing.
   */
  fillOnOpen?: boolean | undefined;
  /**
   * Records already known that the name may be (the one-time people, PRD FR-7.29). They are rows
   * at the head of the list of the name typed first: the Urdu field in the Urdu app, the English
   * field in the English app. Taking one fills both names and whatever else the screen puts in
   * `onPick`, and the pair then treats both names as typed, so nothing overwrites them.
   */
  knownRecords?: readonly ListExtra[] | undefined;
}

const LATIN_WORD = /^[A-Za-z]+$/;

export function NamePair({
  value,
  onChange,
  urduLabel,
  englishLabel,
  urduError,
  englishError,
  kind,
  checkSpelling = true,
  fillOnOpen = false,
  knownRecords,
}: NamePairProps) {
  const language = useLanguage();
  // In the English app the English name is the one typed, and the Urdu name fills from it.
  const englishFirst = language !== 'ur';
  const [mode, setMode] = useState<TypingMode>('roman');
  const [pending, setPending] = useState('');
  // The Urdu word -> the English letters it was typed with, for this pair.
  const known = useRef(new Map<string, string>());
  // A field the user has typed in is never filled by the app again.
  const typedBy = useRef({ urdu: value.urdu !== '', english: value.english !== '' });
  const latest = useRef(value);
  const request = useRef(0);
  const onChangeRef = useRef(onChange);
  useEffect(() => {
    latest.current = value;
    onChangeRef.current = onChange;
  });

  // A pair that opens with one name typed fills the other from it once (fillOnOpen).
  useEffect(() => {
    if (!fillOnOpen) return undefined;
    const { urdu, english } = latest.current;
    const direction =
      english !== '' && urdu === ''
        ? 'to_urdu'
        : urdu !== '' && english === ''
          ? 'to_english'
          : null;
    if (direction === null) return undefined;
    let current = true;
    translitSuggest({ text: direction === 'to_urdu' ? english : urdu, direction })
      .then((answer) => {
        const worked = workedFrom(answer);
        if (!current || worked === null) return;
        const filled =
          direction === 'to_urdu'
            ? withMarksFor(
                { ...latest.current, urdu: worked.text },
                'urdu',
                checkSpelling ? worked.rulesWords : [],
              )
            : withMarksFor(
                { ...latest.current, english: worked.text },
                'english',
                checkSpelling ? worked.rulesWords : [],
              );
        onChangeRef.current(filled);
      })
      .catch(() => undefined);
    return () => {
      current = false;
    };
    // Once, when the pair opens.
  }, [fillOnOpen, checkSpelling]);

  /** The English name for an Urdu text, or null when it cannot be worked out right now. */
  const englishFor = async (urdu: string): Promise<Worked | null> => {
    const words = wordsOf(urdu);
    if (words.length === 0) return { text: '', rulesWords: [] };
    const local = words.map(
      (word) => known.current.get(word) ?? (LATIN_WORD.test(word) ? word : null),
    );
    // Words typed in English letters are exactly what was typed: nothing to check.
    if (local.every((word): word is string => word !== null))
      return { text: capitalizeWords(local.join(' ')), rulesWords: [] };
    try {
      return workedFrom(await translitSuggest({ text: urdu, direction: 'to_english' }));
    } catch {
      return null;
    }
  };

  const urduFor = async (english: string): Promise<Worked | null> => {
    if (english.trim() === '') return { text: '', rulesWords: [] };
    try {
      return workedFrom(await translitSuggest({ text: english, direction: 'to_urdu' }));
    } catch {
      return null;
    }
  };

  const changeUrdu = (urdu: string) => {
    typedBy.current.urdu = true;
    // Marks of words that are no longer in the text go away, so a word typed again by hand later
    // is not marked by an old conversion.
    const marks = latest.current.spelling ?? NO_MARKS;
    onChange({
      ...latest.current,
      urdu,
      spelling: { ...marks, urdu: wordsStillIn(urdu, marks.urdu) },
    });
    if (typedBy.current.english) return;
    request.current += 1;
    const id = request.current;
    void englishFor(urdu).then((worked) => {
      if (worked === null || id !== request.current || typedBy.current.english) return;
      onChange(
        withMarksFor(
          { ...latest.current, english: worked.text },
          'english',
          checkSpelling ? worked.rulesWords : [],
        ),
      );
    });
  };

  const changeEnglish = (english: string) => {
    typedBy.current.english = true;
    // An English name typed by hand is his own spelling: nothing for the app to mark.
    onChange(withMarksFor({ ...latest.current, english }, 'english', []));
    if (typedBy.current.urdu) return;
    request.current += 1;
    const id = request.current;
    void urduFor(english).then((worked) => {
      if (worked === null || id !== request.current || typedBy.current.urdu) return;
      onChange(
        withMarksFor(
          { ...latest.current, urdu: worked.text },
          'urdu',
          checkSpelling ? worked.rulesWords : [],
        ),
      );
    });
  };

  const converted = ({ roman, urdu, source }: ConvertedWord) => {
    known.current.set(urdu, roman);
    // A spelling made by the rules alone is marked until he confirms or changes it (FR-7.66).
    // The change that comes with this word reads the marks from here, so it is not lost.
    if (checkSpelling && source === 'rules' && !isInitial(urdu)) {
      latest.current = {
        ...latest.current,
        spelling: withMark(latest.current.spelling, 'urdu', urdu),
      };
    }
  };

  // A known record taken from the list: both names are his, and neither is overwritten again.
  const knownRows: ListExtra[] | undefined = knownRecords?.map((row) => ({
    ...row,
    onPick: () => {
      typedBy.current = { urdu: true, english: true };
      request.current += 1;
      row.onPick();
    },
  }));

  // The English word being typed is not Urdu yet, so it is not a mistake.
  const urduChecked = pending === '' ? value.urdu : withoutTrailingLatinRun(value.urdu);
  const marks = value.spelling ?? NO_MARKS;
  const urduHintId = scriptHintId(urduChecked, 'ur');
  const englishHintId = scriptHintId(value.english, 'en');

  const urduField = (
    <Field
      label={urduLabel}
      hint={
        <Localized
          id={englishFirst ? 'translit-pair-urdu-hint-second' : 'translit-pair-urdu-hint'}
          vars={{ key: SHORTCUTS.typingMode.label }}
        />
      }
      error={urduError ?? (urduHintId === null ? undefined : <Localized id={urduHintId} />)}
    >
      {(control) => (
        <RomanUrduInput
          {...control}
          value={value.urdu}
          onChange={changeUrdu}
          mode={mode}
          onModeChange={setMode}
          kind={kind}
          onWordConverted={converted}
          onPendingChange={setPending}
          marked={marks.urdu}
          extras={englishFirst ? undefined : knownRows}
        />
      )}
    </Field>
  );
  const englishField = (
    <Field
      label={englishLabel}
      hint={
        <Localized
          id={englishFirst ? 'translit-pair-english-hint-first' : 'translit-pair-english-hint'}
        />
      }
      error={
        englishError ?? (englishHintId === null ? undefined : <Localized id={englishHintId} />)
      }
    >
      {(control) => (
        <>
          {/* The same input stays while the list comes and goes, so focus is never lost. */}
          {englishFirst && knownRows !== undefined ? (
            <SuggestInput
              {...control}
              value={value.english}
              onChange={changeEnglish}
              suggestions={knownRows.map((row) => ({
                value: row.key,
                content: row.content,
                onPick: row.onPick,
              }))}
            />
          ) : (
            <TextInput {...control} value={value.english} onChange={changeEnglish} lang="en" />
          )}
          <SpellingMark words={wordsStillIn(value.english, marks.english)} lang="en" />
        </>
      )}
    </Field>
  );

  // The name to type comes first: Urdu in the Urdu app, English in the English app.
  return !englishFirst ? (
    <>
      {urduField}
      {englishField}
    </>
  ) : (
    <>
      {englishField}
      {urduField}
    </>
  );
}
