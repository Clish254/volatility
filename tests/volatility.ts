import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Volatility } from '../target/types/volatility';
import { SwitchboardTestContext } from '@switchboard-xyz/sbv2-utils';
import {
    AggregatorAccount,
    AggregatorHistoryRow,
} from '@switchboard-xyz/switchboard-v2';

export const AGGREGATOR_PUBKEY: anchor.web3.PublicKey =
    new anchor.web3.PublicKey('GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR');

export const HISTORY_BUFFER_PUBKEY: anchor.web3.PublicKey =
    new anchor.web3.PublicKey('7LLvRhMs73FqcLkA8jvEE1AM2mYZXTmqfUv8GAEurymx');

export const sleep = (ms: number): Promise<any> =>
    new Promise((s) => setTimeout(s, ms));

describe('volatility', () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Volatility as Program<Volatility>;

    let switchboard: SwitchboardTestContext;

    before(async () => {
        try {
            switchboard = await SwitchboardTestContext.loadDevnetQueue(
                program.provider as anchor.AnchorProvider,
                'F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy',
            );
            console.log('devnet detected');
            return;
        } catch (error: any) {
            console.log(`Error: SBV2 Devnet - ${error.message}`);
        }
        // If fails, throw error
        throw new Error(
            `Failed to load the SwitchboardTestContext from devnet`,
        );
    });

    it('Reads an aggregator history buffer and logs volatility', async () => {
        // const ONE_HOUR_AGO: number = Math.floor(Date.now()) - 60 * 60;

        const aggregatorAccount = new AggregatorAccount({
            program: switchboard.program,
            publicKey: AGGREGATOR_PUBKEY,
        });
        const aggregator = await aggregatorAccount.loadData();

        // TODO: Verify the correctness of values in the logs
        const history = await aggregatorAccount.loadHistory();
        console.log({ firstTimestamp: history[0].timestamp.toNumber() });
        console.log({
            lastTimestamp: history[history.length - 1].timestamp.toNumber(),
        });

        const tx = await program.methods
            .readHistory()
            .accounts({
                aggregator: AGGREGATOR_PUBKEY,
                historyBuffer: aggregator.historyBuffer,
            })
            .rpc();
        console.log('Your transaction signature', tx);

        await sleep(5000);

        const confirmedTxn =
            await program.provider.connection.getParsedTransaction(
                tx,
                'confirmed',
            );

        console.log(
            JSON.stringify(confirmedTxn?.meta?.logMessages, undefined, 2),
        );
    });
});
