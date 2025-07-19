import { PublicKey } from "@solana/web3.js";
export declare function configTransactionExecute({ multisigPda, transactionIndex, member, rentPayer, spendingLimits, programId, }: {
    multisigPda: PublicKey;
    transactionIndex: bigint;
    member: PublicKey;
    rentPayer?: PublicKey;
    /** In case the transaction adds or removes SpendingLimits, pass the array of their Pubkeys here. */
    spendingLimits?: PublicKey[];
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=configTransactionExecute.d.ts.map