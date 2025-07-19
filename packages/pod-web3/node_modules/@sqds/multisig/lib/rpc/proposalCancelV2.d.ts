import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
/** Cancel a config transaction on behalf of the `member`. */
export declare function proposalCancelV2({ connection, feePayer, member, multisigPda, transactionIndex, memo, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    member: Signer;
    multisigPda: PublicKey;
    transactionIndex: bigint;
    memo?: string;
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=proposalCancelV2.d.ts.map