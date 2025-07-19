import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
import { Member } from "../generated";
/** Creates a new multisig. */
export declare function multisigCreateV2({ connection, treasury, createKey, creator, multisigPda, configAuthority, threshold, members, timeLock, rentCollector, memo, sendOptions, programId, }: {
    connection: Connection;
    treasury: PublicKey;
    createKey: Signer;
    creator: Signer;
    multisigPda: PublicKey;
    configAuthority: PublicKey | null;
    threshold: number;
    members: Member[];
    timeLock: number;
    rentCollector: PublicKey | null;
    memo?: string;
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=multisigCreateV2.d.ts.map