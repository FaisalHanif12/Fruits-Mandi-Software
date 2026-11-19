// The mark of the app (plan 75): a gold mandi crate with an orange, a green apple and a leaf, on the
// deep navy tile of the logo (frontend/brand/app-icon.svg, drawn here as inline SVG so it needs no file and
// no network). It stands beside the app name on the sign-in panel (large) and at the top of the main menu
// (small), so the two places look like one product. It has no words, so it is the same in both
// languages and in both directions. It only decorates, so a screen reader skips it. The colours are the
// `--color-brand-*` tokens.

import { useId, type CSSProperties } from 'react';

export type BrandMarkSize = 'sm' | 'lg';

interface BrandMarkProps {
  size?: BrandMarkSize;
}

const SIZE_CLASSES: Record<BrandMarkSize, string> = {
  sm: 'size-target',
  lg: 'size-badge',
};

/** The colour of one gradient stop, from a token. */
const stop = (token: string): CSSProperties => ({ stopColor: `var(--color-brand-${token})` });

/** The tile of the logo is 824 units square, 100 units in from the edge of its 1024 canvas. */
const TILE_VIEW_BOX = '100 100 824 824';

export function BrandMark({ size = 'lg' }: BrandMarkProps) {
  const key = useId();
  const ground = `${key}-ground`;
  const orange = `${key}-orange`;
  const apple = `${key}-apple`;
  const crate = `${key}-crate`;
  return (
    <svg
      aria-hidden="true"
      focusable="false"
      viewBox={TILE_VIEW_BOX}
      className={`shrink-0 ${SIZE_CLASSES[size]}`}
    >
      <defs>
        <linearGradient id={ground} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" style={stop('ground-top')} />
          <stop offset="1" style={stop('ground-bottom')} />
        </linearGradient>
        <radialGradient id={orange} cx="0.35" cy="0.3" r="0.8">
          <stop offset="0" style={stop('orange-light')} />
          <stop offset="1" style={stop('orange')} />
        </radialGradient>
        <radialGradient id={apple} cx="0.35" cy="0.3" r="0.8">
          <stop offset="0" style={stop('apple-light')} />
          <stop offset="1" style={stop('apple')} />
        </radialGradient>
        <clipPath id={crate}>
          <rect x="236" y="566" width="552" height="274" rx="34" />
        </clipPath>
      </defs>
      <rect x="100" y="100" width="824" height="824" rx="185" fill={`url(#${ground})`} />
      <rect
        x="104"
        y="104"
        width="816"
        height="816"
        rx="181"
        fill="none"
        strokeWidth="8"
        strokeOpacity="0.08"
        className="stroke-on-primary"
      />
      {/* The fruit behind the crate. */}
      <circle cx="352" cy="526" r="112" fill={`url(#${apple})`} />
      <circle cx="540" cy="468" r="176" fill={`url(#${orange})`} />
      <path
        d="M548 306 C548 244 588 202 668 194 C672 272 626 324 548 306 Z"
        className="fill-brand-leaf"
      />
      <path
        d="M550 304 C590 268 628 234 656 206"
        fill="none"
        strokeWidth="12"
        strokeLinecap="round"
        className="stroke-brand-leaf-vein"
      />
      {/* The crate: three equal slats, two gaps, a post at each end. */}
      <g clipPath={`url(#${crate})`}>
        <rect x="236" y="566" width="552" height="274" className="fill-brand-gold" />
        <rect x="236" y="566" width="552" height="22" className="fill-brand-gold-light" />
        <rect x="236" y="566" width="60" height="274" className="fill-brand-gold-dark" />
        <rect x="728" y="566" width="60" height="274" className="fill-brand-gold-dark" />
        <rect x="236" y="651" width="552" height="26" className="fill-brand-ground-bottom" />
        <rect x="236" y="742" width="552" height="26" className="fill-brand-ground-bottom" />
      </g>
    </svg>
  );
}
