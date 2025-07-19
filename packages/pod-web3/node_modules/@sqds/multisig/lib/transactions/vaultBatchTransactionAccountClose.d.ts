import { PublicKey, VersionedTransaction } from "@solana/web3.js";
/**
 * Closes a VaultBatchTransaction belonging to the Batch and Proposal defined by `batchIndex`.
 * VaultBatchTransaction can be closed if either:
 * - it's marked as executed within the batch;
 * - the proposal is in a terminal state: `Executed`, `Rejected`, or `Cancelled`.
 * - the proposal is stale and not `Approved`.
 */
export declare function vaultBatchTransactionAccountClose({ blockhash, feePayer, multisigPda, rentCollector, batchIndex, transactionIndex, programId, }: {
    blockhash: string;
    feePayer: PublicKey;
    multisigPda: PublicKey;
    rentCollector: PublicKey;
    batchIndex: bigint;
    transactionIndex: number;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=vaultBatchTransactionAccountClose.d.ts.map