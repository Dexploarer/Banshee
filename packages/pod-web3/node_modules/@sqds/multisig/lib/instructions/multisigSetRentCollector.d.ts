import { PublicKey } from "@solana/web3.js";
export declare function multisigSetRentCollector({ multisigPda, configAuthority, newRentCollector, rentPayer, memo, programId, }: {
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    newRentCollector: PublicKey | null;
    rentPayer: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=multisigSetRentCollector.d.ts.map