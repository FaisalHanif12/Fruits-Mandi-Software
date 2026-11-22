// The mark of the app (plan 75, plan 93 T2): the app icon itself, a gold mandi crate with an orange, a green apple and
// a leaf on the deep navy tile (`backend/app/icons/128x128@2x.png`, copied to `src/assets/app-icon.png`). Vite bundles
// the file with the app, so it needs no network (CSP `img-src 'self'`). It stands at the top of the main menu only (Umar,
// 22-11-2026: the fruit is back there; the sign-in panel has no logo). The picture has a clear margin around its tile;
// it is drawn a quarter larger inside a box that clips it, so the tile fills the box. It has no words, so it is the same
// in both languages and in both directions. It only decorates, so a screen reader skips it.

import appIcon from '../../assets/app-icon.png';

export function BrandMark() {
  return (
    <span aria-hidden="true" className="inline-flex size-target shrink-0 overflow-hidden">
      <img src={appIcon} alt="" draggable={false} className="size-full scale-125" />
    </span>
  );
}
