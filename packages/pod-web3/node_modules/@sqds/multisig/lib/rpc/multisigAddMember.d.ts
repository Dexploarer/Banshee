import { Connection, PublicKey, SendOptions, Signer, TransactionSignature } from "@solana/web3.js";
import { Member } from "../generated";
/** Add a member/key to the multisig and reallocate space if necessary. */
export declare function multisigAddMember({ connection, feePayer, multisigPda, configAuthority, rentPayer, newMember, memo, signers, sendOptions, programId, }: {
    connection: Connection;
    feePayer: Signer;
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    rentPayer: Signer;
    newMember: Member;
    memo?: string;
    signers?: Signer[];
    sendOptions?: SendOptions;
    programId?: PublicKey;
}): Promise<TransactionSignature>;
//# sourceMappingURL=multisigAddMember.d.ts.map