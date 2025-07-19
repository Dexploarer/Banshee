import { AddressLookupTableAccount, Connection, PublicKey, TransactionInstruction } from "@solana/web3.js";
export declare function vaultTransactionExecute({ connection, multisigPda, transactionIndex, member, programId, }: {
    connection: Connection;
    multisigPda: PublicKey;
    transactionIndex: bigint;
    member: PublicKey;
    programId?: PublicKey;
}): Promise<{
    instruction: TransactionInstruction;
    lookupTableAccounts: AddressLookupTableAccount[];
}>;
//# sourceMappingURL=vaultTransactionExecute.d.ts.map