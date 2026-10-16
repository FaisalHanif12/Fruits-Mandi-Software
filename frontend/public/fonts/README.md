# Bundled fonts

Loaded from the app's own files, never from the internet (NFR-URD-9, NFR-URD-27, Code Standards S5).
All four are under the SIL Open Font License 1.1. The licence of each font is next to it.

| File | Font | Use (Tech Stack 4.1) | Source |
|---|---|---|---|
| `Gulzar-Regular.woff2` | Gulzar | Urdu labels, headings, menus, printed bills | google/fonts `ofl/gulzar/Gulzar-Regular.ttf` |
| `NotoNastaliqUrdu-Variable.woff2` | Noto Nastaliq Urdu | Fallback Nastaliq | google/fonts `ofl/notonastaliqurdu/NotoNastaliqUrdu[wght].ttf` |
| `NotoNaskhArabic-Variable.woff2` | Noto Naskh Arabic | Dense grids (user setting), Urdu digits on prints | google/fonts `ofl/notonaskharabic/NotoNaskhArabic[wght].ttf` |
| `Inter-Variable.woff2` | Inter | Numbers, Latin text and the whole English app | google/fonts `ofl/inter/Inter[opsz,wght].ttf` |

Taken from the `main` branch of https://github.com/google/fonts on 10-10-2026 and converted from
TTF to WOFF2 with fontTools (no glyph changes). None of these fonts declares a Reserved Font Name.
