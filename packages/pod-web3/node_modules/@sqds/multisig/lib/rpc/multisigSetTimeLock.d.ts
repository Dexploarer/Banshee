import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
/** Set the `time_lock` config parameter for the multisig. */
export declare function multisigSetTimeLock({ connection, feePayer, multisigPda, configAuthority, timeLock, memo, signers, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    timeLock: number;
    memo?: string;
    signers?: Signer[];
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=multisigSetTimeLock.d.ts.map