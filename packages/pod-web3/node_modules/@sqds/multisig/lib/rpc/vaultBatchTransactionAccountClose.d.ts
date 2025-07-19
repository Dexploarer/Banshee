import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
/**
 * Closes a VaultBatchTransaction belonging to the Batch and Proposal defined by `batchIndex`.
 * VaultBatchTransaction can be closed if either:
 * - it's marked as executed within the batch;
 * - the proposal is in a terminal state: `Executed`, `Rejected`, or `Cancelled`.
 * - the proposal is stale and not `Approved`.
 */
export declare function vaultBatchTransactionAccountClose({ connection, feePayer, multisigPda, rentCollector, batchIndex, transactionIndex, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    multisigPda: PublicKey;
    rentCollector: PublicKey;
    batchIndex: bigint;
    transactionIndex: number;
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=vaultBatchTransactionAccountClose.d.ts.map