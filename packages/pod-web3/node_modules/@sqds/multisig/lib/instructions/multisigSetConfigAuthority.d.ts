import { PublicKey } from "@solana/web3.js";
export declare function multisigSetConfigAuthority({ multisigPda, configAuthority, newConfigAuthority, memo, programId, }: {
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    newConfigAuthority: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=multisigSetConfigAuthority.d.ts.map