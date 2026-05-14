# Move Fixture — Coin Module

This fixture contains a single Move module `0x1::Coin` that demonstrates:

- `struct` definitions (`Coin`, `MintCapability`) with ability declarations
- `public fun` declarations (`initialize`, `mint`, `value`, `destroy`)
- `entry fun` declaration (`transfer`)
- `const` definition (`MAX_SUPPLY`)
- `use` declarations importing from `std::signer` and `0x1::Event`

Expected symbols: `Coin` (module/class), `Coin` (struct), `MintCapability` (struct),
`initialize`, `mint`, `value`, `transfer`, `destroy` (functions), `MAX_SUPPLY` (const).
