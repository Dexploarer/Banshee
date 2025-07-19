import { PublicKey, VersionedTransaction } from "@solana/web3.js";
/**
 * Returns unsigned `VersionedTransaction` that needs to be
 * signed by `configAuthority` and `feePayer` before sending it.
 */
export declare function multisigRemoveSpendingLimit({ blockhash, feePayer, multisigPda, configAuthority, spendingLimit, rentCollector, memo, programId, }: {
    blockhash: string;
    feePayer: PublicKey;
    multisigPda: PublicKey;
    spendingLimit: PublicKey;
    configAuthority: PublicKey;
    rentCollector: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=multisigRemoveSpendingLimit.d.ts.map