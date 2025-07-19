import { PublicKey } from "@solana/web3.js";
export declare function batchCreate({ multisigPda, creator, rentPayer, batchIndex, vaultIndex, memo, programId, }: {
    multisigPda: PublicKey;
    /** Member of the multisig that is creating the batch. */
    creator: PublicKey;
    /** Payer for the batch account rent. If not provided, `creator` is used. */
    rentPayer?: PublicKey;
    batchIndex: bigint;
    vaultIndex: number;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=batchCreate.d.ts.map