# shared/urdu

Typing Urdu (PRD 7.4, Tech Stack 4.4): the phonetic keyboard, the typing-mode badge
(رومن، اردو، EN) and the typing commands.

- `RomanUrduInput` and `NamePair` live in `shared/ui` and use these.
- `api.ts`: `translit_suggest`, `translit_learn` and the learned words list.
- `useSuggestions.ts`: the spellings of the English word being typed, with their sources.
- `spelling.ts`: the spelling check of names (PRD FR-7.66). A word the rules alone made is marked
  "check the spelling"; the marks live next to the names in the screen's state. A save asks the
  user to confirm or fix them (`shared/hooks/useSpellingFlow`) and sends `confirmed_rules_words`.
  The backend decides again and learns the confirmed pairs itself.
- `phonetic_keyboard.ts` and `KeyMap.tsx`: the اردو کی بورڈ mode.
