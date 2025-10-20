# @dedot/wasm

WASM utilities for extracting metadata from Substrate/Polkadot runtime blobs. Built with Rust and compiled to WebAssembly for high performance.

## Installation

```bash
npm install @dedot/wasm
```

## Usage

```typescript
import { getMetadataFromWasmRuntime } from '@dedot/wasm'
import { readFileSync } from 'fs'

// Read your runtime WASM file
const wasmBytes = readFileSync('./runtime.wasm')
const wasmHex = `0x${wasmBytes.toString('hex')}` as `0x${string}`

// Extract metadata
const metadataHex = getMetadataFromWasmRuntime(wasmHex)

console.log('Metadata:', metadataHex)
// Output: 0x6d65746115... (metadata in hex format)
```

## API

### `getMetadataFromWasmRuntime(wasmBlob: HexString): HexString`

Extracts runtime metadata from a WASM blob. Automatically detects the best available metadata version (V16 → V15 → V14) and falls back to legacy extraction for older runtimes.

**Parameters:**
- `wasmBlob`: Runtime WASM as hex string with `0x` prefix

**Returns:**
- Metadata as hex string (ready for decoding with `RuntimeMetadataPrefixed`)

## License

MIT
