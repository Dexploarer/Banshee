import { PublicKey, VersionedTransaction } from "@solana/web3.js";
/**
 * Closes Batch and the corresponding Proposal accounts for proposals in terminal states:
 * `Executed`, `Rejected`, or `Cancelled` or stale proposals that aren't Approved.
 *
 * This instruction is only allowed to be executed when all `VaultBatchTransaction` accounts
 * in the `batch` are already closed: `batch.size == 0`.
 */
export declare function batchAccountsClose({ blockhash, feePayer, multisigPda, rentCollector, batchIndex, programId, }: {
    blockhash: string;
    feePayer: PublicKey;
    multisigPda: PublicKey;
    rentCollector: PublicKey;
    batchIndex: bigint;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=batchAccountsClose.d.ts.map