import { PublicKey } from "@solana/web3.js";
export declare function proposalCancel({ multisigPda, transactionIndex, member, memo, programId, }: {
    multisigPda: PublicKey;
    transactionIndex: bigint;
    member: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=proposalCancel.d.ts.map