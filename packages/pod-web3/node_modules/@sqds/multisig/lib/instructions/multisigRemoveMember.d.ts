import { PublicKey } from "@solana/web3.js";
export declare function multisigRemoveMember({ multisigPda, configAuthority, oldMember, memo, programId, }: {
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    oldMember: PublicKey;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=multisigRemoveMember.d.ts.map