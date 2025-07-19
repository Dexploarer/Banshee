import { PublicKey, VersionedTransaction } from "@solana/web3.js";
/**
 * Returns unsigned `VersionedTransaction` that needs to be
 * signed by `member` and `feePayer` before sending it.
 */
export declare function proposalActivate({ blockhash, feePayer, multisigPda, transactionIndex, member, programId, }: {
    blockhash: string;
    feePayer: PublicKey;
    multisigPda: PublicKey;
    transactionIndex: bigint;
    member: PublicKey;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=proposalActivate.d.ts.map