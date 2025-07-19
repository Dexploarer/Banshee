import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
export declare function spendingLimitUse({ connection, feePayer, member, multisigPda, spendingLimit, mint, vaultIndex, amount, decimals, destination, tokenProgram, memo, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    member: Signer;
    multisigPda: PublicKey;
    spendingLimit: PublicKey;
    /** Provide if `spendingLimit` is for an SPL token, omit if it's for SOL. */
    mint?: PublicKey;
    vaultIndex: number;
    amount: number;
    decimals: number;
    destination: PublicKey;
    tokenProgram?: PublicKey;
    memo?: string;
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=spendingLimitUse.d.ts.map