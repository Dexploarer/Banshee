import { PublicKey, VersionedTransaction } from "@solana/web3.js";
import { Member } from "../generated";
/**
 * @deprecated This instruction is deprecated and will be removed soon. Please use `multisigCreateV2` to ensure future compatibility.
 *
 * Returns unsigned `VersionedTransaction` that needs to be signed by `creator` and `createKey` before sending it.
 */
export declare function multisigCreate({ blockhash, configAuthority, createKey, creator, multisigPda, threshold, members, timeLock, memo, programId, }: {
    blockhash: string;
    createKey: PublicKey;
    creator: PublicKey;
    multisigPda: PublicKey;
    configAuthority: PublicKey | null;
    threshold: number;
    members: Member[];
    timeLock: number;
    memo?: string;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=multisigCreate.d.ts.map