import { SmoldotProvider } from '@dedot/providers';
import * as smoldot from 'smoldot';
import * as fs from 'fs';

const chainSpecFile = process.argv[2];

if (!chainSpecFile) {
  throw new Error('Please provide a path to the chain spec file as the first argument');
}

const chainSpec = fs.readFileSync(chainSpecFile, 'utf-8');

const client = smoldot.start();
const chain = await client.addChain({ chainSpec });

const provider = new SmoldotProvider(chain);

await provider.connect();

const genesisHash = await provider.send('chain_getBlockHash', [0]);

console.log(genesisHash, 'genesisHash');

await provider.disconnect();
await client.terminate();

process.exit(0);
