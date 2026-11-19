// A quiet picture for the brand panel of the sign-in screen (plan 75 T2): a soft glow and three thin
// rings that rise from the bottom corner at the end side. It carries no meaning, so a screen reader
// skips it. The colours are tokens (`text-primary-chip` sets the glow, `stroke-nav-active` the
// rings). In the right-to-left screen the picture is mirrored, so the rings stay in the corner at the
// end side, as the panel itself stands at the start side.

import { useId } from 'react';

/** The drawing is 400 units square and the corner is its far corner. Only the proportions matter. */
const SIZE = 400;
const VIEW_BOX = '0 0 400 400';
const RING_RADII = [140, 220, 300] as const;

export function BrandRings() {
  const glow = useId();
  return (
    <svg
      aria-hidden="true"
      focusable="false"
      viewBox={VIEW_BOX}
      fill="none"
      className="pointer-events-none absolute end-0 bottom-0 w-full text-primary-chip rtl:-scale-x-100"
    >
      <defs>
        <radialGradient id={glow} cx={SIZE} cy={SIZE} r={SIZE} gradientUnits="userSpaceOnUse">
          <stop offset="0" stopColor="currentColor" stopOpacity="0.45" />
          <stop offset="1" stopColor="currentColor" stopOpacity="0" />
        </radialGradient>
      </defs>
      <rect width={SIZE} height={SIZE} fill={`url(#${glow})`} />
      {RING_RADII.map((radius) => (
        <circle
          key={radius}
          cx={SIZE}
          cy={SIZE}
          r={radius}
          strokeWidth="1"
          vectorEffect="non-scaling-stroke"
          className="stroke-nav-active"
        />
      ))}
    </svg>
  );
}
