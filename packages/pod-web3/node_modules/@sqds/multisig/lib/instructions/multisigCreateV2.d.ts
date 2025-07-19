import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { Member } from "../generated";
export declare function multisigCreateV2({ treasury, creator, multisigPda, configAuthority, threshold, members, timeLock, createKey, rentCollector, memo, programId, }: {
    treasury: PublicKey;
    creator: PublicKey;
    multisigPda: PublicKey;
    configAuthority: PublicKey | null;
    threshold: number;
    members: Member[];
    timeLock: number;
    createKey: PublicKey;
    rentCollector: PublicKey | null;
    memo?: string;
    programId?: PublicKey;
}): TransactionInstruction;
//# sourceMappingURL=multisigCreateV2.d.ts.map