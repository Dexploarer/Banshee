import { PublicKey, VersionedTransaction } from "@solana/web3.js";
/**
 * Returns unsigned `VersionedTransaction` that needs to be
 * signed by `configAuthority` and `feePayer` before sending it.
 */
export declare function multisigSetRentCollector({ blockhash, feePayer, multisigPda, configAuthority, newRentCollector, rentPayer, memo, programId, }: {
    blockhash: string;
    feePayer: PublicKey;
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    newRentCollector: PublicKey | null;
    rentPayer: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=multisigSetRentCollector.d.ts.map