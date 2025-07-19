import { PublicKey } from "@solana/web3.js";
export declare function proposalCancelV2({ multisigPda, transactionIndex, member, memo, programId, }: {
    multisigPda: PublicKey;
    transactionIndex: bigint;
    member: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=proposalCancelV2.d.ts.map