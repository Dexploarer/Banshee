import { PublicKey } from "@solana/web3.js";
export declare function multisigChangeThreshold({ multisigPda, configAuthority, rentPayer, newThreshold, memo, programId, }: {
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    rentPayer: PublicKey;
    newThreshold: number;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=multisigChangeThreshold.d.ts.map