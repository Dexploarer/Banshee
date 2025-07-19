import { PublicKey, VersionedTransaction } from "@solana/web3.js";
/**
 * Returns unsigned `VersionedTransaction` that needs to be
 * signed by `configAuthority` and `feePayer` before sending it.
 */
export declare function multisigSetTimeLock({ blockhash, feePayer, multisigPda, configAuthority, timeLock, memo, programId, }: {
    blockhash: string;
    feePayer: PublicKey;
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    timeLock: number;
    memo?: string;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=multisigSetTimeLock.d.ts.map