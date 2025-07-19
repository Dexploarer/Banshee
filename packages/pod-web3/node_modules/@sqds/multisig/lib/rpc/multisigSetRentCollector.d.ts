import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
/** Set the multisig `rent_collector`. */
export declare function multisigSetRentCollector({ connection, feePayer, multisigPda, configAuthority, newRentCollector, rentPayer, memo, signers, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    newRentCollector: PublicKey | null;
    rentPayer: PublicKey;
    memo?: string;
    signers?: Signer[];
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=multisigSetRentCollector.d.ts.map