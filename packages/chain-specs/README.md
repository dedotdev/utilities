# @dedot/chain-specs

A collection of well-known chain specifications for connecting to Polkadot, Kusama, and testnet networks using [Dedot](https://dedot.dev) with the [Smoldot](https://github.com/smol-dot/smoldot) light client.

This package provides optimized chain specs with full ESM and CommonJS support via subpath exports, allowing you to import individual network specifications on-demand.

This package is a fork of [@substrate/connect-known-chains](https://github.com/paritytech/substrate-connect/tree/main/packages/connect-known-chains) with customizations!

## Installation

```bash
# npm
npm install @dedot/chain-specs

# yarn
yarn add @dedot/chain-specs

# pnpm
pnpm add @dedot/chain-specs
```

## Usage

### Basic Usage

Import chain specs and use them with Dedot and Smoldot:

```typescript
import { DedotClient, SmoldotProvider } from 'dedot';
import { start } from 'dedot/smoldot';
import { chainSpec } from '@dedot/chain-specs/polkadot';

const smoldot = start();
const chain = await smoldot.addChain({ chainSpec });
const provider = new SmoldotProvider(chain);
const client = await DedotClient.new(provider);
```

### Importing Multiple Networks

```typescript
// Import specific networks
import { chainSpec as polkadot } from '@dedot/chain-specs/polkadot';
import { chainSpec as kusama } from '@dedot/chain-specs/ksmcc3';
import { chainSpec as assetHub } from '@dedot/chain-specs/polkadot_asset_hub';

// Or import all at once
import { polkadot, ksmcc3, polkadot_asset_hub } from '@dedot/chain-specs';
```

## Supported Networks

- `polkadot` - Polkadot relay chain
- `polkadot_asset_hub` - Polkadot Asset Hub parachain
- `polkadot_bridge_hub` - Polkadot Bridge Hub parachain
- `polkadot_collectives` - Polkadot Collectives parachain
- `polkadot_people` - Polkadot People parachain
- `ksmcc3` - Kusama relay chain
- `ksmcc3_asset_hub` - Kusama Asset Hub parachain
- `ksmcc3_bridge_hub` - Kusama Bridge Hub parachain
- `ksmcc3_people` - Kusama People parachain
- `westend2` - Westend testnet relay chain
- `westend2_asset_hub` - Westend Asset Hub parachain
- `westend2_bridge_hub` - Westend Bridge Hub parachain
- `westend2_collectives` - Westend Collectives parachain
- `westend2_people` - Westend People parachain
- `paseo` - Paseo testnet
- `paseo_asset_hub` - Paseo Asset Hub
- `paseo_bridge_hub` - Paseo Bridge Hub
- `paseo_collectives` - Paseo Collectives
- `paseo_people` - Paseo People

## Documentation

For more information on connecting to networks with Dedot and Smoldot, visit the [Dedot documentation](https://docs.dedot.dev/getting-started/connect-to-network#smoldot).

## License

MIT
