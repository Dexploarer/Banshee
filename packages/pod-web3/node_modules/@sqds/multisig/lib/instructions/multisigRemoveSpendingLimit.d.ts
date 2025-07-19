import { PublicKey } from "@solana/web3.js";
export declare function multisigRemoveSpendingLimit({ multisigPda, configAuthority, spendingLimit, rentCollector, memo, programId, }: {
    multisigPda: PublicKey;
    spendingLimit: PublicKey;
    configAuthority: PublicKey;
    rentCollector: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=multisigRemoveSpendingLimit.d.ts.map