import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
/**
 * Close the Proposal and ConfigTransaction accounts associated with a config transaction.
 */
export declare function vaultTransactionAccountsClose({ connection, feePayer, multisigPda, rentCollector, transactionIndex, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    multisigPda: PublicKey;
    rentCollector: PublicKey;
    transactionIndex: bigint;
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=vaultTransactionAccountsClose.d.ts.map