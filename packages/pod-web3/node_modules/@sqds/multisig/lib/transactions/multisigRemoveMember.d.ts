import { PublicKey, VersionedTransaction } from "@solana/web3.js";
/**
 * Returns unsigned `VersionedTransaction` that needs to be
 * signed by `configAuthority` and `feePayer` before sending it.
 */
export declare function multisigRemoveMember({ blockhash, feePayer, multisigPda, configAuthority, oldMember, memo, programId, }: {
    blockhash: string;
    feePayer: PublicKey;
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    oldMember: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=multisigRemoveMember.d.ts.map