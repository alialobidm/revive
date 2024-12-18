# Changelog

## Unreleased

## v0.1.0-dev.6

This is a development pre-release.

# Added
- Implement the `BLOCKHASH` opcode.
- Implement delegate calls.
- Implement the `GASPRICE` opcode. Currently hard-coded to return `1`.
- The ELF shared object contract artifact is dumped into the debug output directory.
- Initial support for emitting debug info (opt in via the `-g` flag)

# Changed
- resolc now emits 64bit PolkaVM blobs, reducing contract code size and execution time.
- The RISC-V bit-manipulation target feature (`zbb`) is enabled.

# Fixed
- Compilation to Wasm (for usage in node and web browsers)


## v0.1.0-dev.5

This is development pre-release.

# Added
- Implement the `CODESIZE` and `EXTCODESIZE` opcodes.

# Changed
- Include the full revive version in the contract metadata.

# Fixed

## v0.1.0-dev-4

This is development pre-release.

# Added
- Support the `ORIGIN` opcode.

# Changed
- Update polkavm to `v0.14.0`.
- Enable the `a`, `fast-unaligned-access` and `xtheadcondmov` LLVM target features, decreasing the code size for some contracts.

# Fixed
