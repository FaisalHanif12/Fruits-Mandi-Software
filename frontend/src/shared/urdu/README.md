# shared/urdu

Typing Urdu (PRD 7.4, Tech Stack 4.4): the phonetic keyboard, the typing-mode badge
(رومن، اردو، EN) and the typing commands.

- `RomanUrduInput` and `NamePair` live in `shared/ui` and use these.
- `api.ts`: `translit_suggest`, `translit_learn` and the learned words list.
- `useSuggestions.ts`: the spellings of the English word being typed, with their sources. The field says its kind
  (`word` for free text, a name kind for names) and the text before the word, so the backend ranks everyday words
  first in free text and names first in name fields, and reads a word in its phrase (plan 99). `FreeTextInput` is
  `word` unless the screen says `name` or `place`; each `NamePair` says its kind (the master lists in
  `features/masters/configs.ts`).
- `spelling.ts`: the spelling check of names (PRD FR-7.66). A word the rules alone made is marked
  "check the spelling"; the marks live next to the names in the screen's state. A save asks the
  user to confirm or fix them (`shared/hooks/useSpellingFlow`) and sends `confirmed_rules_words`.
  The backend decides again and learns the confirmed pairs itself.
- `phonetic_keyboard.ts` and `KeyMap.tsx`: the اردو کی بورڈ mode.
