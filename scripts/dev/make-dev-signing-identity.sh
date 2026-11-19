#!/bin/bash
# Makes the self-signed code-signing identity "Mandi Dev" in the login keychain, once.
#
#   bash scripts/dev/make-dev-signing-identity.sh
#
# Why: see scripts/dev/run-signed.sh and plan 75 Section 7.8. With this identity every dev build of
# the app is signed the same way, so the Mac Keychain asks for the Mac password once, and your
# "Always Allow" lasts across rebuilds.
#
# It is safe to run again: when the identity already exists it only says so. Mac only. Dev only.
# macOS will ask for your Mac password, to trust the certificate and (optionally) to let
# `codesign` use the key without asking. The password is never stored or printed.

set -euo pipefail

NAME="Mandi Dev"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"
OPENSSL=/usr/bin/openssl   # LibreSSL: its PKCS#12 file is one that `security import` accepts.
P12_PASSWORD="mandi-dev-import"   # only protects a temporary file that is deleted at the end

echo "== Mandi Dev signing identity =="

if [ "$(uname -s)" != "Darwin" ]; then
  echo "This script is for macOS only. Nothing to do here."
  exit 0
fi

if security find-identity -v -p codesigning "$KEYCHAIN" 2>/dev/null | grep -q "\"$NAME\""; then
  echo "OK: the identity \"$NAME\" already exists and is trusted. Nothing to do."
  exit 0
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

echo "1/5 Making a certificate for \"$NAME\" (valid 10 years, code signing only)..."
cat > "$work/openssl.cnf" <<CNF
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $NAME
[ext]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
CNF
"$OPENSSL" req -x509 -newkey rsa:2048 -nodes -days 3650 \
  -keyout "$work/key.pem" -out "$work/cert.pem" -config "$work/openssl.cnf" 2>/dev/null

echo "2/5 Packing the key and the certificate..."
"$OPENSSL" pkcs12 -export -inkey "$work/key.pem" -in "$work/cert.pem" \
  -name "$NAME" -out "$work/identity.p12" -passout "pass:$P12_PASSWORD"

echo "3/5 Importing them into your login keychain (codesign may use the key)..."
security import "$work/identity.p12" -k "$KEYCHAIN" -P "$P12_PASSWORD" -T /usr/bin/codesign

echo "4/5 Trusting the certificate for code signing."
echo "    macOS will ask for your Mac password now. This is the only place it is needed."
security add-trusted-cert -r trustRoot -p codeSign -k "$KEYCHAIN" "$work/cert.pem"

echo "5/5 Letting codesign use the key without asking each time."
printf "    Type your Mac password to allow this (it is not shown), or press Enter to skip: "
read -r -s mac_password || mac_password=""
echo
if [ -n "$mac_password" ]; then
  if security set-key-partition-list -S apple-tool:,apple:,codesign: -s \
    -k "$mac_password" "$KEYCHAIN" >/dev/null 2>&1; then
    echo "    Done."
  else
    echo "    That did not work (wrong password?). codesign may ask once; choose Always Allow."
  fi
else
  echo "    Skipped. codesign may ask once; choose Always Allow."
fi
mac_password=""

if security find-identity -v -p codesigning "$KEYCHAIN" 2>/dev/null | grep -q "\"$NAME\""; then
  echo "OK: the identity \"$NAME\" is ready."
  echo "Next: run the app (pnpm dev). Enter your Mac password in the Keychain box once and choose"
  echo "\"Always Allow\". Rebuilds keep that choice."
else
  echo "Problem: the identity \"$NAME\" is not listed as a valid code-signing identity."
  echo "Open Keychain Access, find \"$NAME\", open it, and set Trust > Code Signing to Always Trust."
  exit 1
fi
