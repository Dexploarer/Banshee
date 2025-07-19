import { AddressLookupTableAccount, PublicKey, TransactionMessage } from "@solana/web3.js";
export declare function vaultTransactionCreate({ multisigPda, transactionIndex, creator, rentPayer, vaultIndex, ephemeralSigners, transactionMessage, addressLookupTableAccounts, memo, programId, }: {
    multisigPda: PublicKey;
    transactionIndex: bigint;
    creator: PublicKey;
    rentPayer?: PublicKey;
    vaultIndex: number;
    /** Number of additional signing PDAs required by the transaction. */
    ephemeralSigners: number;
    /** Transaction message to wrap into a multisig transaction. */
    transactionMessage: TransactionMessage;
    /** `AddressLookupTableAccount`s referenced in `transaction_message`. */
    addressLookupTableAccounts?: AddressLookupTableAccount[];
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=vaultTransactionCreate.d.ts.map