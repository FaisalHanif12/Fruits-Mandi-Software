// The fruit picture at the top left of every print header (PRD FR-19.13, plan 76 point 6): the mandi
// crate with an orange, an apple and a leaf, our own drawing from the app mark (`shared/ui/BrandMark`),
// drawn here as line art for paper. Inline SVG, so it needs no file and no network (CSP) and stays
// sharp at any size. Black outlines carry every shape and light tints of the brand colours fill them,
// so on a black and white printer the shapes are still clear and the tints turn into light grey. Each
// shape sits on a white underlay, so the fruit behind the crate and behind the orange is hidden as in
// the mark. It has no words, so it is the same in both print languages, and it only decorates, so a
// screen reader skips it. Its place is the physical left edge of the header (`.print-mark`).

import type { CSSProperties } from 'react';

/** The size of the drawing in its own units: the crate and the fruit of the app mark, cropped. */
const VIEW_BOX = '212 170 604 696';
const OUTLINE = 17;

/** A tint of a brand colour: the colour at a low strength, so black outlines stay the strongest ink. */
const tint = (token: string): CSSProperties => ({
  fill: `var(--color-brand-${token})`,
  fillOpacity: 0.4,
});

const UNDERLAY: CSSProperties = { fill: 'var(--color-surface)' };

export function PrintFruitMark() {
  return (
    <svg
      aria-hidden="true"
      focusable="false"
      viewBox={VIEW_BOX}
      className="print-mark"
      fill="none"
      stroke="currentColor"
      strokeWidth={OUTLINE}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {/* The apple, behind the orange. */}
      <circle cx="352" cy="526" r="112" style={UNDERLAY} stroke="none" />
      <circle cx="352" cy="526" r="112" style={tint('apple')} />
      <path d="M352 414 C352 392 360 378 374 368" />
      {/* The orange, with its leaf. */}
      <circle cx="540" cy="468" r="176" style={UNDERLAY} stroke="none" />
      <circle cx="540" cy="468" r="176" style={tint('orange')} />
      <path d="M548 306 C548 244 588 202 668 194 C672 272 626 324 548 306 Z" style={tint('leaf')} />
      <path d="M550 304 C590 268 628 234 656 206" />
      {/* The crate: three slats, two gaps, a post at each end. It hides the foot of the fruit. */}
      <rect x="236" y="566" width="552" height="274" rx="34" style={UNDERLAY} stroke="none" />
      <rect x="236" y="566" width="552" height="274" rx="34" style={tint('gold')} />
      <path d="M296 566 V840 M728 566 V840 M236 651 H788 M236 742 H788" />
    </svg>
  );
}
