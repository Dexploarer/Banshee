import { PublicKey } from "@solana/web3.js";
import { ConfigAction } from "../generated";
export declare function configTransactionCreate({ multisigPda, transactionIndex, creator, rentPayer, actions, memo, programId, }: {
    multisigPda: PublicKey;
    /** Member of the multisig that is creating the transaction. */
    creator: PublicKey;
    /** Payer for the transaction account rent. If not provided, `creator` is used. */
    rentPayer?: PublicKey;
    transactionIndex: bigint;
    actions: ConfigAction[];
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=configTransactionCreate.d.ts.map