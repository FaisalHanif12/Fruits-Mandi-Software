# Urdu and English style guide

## Glossary terms with gender

Each Urdu business word is a Fluent term in `ur/glossary.ftl` with a `.gender` attribute, so messages can agree with it.

```ftl
-bikri = بکری
    .gender = feminine
-khata = کھاتہ
    .gender = masculine
```

```ftl
# Using the attribute in a selector
saved = { -bikri.gender ->
    [feminine] { -bikri } محفوظ ہو گئی
   *[masculine] { -bikri } محفوظ ہو گیا
}
```

In `en/glossary.ftl` the same terms hold the English words (`-bikri = Sale`, `-laga = Laga`).

Only words in the PRD glossary (Section 3) become terms. Other common words, such as واؤچر, are written directly in messages with their fixed spelling.

## Gender of common words

The verb agrees with the noun: feminine `ہو گئی`, masculine `ہو گیا`. If a word is not in this list, ask the Urdu reviewer and add it here.

| Feminine | Masculine |
|---|---|
| بکری، وصولی، ادائیگی، پیشگی، اگراہی | واؤچر، کھاتہ، اندراج، روزنامچہ، ٹرائل بیلنس |
| رقم، قیمت، مقدار، تعداد | کمیشن، لاگا، کرایہ، ریٹ، نقد، فون |
| تاریخ، رپورٹ، فہرست، تفصیل | بیک اپ، سٹاک، خرچہ، حصہ، کوڈ، نام |
| مزدوری، پیکنگ، قسم | لاٹ، ٹرک، اڈا، مارکہ، صارف، پاس ورڈ |

## Tone

| Do | Don't | Why |
|---|---|---|
| محفوظ کریں | محفوظ کرو | Polite form for commands (NFR-URD-5) |
| محفوظ کر کے نئی بکری | محفوظ کریں اور نیا | What a munshi would say, not a translation |
| بکری 1 محفوظ ہو گئی | ریکارڈ کامیابی سے محفوظ ہو گیا | Name the thing; no technical words |
| رقم پورے روپوں میں لکھیں | غلط ان پٹ | Say what to do |
| گاہک کا کوڈ یا نام لکھیں | کوڈ درج کریں | Hints say exactly what to type |
| اس کوڈ کا کوئی گاہک نہیں | ریکارڈ نہیں ملا | Plain words |

Keep messages short. One idea per sentence. No exclamation marks in success messages.

## Punctuation and digits

- Full stop `۔`, comma `،`, question mark `؟`.
- English digits (0-9) on screen in both languages (PRD FR-7.32).
- Numbers, codes and money are variables, so they are isolated and never flip: `مقدار { $qty } نگ`.

## English

- Plain, professional English, British spelling as used in Pakistan: Labour, Colour.
- Business words from the glossary's English column only: Sale, Sale Book, Receipt, Payment, Advance, Commission, Laga, Munshiana, Pukhta Khata, Adda, Nag, Gross Sale, Net Sale.
- Sentence case for messages ("Sale 1 saved"), the glossary's capitals for business words.
- The same meaning as the Urdu. Never shorter or vaguer.

## Worked examples

| ID | Urdu | English |
|---|---|---|
| `bikri-action-save-new` | محفوظ کر کے نئی بکری | Save and new sale |
| `common-leave-unsaved` | محفوظ کریں؟ | Save changes? |
| `err-decimals-not-allowed` | رقم پورے روپوں میں لکھیں۔ | Enter the amount in whole rupees. |
| `err-date-closed` | یہ دن بند ہو چکا ہے۔ کھلی تاریخ لکھیں یا مالک سے دن کھلوائیں۔ | This day is closed. Enter an open date, or ask the Owner to reopen the day. |
| `err-pukhta-name-required` | { -pukhta-khata } میں نام لکھنا ضروری ہے۔ | A name is required in { -pukhta-khata }. |
| `dashboard-empty-today` | آج کوئی بکری نہیں۔ نئی بکری کے لیے Alt+B دبائیں۔ | No sales today. Press Alt+B for a new sale. |
