# Test keys for the updater tests (backend/app/tests/updates.rs)

Throw-away keys made with `tauri signer generate`; the private keys were deleted. They are NOT the
release keys.

- `test.pub`: the public key given to the updater plugin in the tests.
- `bundle.bin`: stands for the installer package.
- `good.sig`: its signature by the matching private key.
- `other.sig`: its signature by a different private key (an update not signed by us).
