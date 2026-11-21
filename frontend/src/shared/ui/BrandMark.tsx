// The mark of the app (plan 75, simplified by Umar on 21-11-2026): a gold mandi crate on the deep navy tile of
// the logo, with no fruit (the fruit picture belongs to the print headers only, `shared/print/PrintFruitMark`).
// It is drawn here as inline SVG so it needs no file and no network. It stands beside the app name on the
// sign-in panel (large) and at the top of the main menu (small), so the two places look like one product. It has no words, so it is the same in both
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

/**
 * The crate stood low on the old tile, under the fruit. Alone, it is moved so its middle (x 512, y 703) is the
 * tile's middle (512, 512) and drawn a quarter larger, so it fills the tile (690 of 824 units wide).
 */
const CRATE_LIFT = 'translate(512 512) scale(1.25) translate(-512 -703)';

/** The tile of the logo is 824 units square, 100 units in from the edge of its 1024 canvas. */
const TILE_VIEW_BOX = '100 100 824 824';

export function BrandMark({ size = 'lg' }: BrandMarkProps) {
  const key = useId();
  const ground = `${key}-ground`;
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
      {/* The crate: three equal slats, two gaps, a post at each end. Alone on the tile, it is lifted to the middle. */}
      <g transform={CRATE_LIFT}>
        <g clipPath={`url(#${crate})`}>
          <rect x="236" y="566" width="552" height="274" className="fill-brand-gold" />
          <rect x="236" y="566" width="552" height="22" className="fill-brand-gold-light" />
          <rect x="236" y="566" width="60" height="274" className="fill-brand-gold-dark" />
          <rect x="728" y="566" width="60" height="274" className="fill-brand-gold-dark" />
          <rect x="236" y="651" width="552" height="26" className="fill-brand-ground-bottom" />
          <rect x="236" y="742" width="552" height="26" className="fill-brand-ground-bottom" />
        </g>
      </g>
    </svg>
  );
}
