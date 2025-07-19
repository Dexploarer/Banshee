import { AddressLookupTableAccount, Connection, PublicKey, TransactionInstruction } from "@solana/web3.js";
export declare function batchExecuteTransaction({ connection, multisigPda, member, batchIndex, transactionIndex, programId, }: {
    connection: Connection;
    multisigPda: PublicKey;
    member: PublicKey;
    batchIndex: bigint;
    transactionIndex: number;
    programId?: PublicKey;
}): Promise<{
    instruction: TransactionInstruction;
    lookupTableAccounts: AddressLookupTableAccount[];
}>;
//# sourceMappingURL=batchExecuteTransaction.d.ts.map