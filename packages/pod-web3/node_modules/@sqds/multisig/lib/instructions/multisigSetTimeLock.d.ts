import { PublicKey } from "@solana/web3.js";
export declare function multisigSetTimeLock({ multisigPda, configAuthority, timeLock, memo, programId, }: {
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    timeLock: number;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=multisigSetTimeLock.d.ts.map