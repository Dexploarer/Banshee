import { PublicKey, VersionedTransaction } from "@solana/web3.js";
import { Member } from "../generated";
/**
 * Returns unsigned `VersionedTransaction` that needs to be signed by `creator` and `createKey` before sending it.
 */
export declare function multisigCreateV2({ blockhash, treasury, configAuthority, createKey, creator, multisigPda, threshold, members, timeLock, rentCollector, memo, programId, }: {
    blockhash: string;
    treasury: PublicKey;
    createKey: PublicKey;
    creator: PublicKey;
    multisigPda: PublicKey;
    configAuthority: PublicKey | null;
    threshold: number;
    members: Member[];
    timeLock: number;
    rentCollector: PublicKey | null;
    memo?: string;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=multisigCreateV2.d.ts.map