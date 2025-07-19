import { PublicKey } from "@solana/web3.js";
export declare function proposalReject({ multisigPda, transactionIndex, member, memo, programId, }: {
    multisigPda: PublicKey;
    transactionIndex: bigint;
    member: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=proposalReject.d.ts.map