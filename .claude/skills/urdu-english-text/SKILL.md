---
name: urdu-english-text
description: Writes or changes any user-facing text in the Mandi Software (labels, buttons, hints, messages, errors, confirmations, report headings, print text) as professional mandi Urdu and plain English in Project Fluent files, using the locked glossary. Use whenever a screen, error code, report or print needs words, and when reviewing text.
---

# Write Urdu and English text

The munshi must read every screen as if an experienced Urdu-speaking mandi accountant wrote it, not a translator (NFR Section 7). The English must read like a careful Pakistani accountant wrote it. Both live in Fluent files. **No word is ever typed in code** (Code Standards F31).

Grammar notes, the gender list and worked examples are in [style-guide.md](style-guide.md). Read it before writing Urdu.

## Where text lives

```
frontend/src/i18n/
  ur/  glossary.ftl  common.ftl  errors.ftl  bikri.ftl  vouchers.ftl ...   (one file per module)
  en/  the same files, the same message IDs
```

- `glossary.ftl` holds the locked business words as Fluent **terms** (PRD Section 3). Other files use the terms, so each word is spelled once.
- `errors.ftl` holds one message per backend error code.
- The backend reads the same files at build time for Excel headings and the backup pack. It never writes sentences itself.

## Message IDs

| Kind | ID pattern | Example |
|---|---|---|
| Label, button, hint | `<module>-<area>-<thing>` | `bikri-header-truck-no`, `bikri-action-save-new` |
| Error | `err-` + the code in kebab case | `ERR_QTY_EXCEEDS_LOT` becomes `err-qty-exceeds-lot` |
| Glossary term | `-` + the code name of the term | `-bikri`, `-laga`, `-pukhta-khata` |
| Shared word | `common-<thing>` | `common-save`, `common-cancel` |

The same ID must exist in `ur/` and `en/`. `pnpm i18n:check` fails otherwise (NFR-URD-22).

## Steps

1. **Use the glossary first.** If the word is a business word (بکری, لاگا, صافی بکری, وصولی), use its term: `{ -bikri }`. Never retype it. If a business word is missing from the glossary, stop and ask `tech-lead`. Only the PRD glossary can add one.
2. **Write the Urdu message** following the style guide:
   - polite commands (`محفوظ کریں`, never `محفوظ کرو`);
   - correct gender and number agreement, using the term's `.gender`;
   - short, natural mandi Urdu, never word-by-word translation;
   - Urdu punctuation (`۔` `،` `؟`) and English digits.
3. **Write the English message** for the same ID:
   - plain, professional English with British spelling as used in Pakistan (Labour, Cheque);
   - the glossary's English words (Sale, Receipt, Laga, Munshiana, Pukhta Khata, Nag);
   - sentence case for messages, and the glossary's capitals for business words.
4. **Values go in as variables, already formatted.** Pass money and numbers as text already made by `formatPaisa` or the number formatter, so both languages show the same digits and grouping (PRD FR-7.32). Pass a raw number only for a plural choice.
5. **Mark the review state.** Put a comment above each new or changed message: `# review: pending`. The Urdu reviewer and the munshi replace it with `# review: ur=<name> <date>`, and the English reviewer adds `en=<name> <date>` (NFR-URD-4, NFR-URD-21). A release check fails on any `pending`.
6. **Run** `pnpm i18n:check`.

## Patterns

**Error message: what went wrong, then what to do** (NFR-USE-13):

```ftl
# ur/errors.ftl
# review: pending
err-qty-exceeds-lot = مقدار { -baqi-maal } سے زیادہ ہے۔ باقی { $available } { -nag } ہیں۔

# en/errors.ftl
# review: pending
err-qty-exceeds-lot = The quantity is more than the { -baqi-maal }. Only { $available } { -nag } left.
```

**Agreement with the term's gender:**

```ftl
# ur/common.ftl
common-saved-toast = { $kind ->
    [bikri] { -bikri } { $number } محفوظ ہو گئی
    [receipt] { -wasooli } { $number } محفوظ ہو گئی
   *[voucher] واؤچر { $number } محفوظ ہو گیا
}
```

**Confirmations say exactly what will happen** (PRD FR-7.8):

```ftl
bikri-confirm-cancel = { -bikri } { $number } منسوخ ہو جائے گی۔ اس کے تمام اندراج الٹ دیے جائیں گے۔ وجہ لکھیں۔
```

## Rules

- No technical words on screen: no "record", "null", "transaction" or "error code" (NFR-USE-14).
- One fixed spelling per word across the app. For example, always `واؤچر`, never `ووچر` (NFR-URD-7).
- Machine translation may draft long help text only. A person rewrites it (NFR-URD-3).
- Placeables are isolated (Fluent's default bidi isolation is kept on), so numbers and names never flip inside Urdu.
- Amounts in words are not written here. They come from the backend modules `urdu_words` and `english_words`.
- Month names and weekdays come from Fluent messages in `common.ftl`. Digits, including the optional Urdu digits on Urdu prints, come from the backend number formatter.

## Done when

- [ ] Every new word is in Fluent, in `ur` and `en`, with the same IDs.
- [ ] Business words come from glossary terms.
- [ ] Urdu agrees in gender and number, uses polite forms and fixed spellings.
- [ ] Errors say what went wrong and what to do.
- [ ] Each new or changed message is marked `# review: pending` or signed by a reviewer.
- [ ] `pnpm i18n:check` passes.
