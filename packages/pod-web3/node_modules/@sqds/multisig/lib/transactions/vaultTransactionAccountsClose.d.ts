import { PublicKey, VersionedTransaction } from "@solana/web3.js";
export declare function vaultTransactionAccountsClose({ blockhash, feePayer, multisigPda, rentCollector, transactionIndex, programId, }: {
    blockhash: string;
    feePayer: PublicKey;
    multisigPda: PublicKey;
    rentCollector: PublicKey;
    transactionIndex: bigint;
    programId?: PublicKey;
}): VersionedTransaction;
//# sourceMappingURL=vaultTransactionAccountsClose.d.ts.map