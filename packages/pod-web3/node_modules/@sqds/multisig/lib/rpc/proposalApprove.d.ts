import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
export declare function proposalApprove({ connection, feePayer, member, multisigPda, transactionIndex, memo, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    member: Signer;
    multisigPda: PublicKey;
    transactionIndex: bigint;
    memo?: string;
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=proposalApprove.d.ts.map