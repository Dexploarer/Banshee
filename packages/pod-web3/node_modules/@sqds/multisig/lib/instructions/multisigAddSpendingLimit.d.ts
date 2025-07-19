import { PublicKey } from "@solana/web3.js";
import { Period } from "../generated";
export declare function multisigAddSpendingLimit({ multisigPda, configAuthority, spendingLimit, rentPayer, createKey, vaultIndex, mint, amount, period, members, destinations, memo, programId, }: {
    multisigPda: PublicKey;
    spendingLimit: PublicKey;
    configAuthority: PublicKey;
    rentPayer: PublicKey;
    createKey: PublicKey;
    vaultIndex: number;
    mint: PublicKey;
    amount: bigint;
    period: Period;
    members: PublicKey[];
    destinations: PublicKey[];
    memo?: string;
    programId?: PublicKey;
}): import("@solana/web3.js").TransactionInstruction;
//# sourceMappingURL=multisigAddSpendingLimit.d.ts.map