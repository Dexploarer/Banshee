import { PublicKey } from "@solana/web3.js";
import { Member } from "../generated";
export declare function multisigAddMember({ multisigPda, configAuthority, rentPayer, newMember, memo, programId, }: {
    multisigPda: PublicKey;
    configAuthority: PublicKey;
    rentPayer: PublicKey;
    newMember: Member;
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=multisigAddMember.d.ts.map