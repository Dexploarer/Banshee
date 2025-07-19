import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
/** Add a member/key to the multisig and reallocate space if necessary. */
export declare function multisigRemoveMember({ connection, feePayer, multisigPda, configAuthority, oldMember, memo, signers, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    oldMember: PublicKey;
    memo?: string;
    signers?: Signer[];
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=multisigRemoveMember.d.ts.map