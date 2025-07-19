import { BN, DriftClient, UserMap, OracleSource } from '@drift-labs/sdk';
import { Program, ProgramAccount } from '@coral-xyz/anchor';
import { DriftVaults } from './types/drift_vaults';
import { AccountMeta, AddressLookupTableAccount, PublicKey, TransactionInstruction, TransactionSignature, VersionedTransaction } from '@solana/web3.js';
import { Vault, VaultDepositor, VaultProtocol, VaultProtocolParams, WithdrawUnit } from './types/types';
import { UserMapConfig } from '@drift-labs/sdk';
import { Metaplex } from '@metaplex-foundation/js';
export type TxParams = {
    cuLimit?: number;
    cuPriceMicroLamports?: number;
    simulateTransaction?: boolean;
    lookupTables?: AddressLookupTableAccount[];
    oracleFeedsToCrank?: {
        feed: PublicKey;
        oracleSource: OracleSource;
    }[];
};
export declare class VaultClient {
    driftClient: DriftClient;
    metaplex?: Metaplex;
    program: Program<DriftVaults>;
    cliMode: boolean;
    /**
     * Cache map of drift user accounts of vaults.
     */
    readonly vaultUsers: UserMap;
    constructor({ driftClient, program, metaplex, cliMode, userMapConfig, }: {
        driftClient: DriftClient;
        program: Program<DriftVaults>;
        metaplex?: Metaplex;
        cliMode?: boolean;
        userMapConfig?: UserMapConfig;
    });
    /**
     * Unsubscribes from the vault users map. Call this to clean up any dangling promises.
     */
    unsubscribe(): Promise<void>;
    getVault(vault: PublicKey): Promise<Vault>;
    getVaultAndSlot(vault: PublicKey): Promise<{
        vault: Vault;
        slot: number;
    }>;
    getVaultDepositor(vaultDepositor: PublicKey): Promise<any>;
    getVaultDepositorAndSlot(vaultDepositor: PublicKey): Promise<{
        vaultDepositor: any;
        slot: number;
    }>;
    getVaultProtocolAddress(vault: PublicKey): PublicKey;
    getVaultProtocol(vaultProtocol: PublicKey): Promise<VaultProtocol>;
    getVaultProtocolAndSlot(vaultProtocol: PublicKey): Promise<{
        vaultProtocol: VaultProtocol;
        slot: number;
    }>;
    getAllVaultDepositorsWithNoWithdrawRequest(vault: PublicKey): Promise<ProgramAccount<VaultDepositor>[]>;
    getAllVaultDepositors(vault: PublicKey): Promise<ProgramAccount<VaultDepositor>[]>;
    getSubscribedVaultUser(vaultDriftUserAccountPubKey: PublicKey): Promise<import("@drift-labs/sdk").User>;
    /**
     *
     * @param vault pubkey
     * @param factorUnrealizedPNL add unrealized pnl to net balance
     * @returns vault equity, in USDC
     */
    calculateVaultEquity(params: {
        address?: PublicKey;
        vault?: Vault;
        factorUnrealizedPNL?: boolean;
    }): Promise<BN>;
    calculateVaultAllTimeNotionalPnl(params: {
        address?: PublicKey;
        vault?: Vault;
    }): Promise<BN>;
    /**
     *
     * @param vault pubkey
     * @param factorUnrealizedPNL add unrealized pnl to existing equity
     * @returns total vault equity, in spot deposit asset
     */
    calculateVaultEquityInDepositAsset(params: {
        address?: PublicKey;
        vault?: Vault;
        factorUnrealizedPNL?: boolean;
    }): Promise<BN>;
    /**
     * @param params
     * @returns vault depositor equity, in spot market value (which is usually USDC)
     */
    calculateWithdrawableVaultDepositorEquity(params: {
        vaultDepositorAddress?: PublicKey;
        vaultDepositor?: VaultDepositor;
        vaultAddress?: PublicKey;
        vault?: Vault;
    }): Promise<BN>;
    calculateWithdrawableVaultDepositorEquityInDepositAsset(params: {
        vaultDepositorAddress?: PublicKey;
        vaultDepositor?: VaultDepositor;
        vaultAddress?: PublicKey;
        vault?: Vault;
    }): Promise<BN>;
    calculateVaultProtocolEquity(params: {
        vault: PublicKey;
    }): Promise<BN>;
    initializeVault(params: {
        name: number[];
        spotMarketIndex: number;
        redeemPeriod: BN;
        maxTokens: BN;
        minDepositAmount: BN;
        managementFee: BN;
        profitShare: number;
        hurdleRate: number;
        permissioned: boolean;
        vaultProtocol?: VaultProtocolParams;
    }, uiTxParams?: TxParams): Promise<TransactionSignature>;
    /**
     * Updates the delegate address for a vault. The delegate address will be allowed to trade
     * on behalf of the vault.
     * @param vault vault address to update
     * @param delegate delegate address to update to
     * @returns
     */
    updateDelegate(vault: PublicKey, delegate: PublicKey, uiTxParams?: TxParams): Promise<TransactionSignature>;
    /**
     * Updates the vault margin trading status.
     * @param vault vault address to update
     * @param enabled whether to enable margin trading
     * @returns
     */
    updateMarginTradingEnabled(vault: PublicKey, enabled: boolean, uiTxParams?: TxParams): Promise<TransactionSignature>;
    private handleWSolMovement;
    /**
     *
     * @param vault vault address to deposit to
     * @param amount amount to deposit
     * @returns
     */
    managerDeposit(vault: PublicKey, amount: BN, uiTxParams?: TxParams, managerTokenAccount?: PublicKey): Promise<TransactionSignature>;
    managerRequestWithdraw(vault: PublicKey, amount: BN, withdrawUnit: WithdrawUnit, uiTxParams?: TxParams): Promise<TransactionSignature>;
    managerCancelWithdrawRequest(vault: PublicKey, uiTxParams?: TxParams): Promise<TransactionSignature>;
    managerWithdraw(vault: PublicKey, uiTxParams?: TxParams): Promise<TransactionSignature>;
    managerUpdateVault(vault: PublicKey, params: {
        redeemPeriod: BN | null;
        maxTokens: BN | null;
        managementFee: BN | null;
        minDepositAmount: BN | null;
        profitShare: number | null;
        hurdleRate: number | null;
        permissioned: boolean | null;
    }, uiTxParams?: TxParams): Promise<TransactionSignature>;
    getApplyProfitShareIx(vault: PublicKey, vaultDepositor: PublicKey): Promise<TransactionInstruction>;
    getApplyRebaseTokenizedDepositorIx(vault: PublicKey, tokenizedVaultDepositor: PublicKey): Promise<TransactionInstruction>;
    applyRebase(vault: PublicKey, vaultDepositor: PublicKey): Promise<TransactionSignature>;
    getApplyRebaseIx(vault: PublicKey, vaultDepositor: PublicKey): Promise<TransactionInstruction>;
    applyRebaseTokenizedDepositor(vault: PublicKey, tokenizedVaultDepositor: PublicKey): Promise<TransactionSignature>;
    private createInitVaultDepositorIx;
    /**
     * Initializes the vault depositor account. This account is used to deposit funds into a vault.
     * @param vault the vault address to deposit into
     * @param authority the authority allowed to make deposits into the vault
     * @returns
     */
    initializeVaultDepositor(vault: PublicKey, authority?: PublicKey, payer?: PublicKey, uiTxParams?: TxParams): Promise<TransactionSignature>;
    initializeTokenizedVaultDepositor(params: {
        vault: PublicKey;
        tokenName: string;
        tokenSymbol: string;
        tokenUri: string;
        decimals?: number;
        sharesBase?: number;
    }): Promise<TransactionSignature>;
    createTokenizeSharesIx(vaultDepositor: PublicKey, amount: BN, unit: WithdrawUnit, mint?: PublicKey): Promise<TransactionInstruction[]>;
    tokenizeShares(vaultDepositor: PublicKey, amount: BN, unit: WithdrawUnit, mint?: PublicKey, txParams?: TxParams): Promise<TransactionSignature>;
    createRedeemTokensIx(vaultDepositor: PublicKey, tokensToBurn: BN, sharesBase?: number): Promise<TransactionInstruction>;
    /**
     * Redeems tokens from the vault.
     * @param vaultDepositor
     * @param tokensToBurn
     * @param mint optionally provide a mint, or infer the mint from the current vault share base
     * @param txParams
     * @returns
     */
    redeemTokens(vaultDepositor: PublicKey, tokensToBurn: BN, sharesBase?: number, txParams?: TxParams): Promise<TransactionSignature>;
    prepDepositTx(vaultDepositor: PublicKey, amount: BN, initVaultDepositor?: {
        authority: PublicKey;
        vault: PublicKey;
    }, depositTokenAccount?: PublicKey): Promise<{
        vaultAccount: {
            name: number[];
            pubkey: PublicKey;
            manager: PublicKey;
            tokenAccount: PublicKey;
            userStats: PublicKey;
            user: PublicKey;
            delegate: PublicKey;
            liquidationDelegate: PublicKey;
            userShares: BN;
            totalShares: BN;
            lastFeeUpdateTs: BN;
            liquidationStartTs: BN;
            redeemPeriod: BN;
            totalWithdrawRequested: BN;
            maxTokens: BN;
            managementFee: BN;
            initTs: BN;
            netDeposits: BN;
            managerNetDeposits: BN;
            totalDeposits: BN;
            totalWithdraws: BN;
            managerTotalDeposits: BN;
            managerTotalWithdraws: BN;
            managerTotalFee: BN;
            managerTotalProfitShare: BN;
            minDepositAmount: BN;
            lastManagerWithdrawRequest: {
                shares: BN;
                value: BN;
                ts: BN;
            };
            sharesBase: number;
            profitShare: number;
            hurdleRate: number;
            spotMarketIndex: number;
            bump: number;
            permissioned: boolean;
            vaultProtocol: boolean;
            padding1: number[];
            padding: BN[];
        };
        accounts: {
            vault: PublicKey;
            vaultDepositor: PublicKey;
            vaultTokenAccount: PublicKey;
            driftUserStats: PublicKey;
            driftUser: PublicKey;
            driftState: PublicKey;
            driftSpotMarketVault: PublicKey;
            userTokenAccount: PublicKey;
            driftProgram: PublicKey;
            tokenProgram: PublicKey;
        };
        remainingAccounts: AccountMeta[];
        preIxs: TransactionInstruction[];
        postIxs: TransactionInstruction[];
    }>;
    /**
     * Creates a transaction to deposit funds into the specified vault.
     * Uses the associated token account of the vault depositor authority and spot market mint,
     * and assumes it exists before calling this function.
     * @param vaultDepositor
     * @param amount
     * @param initVaultDepositor If true, will initialize the vault depositor account
     * @returns transaction
     */
    createDepositTx(vaultDepositor: PublicKey, amount: BN, initVaultDepositor?: {
        authority: PublicKey;
        vault: PublicKey;
    }, txParams?: TxParams): Promise<VersionedTransaction>;
    /**
     * Depositor funds into the specified vault.
     * @param vaultDepositor
     * @param amount
     * @param initVaultDepositor If true, will initialize the vault depositor account
     * @param txParams
     * @returns
     */
    deposit(vaultDepositor: PublicKey, amount: BN, initVaultDepositor?: {
        authority: PublicKey;
        vault: PublicKey;
    }, txParams?: TxParams, userTokenAccount?: PublicKey): Promise<TransactionSignature>;
    requestWithdraw(vaultDepositor: PublicKey, amount: BN, withdrawUnit: WithdrawUnit, txParams?: TxParams): Promise<TransactionSignature>;
    withdraw(vaultDepositor: PublicKey, txParams?: TxParams): Promise<TransactionSignature>;
    forceWithdraw(vaultDepositor: PublicKey): Promise<TransactionSignature>;
    getForceWithdrawIx(vaultDepositor: PublicKey): Promise<TransactionInstruction[]>;
    cancelRequestWithdraw(vaultDepositor: PublicKey, txParams?: TxParams): Promise<TransactionSignature>;
    /**
     * Liquidates (become delegate for) a vault.
     * @param
     * @param
     * @returns
     */
    liquidate(vaultDepositor: PublicKey, txParams?: TxParams): Promise<TransactionSignature>;
    createTxn(vaultIxs: TransactionInstruction[], txParams?: TxParams): Promise<VersionedTransaction>;
    sendTxn(transaction: VersionedTransaction, simulateTransaction?: boolean): Promise<TransactionSignature>;
    /**
     * Used for UI wallet adapters compatibility
     */
    createAndSendTxn(vaultIxs: TransactionInstruction[], txParams?: TxParams): Promise<TransactionSignature>;
    /**
     * Initializes an insurance fund stake for the vault.
     * @param vault vault address to update
     * @param spotMarketIndex spot market index of the insurance fund stake
     * @returns
     */
    initializeInsuranceFundStake(vault: PublicKey, spotMarketIndex: number): Promise<TransactionSignature>;
    /**
     * Adds an amount to an insurance fund stake for the vault.
     * @param vault vault address to update
     * @param spotMarketIndex spot market index of the insurance fund stake
     * @param amount amount to add to the insurance fund stake, in spotMarketIndex precision
     * @returns
     */
    addToInsuranceFundStake(vault: PublicKey, spotMarketIndex: number, amount: BN, managerTokenAccount?: PublicKey): Promise<TransactionSignature>;
    requestRemoveInsuranceFundStake(vault: PublicKey, spotMarketIndex: number, amount: BN): Promise<TransactionSignature>;
    cancelRequestRemoveInsuranceFundStake(vault: PublicKey, spotMarketIndex: number): Promise<TransactionSignature>;
    removeInsuranceFundStake(vault: PublicKey, spotMarketIndex: number, managerTokenAccount?: PublicKey): Promise<TransactionSignature>;
    protocolRequestWithdraw(vault: PublicKey, amount: BN, withdrawUnit: WithdrawUnit): Promise<TransactionSignature>;
    protocolCancelWithdrawRequest(vault: PublicKey): Promise<TransactionSignature>;
    protocolWithdraw(vault: PublicKey): Promise<TransactionSignature>;
    private getOracleFeedsToCrank;
}
