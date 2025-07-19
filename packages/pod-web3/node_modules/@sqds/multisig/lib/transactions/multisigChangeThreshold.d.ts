import { PublicKey, VersionedTransaction } from "@solana/web3.js";
/**
 * Returns unsigned `VersionedTransaction` that needs to be
 * signed by `configAuthority` and `rentPayer` before sending it.
 */
export declare function multisigChangeThreshold({ blockhash, multisigPda, configAuthority, rentPayer, newThreshold, memo, programId, }: {
    blockhash: string;
    multisigPda: PublicKey;
    spendingLimit: PublicKey;
    configAuthority: PublicKey;
    rentPayer: PublicKey;
    newThreshold: number;
    memo?: string;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=multisigChangeThreshold.d.ts.map