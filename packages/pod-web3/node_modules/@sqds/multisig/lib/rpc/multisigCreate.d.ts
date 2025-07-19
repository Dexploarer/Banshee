import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
import { Member } from "../generated";
/**
 * @deprecated This instruction is deprecated and will be removed soon. Please use `multisigCreateV2` to ensure future compatibility.
 *
 * Creates a new multisig.
 */
export declare function multisigCreate({ connection, createKey, creator, multisigPda, configAuthority, threshold, members, timeLock, memo, sendOptions, programId, }: {
    connection: Connection;
    createKey: Signer;
    creator: Signer;
    multisigPda: PublicKey;
    configAuthority: PublicKey | null;
    threshold: number;
    members: Member[];
    timeLock: number;
    memo?: string;
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=multisigCreate.d.ts.map