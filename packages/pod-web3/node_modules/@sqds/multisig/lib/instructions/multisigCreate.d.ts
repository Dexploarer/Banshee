import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { Member } from "../generated";
/** @deprecated This instruction is deprecated and will be removed soon. Please use `multisigCreateV2` to ensure future compatibility. */
export declare function multisigCreate({ creator, multisigPda, configAuthority, threshold, members, timeLock, createKey, memo, programId, }: {
    creator: PublicKey;
    multisigPda: PublicKey;
    configAuthority: PublicKey | null;
    threshold: number;
    members: Member[];
    timeLock: number;
    createKey: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): TransactionInstruction;
//# sourceMappingURL=multisigCreate.d.ts.map