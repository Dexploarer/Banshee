import { PublicKey } from "@solana/web3.js";
/**
 * Closes a VaultBatchTransaction belonging to the Batch and Proposal defined by `batchIndex`.
 * VaultBatchTransaction can be closed if either:
 * - it's marked as executed within the batch;
 * - the proposal is in a terminal state: `Executed`, `Rejected`, or `Cancelled`.
 * - the proposal is stale and not `Approved`.
 */
export declare function vaultBatchTransactionAccountClose({ multisigPda, rentCollector, batchIndex, transactionIndex, programId, }: {
    multisigPda: PublicKey;
    rentCollector: PublicKey;
    batchIndex: bigint;
    transactionIndex: number;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=vaultBatchTransactionAccountClose.d.ts.map