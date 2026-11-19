#!/bin/bash
# The cargo runner for macOS (backend/.cargo/config.toml). Cargo calls it as
#   run-signed.sh <binary> <arguments...>
# in place of running the binary itself.
#
# Why: every rebuild of the app gets a new ad-hoc code signature, so the Mac Keychain forgets that
# the user chose "Always Allow" and asks for the Mac password again (plan 75 Section 7.8). With the
# self-signed identity "Mandi Dev" (made once by scripts/dev/make-dev-signing-identity.sh) every
# build has the same signing identity and the same identifier, so one "Always Allow" lasts.
#
# It signs only the app binary (mandi-app), never a test binary, and only when the identity
# exists. When it does not, or signing fails, the binary runs exactly as before.
# Dev only: the Windows build and the release build never use this file.

binary="$1"
if [ "$#" -gt 0 ]; then
  shift
fi

if [ "$(basename "$binary")" = "mandi-app" ] \
  && command -v security >/dev/null 2>&1 \
  && command -v codesign >/dev/null 2>&1 \
  && security find-identity -v -p codesigning 2>/dev/null | grep -q '"Mandi Dev"'; then
  # A fixed identifier (the app's bundle identifier) keeps the designated requirement the same on
  # every rebuild, which is what the Keychain checks.
  if ! codesign --force -s "Mandi Dev" -i pk.mandisoftware.app "$binary" >/dev/null 2>&1; then
    echo "run-signed: could not sign $binary with \"Mandi Dev\"; running it unsigned." >&2
  fi
fi

exec "$binary" "$@"
