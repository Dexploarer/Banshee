import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
/**
 * Closes Batch and the corresponding Proposal accounts for proposals in terminal states:
 * `Executed`, `Rejected`, or `Cancelled` or stale proposals that aren't Approved.
 *
 * This instruction is only allowed to be executed when all `VaultBatchTransaction` accounts
 * in the `batch` are already closed: `batch.size == 0`.
 */
export declare function batchAccountsClose({ connection, feePayer, multisigPda, rentCollector, batchIndex, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    multisigPda: PublicKey;
    rentCollector: PublicKey;
    batchIndex: bigint;
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=batchAccountsClose.d.ts.map