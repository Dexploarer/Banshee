"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.VaultClient = void 0;
const sdk_1 = require("@drift-labs/sdk");
const anchor_1 = require("@coral-xyz/anchor");
const addresses_1 = require("./addresses");
const web3_js_1 = require("@solana/web3.js");
const spl_token_1 = require("@solana/spl-token");
const bytes_1 = require("@coral-xyz/anchor/dist/cjs/utils/bytes");
const math_1 = require("./math");
const utils_1 = require("./utils");
class VaultClient {
    constructor({ driftClient, program, metaplex, cliMode, userMapConfig, }) {
        this.driftClient = driftClient;
        this.metaplex = metaplex;
        this.program = program;
        this.cliMode = !!cliMode;
        if (!userMapConfig) {
            this.vaultUsers = new sdk_1.UserMap({
                driftClient: driftClient,
                subscriptionConfig: {
                    type: 'polling',
                    frequency: 1000,
                    commitment: 'processed',
                },
            });
        }
        else {
            this.vaultUsers = new sdk_1.UserMap(userMapConfig);
        }
    }
    /**
     * Unsubscribes from the vault users map. Call this to clean up any dangling promises.
     */
    async unsubscribe() {
        await this.vaultUsers.unsubscribe();
    }
    async getVault(vault) {
        return await this.program.account.vault.fetch(vault);
    }
    async getVaultAndSlot(vault) {
        const vaultAndSlot = await this.program.account.vault.fetchAndContext(vault);
        return {
            vault: vaultAndSlot.data,
            slot: vaultAndSlot.context.slot,
        };
    }
    async getVaultDepositor(vaultDepositor) {
        return await this.program.account.vaultDepositor.fetch(vaultDepositor);
    }
    async getVaultDepositorAndSlot(vaultDepositor) {
        const vaultDepositorAndSlot = await this.program.account.vaultDepositor.fetchAndContext(vaultDepositor);
        return {
            vaultDepositor: vaultDepositorAndSlot.data,
            slot: vaultDepositorAndSlot.context.slot,
        };
    }
    getVaultProtocolAddress(vault) {
        return (0, addresses_1.getVaultProtocolAddressSync)(this.program.programId, vault);
    }
    async getVaultProtocol(vaultProtocol) {
        return await this.program.account.vaultProtocol.fetch(vaultProtocol);
    }
    async getVaultProtocolAndSlot(vaultProtocol) {
        const vaultProtocolAndSlot = await this.program.account.vaultProtocol.fetchAndContext(vaultProtocol);
        return {
            vaultProtocol: vaultProtocolAndSlot.data,
            slot: vaultProtocolAndSlot.context.slot,
        };
    }
    async getAllVaultDepositorsWithNoWithdrawRequest(vault) {
        const filters = [
            {
                // discriminator = VaultDepositor
                memcmp: {
                    offset: 0,
                    bytes: bytes_1.bs58.encode(anchor_1.BorshAccountsCoder.accountDiscriminator('VaultDepositor')),
                },
            },
            {
                // vault = vault
                memcmp: {
                    offset: 8,
                    bytes: vault.toBase58(),
                },
            },
            {
                // last_withdraw_request.shares (u128) = 0
                memcmp: {
                    offset: 112,
                    bytes: bytes_1.bs58.encode(new Uint8Array(16).fill(0)),
                },
            },
        ];
        // @ts-ignore
        return (await this.program.account.vaultDepositor.all(filters));
    }
    async getAllVaultDepositors(vault) {
        const filters = [
            {
                // discriminator = VaultDepositor
                memcmp: {
                    offset: 0,
                    bytes: bytes_1.bs58.encode(anchor_1.BorshAccountsCoder.accountDiscriminator('VaultDepositor')),
                },
            },
            {
                // vault = vault
                memcmp: {
                    offset: 8,
                    bytes: vault.toBase58(),
                },
            },
        ];
        // @ts-ignore
        return (await this.program.account.vaultDepositor.all(filters));
    }
    async getSubscribedVaultUser(vaultDriftUserAccountPubKey) {
        return this.vaultUsers.mustGet(vaultDriftUserAccountPubKey.toBase58(), {
            type: 'websocket',
        });
    }
    /**
     *
     * @param vault pubkey
     * @param factorUnrealizedPNL add unrealized pnl to net balance
     * @returns vault equity, in USDC
     */
    async calculateVaultEquity(params) {
        try {
            // defaults to true if undefined
            let factorUnrealizedPNL = true;
            if (params.factorUnrealizedPNL !== undefined) {
                factorUnrealizedPNL = params.factorUnrealizedPNL;
            }
            let vaultAccount;
            if (params.address !== undefined) {
                // @ts-ignore
                vaultAccount = await this.program.account.vault.fetch(params.address);
            }
            else if (params.vault !== undefined) {
                vaultAccount = params.vault;
            }
            else {
                throw new Error('Must supply address or vault');
            }
            const user = await this.getSubscribedVaultUser(vaultAccount.user);
            const netSpotValue = user.getNetSpotMarketValue();
            if (factorUnrealizedPNL) {
                const unrealizedPnl = user.getUnrealizedPNL(true, undefined, undefined);
                return netSpotValue.add(unrealizedPnl);
            }
            else {
                return netSpotValue;
            }
        }
        catch (err) {
            console.error('VaultClient ~ err:', err);
            return sdk_1.ZERO;
        }
    }
    async calculateVaultAllTimeNotionalPnl(params) {
        try {
            let vaultAccount;
            if (params.address !== undefined) {
                // @ts-ignore
                vaultAccount = await this.program.account.vault.fetch(params.address);
            }
            else if (params.vault !== undefined) {
                vaultAccount = params.vault;
            }
            else {
                throw new Error('Must supply address or vault');
            }
            const user = await this.getSubscribedVaultUser(vaultAccount.user);
            const allTimeTotalPnl = user.getTotalAllTimePnl();
            return allTimeTotalPnl;
        }
        catch (err) {
            console.error('VaultClient ~ err:', err);
            return sdk_1.ZERO;
        }
    }
    /**
     *
     * @param vault pubkey
     * @param factorUnrealizedPNL add unrealized pnl to existing equity
     * @returns total vault equity, in spot deposit asset
     */
    async calculateVaultEquityInDepositAsset(params) {
        let vaultAccount;
        if (params.address !== undefined) {
            vaultAccount = await this.program.account.vault.fetch(params.address);
        }
        else if (params.vault !== undefined) {
            vaultAccount = params.vault;
        }
        else {
            throw new Error('Must supply address or vault');
        }
        const vaultEquity = await this.calculateVaultEquity({
            vault: vaultAccount,
            factorUnrealizedPNL: params.factorUnrealizedPNL,
        });
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        const spotOracle = this.driftClient.getOracleDataForSpotMarket(vaultAccount.spotMarketIndex);
        const spotPrecision = sdk_1.TEN.pow(new sdk_1.BN(spotMarket.decimals));
        return vaultEquity.mul(spotPrecision).div(spotOracle.price);
    }
    /**
     * @param params
     * @returns vault depositor equity, in spot market value (which is usually USDC)
     */
    async calculateWithdrawableVaultDepositorEquity(params) {
        let vaultAccount;
        if (params.vaultAddress !== undefined) {
            vaultAccount = await this.program.account.vault.fetch(params.vaultAddress);
        }
        else if (params.vault !== undefined) {
            vaultAccount = params.vault;
        }
        else {
            throw new Error('Must supply vaultAddress or vault');
        }
        let vaultDepositorAccount;
        if (params.vaultDepositorAddress !== undefined) {
            vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(params.vaultDepositorAddress);
        }
        else if (params.vaultDepositor !== undefined) {
            vaultDepositorAccount = params.vaultDepositor;
        }
        else {
            throw new Error('Must supply vaultDepositorAddress or vaultDepositor');
        }
        const vaultEquity = await this.calculateVaultEquity({
            vault: vaultAccount,
            factorUnrealizedPNL: false,
        });
        return (0, math_1.calculateRealizedVaultDepositorEquity)(vaultDepositorAccount, vaultEquity, vaultAccount);
    }
    async calculateWithdrawableVaultDepositorEquityInDepositAsset(params) {
        let vaultAccount;
        if (params.vaultAddress !== undefined) {
            vaultAccount = await this.program.account.vault.fetch(params.vaultAddress);
        }
        else if (params.vault !== undefined) {
            vaultAccount = params.vault;
        }
        else {
            throw new Error('Must supply vaultAddress or vault');
        }
        let vaultDepositorAccount;
        if (params.vaultDepositorAddress !== undefined) {
            vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(params.vaultDepositorAddress);
        }
        else if (params.vaultDepositor !== undefined) {
            vaultDepositorAccount = params.vaultDepositor;
        }
        else {
            throw new Error('Must supply vaultDepositorAddress or vaultDepositor');
        }
        let vaultProtocol = undefined;
        if (vaultAccount.vaultProtocol) {
            vaultProtocol = await this.program.account.vaultProtocol.fetch(this.getVaultProtocolAddress(vaultAccount.pubkey));
        }
        const vaultEquity = await this.calculateVaultEquity({
            vault: vaultAccount,
            factorUnrealizedPNL: false,
        });
        const vdEquity = (0, math_1.calculateRealizedVaultDepositorEquity)(vaultDepositorAccount, vaultEquity, vaultAccount, vaultProtocol);
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        const spotOracle = this.driftClient.getOracleDataForSpotMarket(vaultAccount.spotMarketIndex);
        const spotPrecision = sdk_1.TEN.pow(new sdk_1.BN(spotMarket.decimals));
        return vdEquity.mul(spotPrecision).div(spotOracle.price);
    }
    async calculateVaultProtocolEquity(params) {
        const vaultAccount = await this.program.account.vault.fetch(params.vault);
        const vaultTotalEquity = await this.calculateVaultEquity({
            vault: vaultAccount,
        });
        const vaultProtocol = this.getVaultProtocolAddress(params.vault);
        const vpAccount = await this.program.account.vaultProtocol.fetch(vaultProtocol);
        return (0, sdk_1.unstakeSharesToAmount)(vpAccount.protocolProfitAndFeeShares, vaultAccount.totalShares, vaultTotalEquity);
    }
    async initializeVault(params, uiTxParams) {
        const { vaultProtocol: vaultProtocolParams, ...vaultParams } = params;
        const vault = (0, addresses_1.getVaultAddressSync)(this.program.programId, params.name);
        const tokenAccount = (0, addresses_1.getTokenVaultAddressSync)(this.program.programId, vault);
        const driftState = await this.driftClient.getStatePublicKey();
        const spotMarket = this.driftClient.getSpotMarketAccount(params.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${params.spotMarketIndex} not found on driftClient`);
        }
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault);
        const userKey = (0, sdk_1.getUserAccountPublicKeySync)(this.driftClient.program.programId, vault);
        const accounts = {
            driftSpotMarket: spotMarket.pubkey,
            driftSpotMarketMint: spotMarket.mint,
            driftUserStats: userStatsKey,
            driftUser: userKey,
            driftState,
            vault,
            tokenAccount,
            driftProgram: this.driftClient.program.programId,
        };
        const preIxs = [
            web3_js_1.ComputeBudgetProgram.setComputeUnitLimit({
                units: 400000,
            }),
            web3_js_1.ComputeBudgetProgram.setComputeUnitPrice({
                microLamports: 300000,
            }),
        ];
        if (vaultProtocolParams) {
            const vaultProtocol = this.getVaultProtocolAddress((0, addresses_1.getVaultAddressSync)(this.program.programId, params.name));
            const _params = {
                ...vaultParams,
                vaultProtocol: vaultProtocolParams,
            };
            if (this.cliMode) {
                return await this.program.methods
                    .initializeVaultWithProtocol(_params)
                    .preInstructions(preIxs)
                    .accounts({
                    ...accounts,
                    vaultProtocol,
                })
                    .rpc();
            }
            else {
                const uiAuthority = this.driftClient.wallet.publicKey;
                const initializeVaultWithProtocolIx = await this.program.methods
                    .initializeVaultWithProtocol(_params)
                    .accounts({
                    ...accounts,
                    vaultProtocol,
                    payer: uiAuthority,
                    manager: uiAuthority,
                })
                    .instruction();
                const ixs = [...preIxs, initializeVaultWithProtocolIx];
                return await this.createAndSendTxn(ixs, uiTxParams);
            }
        }
        else {
            const _params = vaultParams;
            if (this.cliMode) {
                return await this.program.methods
                    .initializeVault(_params)
                    .preInstructions(preIxs)
                    .accounts(accounts)
                    .rpc();
            }
            else {
                const uiAuthority = this.driftClient.wallet.publicKey;
                const initializeVaultIx = await this.program.methods
                    .initializeVault(_params)
                    .accounts({
                    ...accounts,
                    payer: uiAuthority,
                    manager: uiAuthority,
                })
                    .instruction();
                const ixs = [initializeVaultIx];
                return await this.createAndSendTxn(ixs, uiTxParams);
            }
        }
    }
    /**
     * Updates the delegate address for a vault. The delegate address will be allowed to trade
     * on behalf of the vault.
     * @param vault vault address to update
     * @param delegate delegate address to update to
     * @returns
     */
    async updateDelegate(vault, delegate, uiTxParams) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const accounts = {
            vault: vault,
            driftUser: vaultAccount.user,
            driftProgram: this.driftClient.program.programId,
        };
        if (this.cliMode) {
            return await this.program.methods
                .updateDelegate(delegate)
                .preInstructions([
                web3_js_1.ComputeBudgetProgram.setComputeUnitLimit({
                    units: 400000,
                }),
                web3_js_1.ComputeBudgetProgram.setComputeUnitPrice({
                    microLamports: 300000,
                }),
            ])
                .accounts(accounts)
                .rpc();
        }
        else {
            const updateDelegateIx = await this.program.methods
                .updateDelegate(delegate)
                .accounts({ ...accounts, manager: this.driftClient.wallet.publicKey })
                .instruction();
            return await this.createAndSendTxn([updateDelegateIx], uiTxParams);
        }
    }
    /**
     * Updates the vault margin trading status.
     * @param vault vault address to update
     * @param enabled whether to enable margin trading
     * @returns
     */
    async updateMarginTradingEnabled(vault, enabled, uiTxParams) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const accounts = {
            vault: vault,
            driftUser: vaultAccount.user,
            driftProgram: this.driftClient.program.programId,
        };
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        let remainingAccounts = [];
        try {
            remainingAccounts = this.driftClient.getRemainingAccounts({
                userAccounts: [user.getUserAccount()],
            });
        }
        catch (err) {
            // do nothing
        }
        if (this.cliMode) {
            return await this.program.methods
                .updateMarginTradingEnabled(enabled)
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .rpc();
        }
        else {
            const updateMarginTradingEnabledIx = await this.program.methods
                .updateMarginTradingEnabled(enabled)
                .accounts({ ...accounts, manager: this.driftClient.wallet.publicKey })
                .remainingAccounts(remainingAccounts)
                .instruction();
            return await this.createAndSendTxn([updateMarginTradingEnabledIx], uiTxParams);
        }
    }
    async handleWSolMovement(amount, driftSpotMarket, userTokenAccount) {
        const isSolDeposit = driftSpotMarket.mint.equals(sdk_1.WRAPPED_SOL_MINT);
        const preIxs = [];
        const postIxs = [];
        if (isSolDeposit) {
            const { ixs: createWSolAccountIxs, pubkey } = await this.driftClient.getWrappedSolAccountCreationIxs(amount, true);
            userTokenAccount = pubkey;
            preIxs.push(...createWSolAccountIxs);
            postIxs.push((0, spl_token_1.createCloseAccountInstruction)(userTokenAccount, this.driftClient.wallet.publicKey, this.driftClient.wallet.publicKey, []));
        }
        return { userTokenAccount, preIxs, postIxs };
    }
    /**
     *
     * @param vault vault address to deposit to
     * @param amount amount to deposit
     * @returns
     */
    async managerDeposit(vault, amount, uiTxParams, managerTokenAccount) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const driftSpotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!driftSpotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const accounts = {
            vault,
            vaultTokenAccount: vaultAccount.tokenAccount,
            driftUser: await (0, sdk_1.getUserAccountPublicKey)(this.driftClient.program.programId, vault),
            driftUserStats: (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault),
            driftProgram: this.driftClient.program.programId,
            driftState: await this.driftClient.getStatePublicKey(),
            driftSpotMarketVault: driftSpotMarket.vault,
            userTokenAccount: managerTokenAccount !== null && managerTokenAccount !== void 0 ? managerTokenAccount : (0, spl_token_1.getAssociatedTokenAddressSync)(driftSpotMarket.mint, this.driftClient.wallet.publicKey),
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        };
        const { userTokenAccount, preIxs, postIxs } = await this.handleWSolMovement(amount, driftSpotMarket, accounts.userTokenAccount);
        if (this.cliMode) {
            return await this.program.methods
                .managerDeposit(amount)
                .accounts({ ...accounts, userTokenAccount })
                .remainingAccounts(remainingAccounts)
                .preInstructions(preIxs)
                .postInstructions(postIxs)
                .rpc();
        }
        else {
            const managerDepositIx = await this.program.methods
                .managerDeposit(amount)
                .accounts({
                ...accounts,
                userTokenAccount,
                manager: this.driftClient.wallet.publicKey,
            })
                .remainingAccounts(remainingAccounts)
                .instruction();
            return await this.createAndSendTxn([...preIxs, managerDepositIx, ...postIxs], uiTxParams);
        }
    }
    async managerRequestWithdraw(vault, amount, withdrawUnit, uiTxParams) {
        this.program.idl.types;
        // @ts-ignore
        const vaultAccount = (await this.program.account.vault.fetch(vault));
        if (!this.driftClient.wallet.publicKey.equals(vaultAccount.manager)) {
            throw new Error(`Only the manager of the vault can request a withdraw.`);
        }
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault);
        const accounts = {
            vault,
            driftUser: vaultAccount.user,
            driftUserStats: userStatsKey,
        };
        if (this.cliMode) {
            return await this.program.methods
                // @ts-ignore, 0.29.0 anchor issues..
                .managerRequestWithdraw(amount, withdrawUnit)
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .rpc();
        }
        else {
            const requestWithdrawIx = this.program.instruction.managerRequestWithdraw(
            // @ts-ignore
            amount, withdrawUnit, {
                accounts: {
                    manager: this.driftClient.wallet.publicKey,
                    ...accounts,
                },
                remainingAccounts,
            });
            return await this.createAndSendTxn([requestWithdrawIx], uiTxParams);
        }
    }
    async managerCancelWithdrawRequest(vault, uiTxParams) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault);
        const accounts = {
            manager: this.driftClient.wallet.publicKey,
            vault,
            driftUser: vaultAccount.user,
            driftUserStats: userStatsKey,
        };
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        if (this.cliMode) {
            return await this.program.methods
                .mangerCancelWithdrawRequest()
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .rpc();
        }
        else {
            const cancelRequestWithdrawIx = this.program.instruction.mangerCancelWithdrawRequest({
                accounts: {
                    ...accounts,
                    driftUserStats: (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault),
                    manager: this.driftClient.wallet.publicKey,
                },
                remainingAccounts,
            });
            return await this.createAndSendTxn([cancelRequestWithdrawIx], uiTxParams);
        }
    }
    async managerWithdraw(vault, uiTxParams) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        if (!this.driftClient.wallet.publicKey.equals(vaultAccount.manager)) {
            throw new Error(`Only the manager of the vault can request a withdraw.`);
        }
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const ix = this.program.instruction.managerWithdraw({
            accounts: {
                vault,
                manager: this.driftClient.wallet.publicKey,
                vaultTokenAccount: vaultAccount.tokenAccount,
                driftUser: await (0, sdk_1.getUserAccountPublicKey)(this.driftClient.program.programId, vault),
                driftProgram: this.driftClient.program.programId,
                driftUserStats: (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault),
                driftState: await this.driftClient.getStatePublicKey(),
                driftSpotMarketVault: spotMarket.vault,
                userTokenAccount: (0, spl_token_1.getAssociatedTokenAddressSync)(spotMarket.mint, this.driftClient.wallet.publicKey),
                driftSigner: this.driftClient.getStateAccount().signer,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
            },
            remainingAccounts,
        });
        return this.createAndSendTxn([ix], uiTxParams);
    }
    async managerUpdateVault(vault, params, uiTxParams) {
        const ix = this.program.instruction.updateVault(params, {
            accounts: {
                vault,
                manager: this.driftClient.wallet.publicKey,
            },
        });
        if (this.cliMode) {
            return this.createAndSendTxn([ix], {
                cuLimit: 600000,
                cuPriceMicroLamports: 10000,
            });
        }
        else {
            return this.createAndSendTxn([ix], uiTxParams);
        }
    }
    async getApplyProfitShareIx(vault, vaultDepositor) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        const accounts = {
            vault,
            vaultDepositor,
            manager: this.driftClient.wallet.publicKey,
            driftUserStats: (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault),
            driftUser: await (0, sdk_1.getUserAccountPublicKey)(this.driftClient.program.programId, vault),
            driftState: await this.driftClient.getStatePublicKey(),
            driftSigner: this.driftClient.getStateAccount().signer,
            driftProgram: this.driftClient.program.programId,
        };
        return this.program.instruction.applyProfitShare({
            accounts: {
                ...accounts,
            },
            remainingAccounts,
        });
    }
    async getApplyRebaseTokenizedDepositorIx(vault, tokenizedVaultDepositor) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        const accounts = {
            vault,
            tokenizedVaultDepositor,
            driftUser: await (0, sdk_1.getUserAccountPublicKey)(this.driftClient.program.programId, vault),
            driftState: await this.driftClient.getStatePublicKey(),
            driftSigner: this.driftClient.getStateAccount().signer,
            driftProgram: this.driftClient.program.programId,
        };
        return this.program.instruction.applyRebaseTokenizedDepositor({
            accounts: {
                ...accounts,
            },
            remainingAccounts,
        });
    }
    async applyRebase(vault, vaultDepositor) {
        return await this.createAndSendTxn([
            await this.getApplyRebaseIx(vault, vaultDepositor),
        ]);
    }
    async getApplyRebaseIx(vault, vaultDepositor) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        const accounts = {
            vault,
            vaultDepositor,
            driftUser: await (0, sdk_1.getUserAccountPublicKey)(this.driftClient.program.programId, vault),
            driftState: await this.driftClient.getStatePublicKey(),
            driftSigner: this.driftClient.getStateAccount().signer,
            driftProgram: this.driftClient.program.programId,
        };
        return this.program.instruction.applyRebase({
            accounts: {
                ...accounts,
            },
            remainingAccounts,
        });
    }
    async applyRebaseTokenizedDepositor(vault, tokenizedVaultDepositor) {
        return await this.createAndSendTxn([
            await this.getApplyRebaseTokenizedDepositorIx(vault, tokenizedVaultDepositor),
        ]);
    }
    createInitVaultDepositorIx(vault, authority, payer) {
        const vaultDepositor = (0, addresses_1.getVaultDepositorAddressSync)(this.program.programId, vault, authority || this.driftClient.wallet.publicKey);
        const accounts = {
            vaultDepositor,
            vault,
            authority: authority || this.driftClient.wallet.publicKey,
        };
        const initIx = this.program.instruction.initializeVaultDepositor({
            accounts: {
                ...accounts,
                payer: payer || authority || this.driftClient.wallet.publicKey,
                rent: web3_js_1.SYSVAR_RENT_PUBKEY,
                systemProgram: web3_js_1.SystemProgram.programId,
            },
        });
        return initIx;
    }
    /**
     * Initializes the vault depositor account. This account is used to deposit funds into a vault.
     * @param vault the vault address to deposit into
     * @param authority the authority allowed to make deposits into the vault
     * @returns
     */
    async initializeVaultDepositor(vault, authority, payer, uiTxParams) {
        const vaultDepositor = (0, addresses_1.getVaultDepositorAddressSync)(this.program.programId, vault, authority || this.driftClient.wallet.publicKey);
        const accounts = {
            vaultDepositor,
            vault,
            authority: authority || this.driftClient.wallet.publicKey,
        };
        if (this.cliMode) {
            return this.program.methods
                .initializeVaultDepositor()
                .accounts({
                ...accounts,
                payer: payer || authority || this.driftClient.wallet.publicKey,
                rent: web3_js_1.SYSVAR_RENT_PUBKEY,
                systemProgram: web3_js_1.SystemProgram.programId,
            })
                .rpc();
        }
        else {
            const initIx = this.createInitVaultDepositorIx(vault, authority, payer);
            return await this.createAndSendTxn([initIx], uiTxParams);
        }
    }
    async initializeTokenizedVaultDepositor(params) {
        var _a;
        if (!this.metaplex) {
            throw new Error('Metaplex instance is required when constructing VaultClient to initialize a tokenized vault depositor');
        }
        let spotMarketDecimals = 6;
        let sharesBase = 0;
        if (params.decimals === undefined || params.sharesBase === undefined) {
            const vault = await this.program.account.vault.fetch(params.vault);
            const spotMarketAccount = this.driftClient.getSpotMarketAccount(vault.spotMarketIndex);
            if (!spotMarketAccount) {
                throw new Error(`DriftClient failed to load vault's spot market (marketIndex: ${vault.spotMarketIndex})`);
            }
            spotMarketDecimals = spotMarketAccount.decimals;
            sharesBase = vault.sharesBase;
        }
        const mintAddress = (0, addresses_1.getTokenizedVaultMintAddressSync)(this.program.programId, params.vault, sharesBase);
        const accounts = {
            vault: params.vault,
            vaultDepositor: (0, addresses_1.getTokenizedVaultAddressSync)(this.program.programId, params.vault, sharesBase),
            mintAccount: mintAddress,
            metadataAccount: this.metaplex.nfts().pdas().metadata({
                mint: mintAddress,
            }),
            tokenMetadataProgram: this.metaplex.programs().getTokenMetadata().address,
            payer: this.driftClient.wallet.publicKey,
        };
        const vaultTokenAta = (0, spl_token_1.getAssociatedTokenAddressSync)(mintAddress, params.vault, true);
        const createAtaIx = (0, spl_token_1.createAssociatedTokenAccountInstruction)(this.driftClient.wallet.publicKey, vaultTokenAta, params.vault, mintAddress);
        if (!this.cliMode) {
            throw new Error('CLI mode is not supported for initializeTokenizedVaultDepositor');
        }
        return await this.program.methods
            .initializeTokenizedVaultDepositor({
            ...params,
            decimals: (_a = params.decimals) !== null && _a !== void 0 ? _a : spotMarketDecimals,
        })
            .preInstructions([
            web3_js_1.ComputeBudgetProgram.setComputeUnitPrice({
                microLamports: 50000,
            }),
        ])
            .postInstructions([createAtaIx])
            .accounts(accounts)
            .rpc();
    }
    async createTokenizeSharesIx(vaultDepositor, amount, unit, mint) {
        const vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(vaultDepositor);
        const vaultAccount = await this.program.account.vault.fetch(vaultDepositorAccount.vault);
        mint =
            mint !== null && mint !== void 0 ? mint : (0, addresses_1.getTokenizedVaultMintAddressSync)(this.program.programId, vaultDepositorAccount.vault, vaultAccount.sharesBase);
        const userAta = (0, spl_token_1.getAssociatedTokenAddressSync)(mint, this.driftClient.wallet.publicKey, true);
        const ixs = [];
        const userAtaExists = await this.driftClient.connection.getAccountInfo(userAta);
        if (userAtaExists === null) {
            ixs.push((0, spl_token_1.createAssociatedTokenAccountInstruction)(this.driftClient.wallet.publicKey, userAta, this.driftClient.wallet.publicKey, mint));
        }
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        ixs.push(await this.program.methods
            // anchor idl bug: https://github.com/coral-xyz/anchor/issues/2914
            // @ts-ignore
            .tokenizeShares(amount, unit)
            .accounts({
            authority: this.driftClient.wallet.publicKey,
            vault: vaultDepositorAccount.vault,
            vaultDepositor,
            tokenizedVaultDepositor: (0, addresses_1.getTokenizedVaultAddressSync)(this.program.programId, vaultDepositorAccount.vault, vaultAccount.sharesBase),
            mint,
            userTokenAccount: userAta,
            driftUser: vaultAccount.user,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        })
            .remainingAccounts(remainingAccounts)
            .instruction());
        return ixs;
    }
    async tokenizeShares(vaultDepositor, amount, unit, mint, txParams) {
        const ixs = await this.createTokenizeSharesIx(vaultDepositor, amount, unit, mint);
        if (this.cliMode) {
            try {
                const tx = new web3_js_1.Transaction().add(...ixs);
                const txSig = await this.driftClient.txSender.send(tx, undefined, undefined, false);
                return txSig.txSig;
            }
            catch (e) {
                console.error(e);
                throw e;
            }
        }
        else {
            return await this.createAndSendTxn(ixs, txParams);
        }
    }
    async createRedeemTokensIx(vaultDepositor, tokensToBurn, sharesBase) {
        const vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(vaultDepositor);
        const vaultAccount = await this.program.account.vault.fetch(vaultDepositorAccount.vault);
        const mint = (0, addresses_1.getTokenizedVaultMintAddressSync)(this.program.programId, vaultDepositorAccount.vault, sharesBase !== null && sharesBase !== void 0 ? sharesBase : vaultAccount.sharesBase);
        const userAta = (0, spl_token_1.getAssociatedTokenAddressSync)(mint, this.driftClient.wallet.publicKey, true);
        const vaultTokenAta = (0, spl_token_1.getAssociatedTokenAddressSync)(mint, vaultDepositorAccount.vault, true);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        return await this.program.methods
            .redeemTokens(tokensToBurn)
            .accounts({
            authority: this.driftClient.wallet.publicKey,
            vault: vaultDepositorAccount.vault,
            vaultDepositor,
            tokenizedVaultDepositor: (0, addresses_1.getTokenizedVaultAddressSync)(this.program.programId, vaultDepositorAccount.vault, sharesBase !== null && sharesBase !== void 0 ? sharesBase : vaultAccount.sharesBase),
            mint,
            userTokenAccount: userAta,
            vaultTokenAccount: vaultTokenAta,
            driftUser: vaultAccount.user,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        })
            .remainingAccounts(remainingAccounts)
            .instruction();
    }
    /**
     * Redeems tokens from the vault.
     * @param vaultDepositor
     * @param tokensToBurn
     * @param mint optionally provide a mint, or infer the mint from the current vault share base
     * @param txParams
     * @returns
     */
    async redeemTokens(vaultDepositor, tokensToBurn, sharesBase, txParams) {
        const ix = await this.createRedeemTokensIx(vaultDepositor, tokensToBurn, sharesBase);
        if (this.cliMode) {
            try {
                const tx = new web3_js_1.Transaction().add(ix);
                const txSig = await this.driftClient.txSender.send(tx, undefined, undefined, false);
                return txSig.txSig;
            }
            catch (e) {
                console.error(e);
                throw e;
            }
        }
        else {
            return await this.createAndSendTxn([ix], txParams);
        }
    }
    async prepDepositTx(vaultDepositor, amount, initVaultDepositor, depositTokenAccount) {
        let vaultPubKey;
        if (initVaultDepositor) {
            vaultPubKey = initVaultDepositor.vault;
        }
        else {
            const vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(vaultDepositor);
            vaultPubKey = vaultDepositorAccount.vault;
        }
        const vaultAccount = await this.program.account.vault.fetch(vaultPubKey);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vaultPubKey);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vaultPubKey);
        const driftStateKey = await this.driftClient.getStatePublicKey();
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const nonWSolUserTokenAccount = depositTokenAccount !== null && depositTokenAccount !== void 0 ? depositTokenAccount : (0, spl_token_1.getAssociatedTokenAddressSync)(spotMarket.mint, this.driftClient.wallet.publicKey, true);
        const { userTokenAccount, preIxs, postIxs } = await this.handleWSolMovement(amount, spotMarket, nonWSolUserTokenAccount);
        const accounts = {
            vault: vaultPubKey,
            vaultDepositor,
            vaultTokenAccount: vaultAccount.tokenAccount,
            driftUserStats: userStatsKey,
            driftUser: vaultAccount.user,
            driftState: driftStateKey,
            driftSpotMarketVault: spotMarket.vault,
            userTokenAccount: userTokenAccount,
            driftProgram: this.driftClient.program.programId,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        };
        return {
            vaultAccount,
            accounts,
            remainingAccounts,
            preIxs,
            postIxs,
        };
    }
    /**
     * Creates a transaction to deposit funds into the specified vault.
     * Uses the associated token account of the vault depositor authority and spot market mint,
     * and assumes it exists before calling this function.
     * @param vaultDepositor
     * @param amount
     * @param initVaultDepositor If true, will initialize the vault depositor account
     * @returns transaction
     */
    async createDepositTx(vaultDepositor, amount, initVaultDepositor, txParams) {
        const { vaultAccount, accounts, remainingAccounts, preIxs, postIxs } = await this.prepDepositTx(vaultDepositor, amount, initVaultDepositor);
        const ixs = [];
        if (initVaultDepositor) {
            ixs.push(this.createInitVaultDepositorIx(vaultAccount.pubkey, initVaultDepositor.authority));
        }
        const depositIx = await this.program.methods
            .deposit(amount)
            .accounts({
            authority: this.driftClient.wallet.publicKey,
            ...accounts,
        })
            .remainingAccounts(remainingAccounts)
            .instruction();
        ixs.push(...preIxs);
        ixs.push(depositIx);
        ixs.push(...postIxs);
        return await this.createTxn(ixs, txParams);
    }
    /**
     * Depositor funds into the specified vault.
     * @param vaultDepositor
     * @param amount
     * @param initVaultDepositor If true, will initialize the vault depositor account
     * @param txParams
     * @returns
     */
    async deposit(vaultDepositor, amount, initVaultDepositor, txParams, userTokenAccount) {
        if (this.cliMode) {
            const { vaultAccount, accounts, remainingAccounts, preIxs, postIxs } = await this.prepDepositTx(vaultDepositor, amount, initVaultDepositor, userTokenAccount);
            if (initVaultDepositor) {
                await this.initializeVaultDepositor(vaultAccount.pubkey, initVaultDepositor.authority);
            }
            return this.program.methods
                .deposit(amount)
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .preInstructions(preIxs)
                .postInstructions(postIxs)
                .rpc();
        }
        else {
            const depositTxn = await this.createDepositTx(vaultDepositor, amount, initVaultDepositor, txParams);
            return this.sendTxn(depositTxn, txParams === null || txParams === void 0 ? void 0 : txParams.simulateTransaction);
        }
    }
    async requestWithdraw(vaultDepositor, amount, withdrawUnit, txParams) {
        const vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(vaultDepositor);
        const vaultAccount = await this.program.account.vault.fetch(vaultDepositorAccount.vault);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vaultDepositorAccount.vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vaultDepositorAccount.vault);
        const accounts = {
            vault: vaultDepositorAccount.vault,
            vaultDepositor,
            driftUser: vaultAccount.user,
            driftUserStats: userStatsKey,
        };
        if (this.cliMode) {
            return await this.program.methods
                // @ts-ignore
                .requestWithdraw(amount, withdrawUnit)
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .rpc();
        }
        else {
            const oracleFeedsToCrankIxs = await this.getOracleFeedsToCrank(txParams === null || txParams === void 0 ? void 0 : txParams.oracleFeedsToCrank);
            const requestWithdrawIx = this.program.instruction.requestWithdraw(
            // @ts-ignore
            amount, withdrawUnit, {
                accounts: {
                    authority: this.driftClient.wallet.publicKey,
                    ...accounts,
                },
                remainingAccounts,
            });
            return await this.createAndSendTxn([...oracleFeedsToCrankIxs, requestWithdrawIx], txParams);
        }
    }
    async withdraw(vaultDepositor, txParams) {
        var _a;
        const vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(vaultDepositor);
        const vaultAccount = await this.program.account.vault.fetch(vaultDepositorAccount.vault);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        const vaultProtocol = this.getVaultProtocolAddress(vaultDepositorAccount.vault);
        if (!vaultProtocol.equals(web3_js_1.SystemProgram.programId)) {
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vaultDepositorAccount.vault);
        const driftStateKey = await this.driftClient.getStatePublicKey();
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const isSolMarket = spotMarket.mint.equals(sdk_1.WRAPPED_SOL_MINT);
        // let createAtaIx: TransactionInstruction | undefined = undefined;
        let userAta = (0, spl_token_1.getAssociatedTokenAddressSync)(spotMarket.mint, this.driftClient.wallet.publicKey, true);
        const preIxs = [];
        const postIxs = [];
        if (isSolMarket) {
            const { ixs, pubkey } = await this.driftClient.getWrappedSolAccountCreationIxs(sdk_1.ZERO, false);
            userAta = pubkey;
            preIxs.push(...ixs);
            postIxs.push((0, spl_token_1.createSyncNativeInstruction)(userAta));
            postIxs.push((0, spl_token_1.createCloseAccountInstruction)(userAta, this.driftClient.wallet.publicKey, this.driftClient.wallet.publicKey, []));
        }
        else {
            const userAtaExists = await this.driftClient.connection.getAccountInfo(userAta);
            if (userAtaExists === null) {
                preIxs.push((0, spl_token_1.createAssociatedTokenAccountInstruction)(this.driftClient.wallet.publicKey, userAta, this.driftClient.wallet.publicKey, spotMarket.mint));
            }
        }
        const accounts = {
            vault: vaultDepositorAccount.vault,
            vaultDepositor,
            vaultTokenAccount: vaultAccount.tokenAccount,
            driftUserStats: userStatsKey,
            driftUser: vaultAccount.user,
            driftState: driftStateKey,
            driftSpotMarketVault: spotMarket.vault,
            driftSigner: this.driftClient.getStateAccount().signer,
            userTokenAccount: userAta,
            driftProgram: this.driftClient.program.programId,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        };
        if (this.cliMode) {
            return await this.program.methods
                .withdraw()
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .preInstructions(preIxs)
                .postInstructions(postIxs)
                .rpc();
        }
        else {
            const oracleFeedsToCrankIxs = await this.getOracleFeedsToCrank(txParams === null || txParams === void 0 ? void 0 : txParams.oracleFeedsToCrank);
            const ixs = [
                ...oracleFeedsToCrankIxs,
                ...preIxs,
                await this.program.methods
                    .withdraw()
                    .accounts({
                    authority: this.driftClient.wallet.publicKey,
                    ...accounts,
                })
                    .remainingAccounts(remainingAccounts)
                    .instruction(),
                ...postIxs,
            ];
            const creationIxs = preIxs.concat(postIxs).length;
            return await this.createAndSendTxn(ixs, {
                cuLimit: ((_a = txParams === null || txParams === void 0 ? void 0 : txParams.cuLimit) !== null && _a !== void 0 ? _a : 650000) + (creationIxs > 0 ? 200000 : 0),
                ...txParams,
            });
        }
    }
    async forceWithdraw(vaultDepositor) {
        const ix = await this.getForceWithdrawIx(vaultDepositor);
        return await this.createAndSendTxn(ix);
    }
    async getForceWithdrawIx(vaultDepositor) {
        const vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(vaultDepositor);
        const vaultAccount = await this.program.account.vault.fetch(vaultDepositorAccount.vault);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vaultDepositorAccount.vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vaultDepositorAccount.vault);
        const driftStateKey = await this.driftClient.getStatePublicKey();
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const [userTokenAccount, createAtaIx] = await (0, utils_1.getOrCreateATAInstruction)(spotMarket.mint, vaultDepositorAccount.authority, this.driftClient.connection, true, this.driftClient.wallet.publicKey);
        if (createAtaIx) {
            console.log(`Creating ATA for ${vaultDepositorAccount.authority.toBase58()} to ${userTokenAccount.toBase58()}`);
        }
        const accounts = {
            manager: this.driftClient.wallet.publicKey,
            vault: vaultDepositorAccount.vault,
            vaultDepositor,
            vaultTokenAccount: vaultAccount.tokenAccount,
            driftUserStats: userStatsKey,
            driftUser: vaultAccount.user,
            driftState: driftStateKey,
            driftSpotMarketVault: spotMarket.vault,
            driftSigner: this.driftClient.getStateAccount().signer,
            userTokenAccount,
            driftProgram: this.driftClient.program.programId,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        };
        const ixs = [];
        if (createAtaIx) {
            ixs.push(createAtaIx);
        }
        ixs.push(await this.program.methods
            .forceWithdraw()
            .accounts(accounts)
            .remainingAccounts(remainingAccounts)
            .instruction());
        return ixs;
    }
    async cancelRequestWithdraw(vaultDepositor, txParams) {
        const vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(vaultDepositor);
        const vaultAccount = await this.program.account.vault.fetch(vaultDepositorAccount.vault);
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vaultDepositorAccount.vault);
        const accounts = {
            vault: vaultDepositorAccount.vault,
            vaultDepositor,
            driftUserStats: userStatsKey,
            driftUser: vaultAccount.user,
        };
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vaultDepositorAccount.vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        if (this.cliMode) {
            return await this.program.methods
                .cancelRequestWithdraw()
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .rpc();
        }
        else {
            const oracleFeedsToCrankIxs = await this.getOracleFeedsToCrank(txParams === null || txParams === void 0 ? void 0 : txParams.oracleFeedsToCrank);
            const cancelRequestWithdrawIx = this.program.instruction.cancelRequestWithdraw({
                accounts: {
                    authority: this.driftClient.wallet.publicKey,
                    ...accounts,
                },
                remainingAccounts,
            });
            return await this.createAndSendTxn([...oracleFeedsToCrankIxs, cancelRequestWithdrawIx], txParams);
        }
    }
    /**
     * Liquidates (become delegate for) a vault.
     * @param
     * @param
     * @returns
     */
    async liquidate(vaultDepositor, txParams) {
        const vaultDepositorAccount = await this.program.account.vaultDepositor.fetch(vaultDepositor);
        const vaultPubKey = vaultDepositorAccount.vault;
        const vaultAccount = await this.program.account.vault.fetch(vaultPubKey);
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vaultPubKey);
        const driftStateKey = await this.driftClient.getStatePublicKey();
        const accounts = {
            vault: vaultPubKey,
            vaultDepositor,
            vaultTokenAccount: vaultAccount.tokenAccount,
            driftUserStats: userStatsKey,
            driftUser: vaultAccount.user,
            driftState: driftStateKey,
            driftProgram: this.driftClient.program.programId,
        };
        if (this.cliMode) {
            return await this.program.methods
                .liquidate()
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .rpc();
        }
        else {
            const liquidateIx = this.program.instruction.liquidate({
                accounts: {
                    authority: this.driftClient.wallet.publicKey,
                    ...accounts,
                },
                remainingAccounts,
            });
            return await this.createAndSendTxn([liquidateIx], txParams);
        }
    }
    async createTxn(vaultIxs, txParams) {
        var _a, _b, _c;
        const ixs = [
            web3_js_1.ComputeBudgetProgram.setComputeUnitLimit({
                units: (_a = txParams === null || txParams === void 0 ? void 0 : txParams.cuLimit) !== null && _a !== void 0 ? _a : 400000,
            }),
            web3_js_1.ComputeBudgetProgram.setComputeUnitPrice({
                microLamports: (_b = txParams === null || txParams === void 0 ? void 0 : txParams.cuPriceMicroLamports) !== null && _b !== void 0 ? _b : 1000000,
            }),
            ...vaultIxs,
        ];
        return (await this.driftClient.txHandler.buildTransaction({
            connection: this.driftClient.connection,
            instructions: ixs,
            lookupTables: (_c = txParams === null || txParams === void 0 ? void 0 : txParams.lookupTables) !== null && _c !== void 0 ? _c : [],
            preFlightCommitment: 'confirmed',
            forceVersionedTransaction: true,
            txVersion: 0,
            fetchMarketLookupTableAccount: this.driftClient.fetchMarketLookupTableAccount.bind(this.driftClient),
        }));
    }
    async sendTxn(transaction, simulateTransaction) {
        var _a;
        let txSig = bytes_1.bs58.encode(transaction.signatures[0]);
        if (simulateTransaction) {
            try {
                const resp = await this.driftClient.connection.simulateTransaction(transaction, {
                    sigVerify: false,
                    commitment: this.driftClient.connection.commitment,
                });
                console.log(`Simulated transaction:\n${JSON.stringify(resp, null, 2)}`);
            }
            catch (e) {
                const err = e;
                console.error(`Error simulating transaction: ${err.message}\n:${(_a = err.stack) !== null && _a !== void 0 ? _a : ''}`);
            }
        }
        else {
            const resp = await this.driftClient.sendTransaction(transaction, [], this.driftClient.opts);
            if (resp.txSig !== txSig) {
                console.error(`Transaction signature mismatch with self calculated value: ${resp.txSig} !== ${txSig}`);
                txSig = resp.txSig;
            }
        }
        return txSig;
    }
    /**
     * Used for UI wallet adapters compatibility
     */
    async createAndSendTxn(vaultIxs, txParams) {
        const tx = await this.createTxn(vaultIxs, txParams);
        const txSig = await this.sendTxn(tx, txParams === null || txParams === void 0 ? void 0 : txParams.simulateTransaction);
        return txSig;
    }
    /**
     * Initializes an insurance fund stake for the vault.
     * @param vault vault address to update
     * @param spotMarketIndex spot market index of the insurance fund stake
     * @returns
     */
    async initializeInsuranceFundStake(vault, spotMarketIndex) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const ifStakeAccountPublicKey = (0, sdk_1.getInsuranceFundStakeAccountPublicKey)(this.driftClient.program.programId, vault, spotMarketIndex);
        const spotMarket = this.driftClient.getSpotMarketAccount(spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${spotMarketIndex} not found on driftClient`);
        }
        const ifVaultTokenAccount = (0, addresses_1.getInsuranceFundTokenVaultAddressSync)(this.program.programId, vault, spotMarketIndex);
        return await this.program.methods
            .initializeInsuranceFundStake(spotMarketIndex)
            .accounts({
            vault: vault,
            driftSpotMarket: spotMarket.pubkey,
            driftSpotMarketMint: spotMarket.mint,
            vaultTokenAccount: ifVaultTokenAccount,
            insuranceFundStake: ifStakeAccountPublicKey,
            driftUserStats: vaultAccount.userStats,
            driftState: await this.driftClient.getStatePublicKey(),
            driftProgram: this.driftClient.program.programId,
        })
            .rpc();
    }
    /**
     * Adds an amount to an insurance fund stake for the vault.
     * @param vault vault address to update
     * @param spotMarketIndex spot market index of the insurance fund stake
     * @param amount amount to add to the insurance fund stake, in spotMarketIndex precision
     * @returns
     */
    async addToInsuranceFundStake(vault, spotMarketIndex, amount, managerTokenAccount) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        if (!vaultAccount.manager.equals(this.driftClient.wallet.publicKey)) {
            throw new Error(`Only the manager of the vault can add to the insurance fund stake.`);
        }
        const ifStakeAccountPublicKey = (0, sdk_1.getInsuranceFundStakeAccountPublicKey)(this.driftClient.program.programId, vault, spotMarketIndex);
        const ifVaultPublicKey = await (0, sdk_1.getInsuranceFundVaultPublicKey)(this.driftClient.program.programId, spotMarketIndex);
        const spotMarket = this.driftClient.getSpotMarketAccount(spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${spotMarketIndex} not found on driftClient`);
        }
        if (!managerTokenAccount) {
            managerTokenAccount = (0, spl_token_1.getAssociatedTokenAddressSync)(spotMarket.mint, this.driftClient.wallet.publicKey);
        }
        const ifVaultTokenAccount = (0, addresses_1.getInsuranceFundTokenVaultAddressSync)(this.program.programId, vault, spotMarketIndex);
        return await this.program.methods
            .addInsuranceFundStake(spotMarketIndex, amount)
            .accounts({
            vault: vault,
            driftSpotMarket: spotMarket.pubkey,
            driftSpotMarketVault: spotMarket.vault,
            insuranceFundStake: ifStakeAccountPublicKey,
            insuranceFundVault: ifVaultPublicKey,
            managerTokenAccount,
            vaultIfTokenAccount: ifVaultTokenAccount,
            driftUserStats: vaultAccount.userStats,
            driftState: await this.driftClient.getStatePublicKey(),
            driftProgram: this.driftClient.program.programId,
            driftSigner: this.driftClient.getStateAccount().signer,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        })
            .rpc();
    }
    async requestRemoveInsuranceFundStake(vault, spotMarketIndex, amount) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const ifStakeAccountPublicKey = (0, sdk_1.getInsuranceFundStakeAccountPublicKey)(this.driftClient.program.programId, vault, spotMarketIndex);
        const ifVaultPublicKey = await (0, sdk_1.getInsuranceFundVaultPublicKey)(this.driftClient.program.programId, spotMarketIndex);
        const spotMarket = this.driftClient.getSpotMarketAccount(spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${spotMarketIndex} not found on driftClient`);
        }
        return await this.program.methods
            .requestRemoveInsuranceFundStake(spotMarketIndex, amount)
            .accounts({
            vault,
            manager: this.driftClient.wallet.publicKey,
            driftSpotMarket: spotMarket.pubkey,
            insuranceFundStake: ifStakeAccountPublicKey,
            insuranceFundVault: ifVaultPublicKey,
            driftUserStats: vaultAccount.userStats,
            driftProgram: this.driftClient.program.programId,
        })
            .rpc();
    }
    async cancelRequestRemoveInsuranceFundStake(vault, spotMarketIndex) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const ifStakeAccountPublicKey = (0, sdk_1.getInsuranceFundStakeAccountPublicKey)(this.driftClient.program.programId, vault, spotMarketIndex);
        const ifVaultPublicKey = await (0, sdk_1.getInsuranceFundVaultPublicKey)(this.driftClient.program.programId, spotMarketIndex);
        const spotMarket = this.driftClient.getSpotMarketAccount(spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${spotMarketIndex} not found on driftClient`);
        }
        return await this.program.methods
            .cancelRequestRemoveInsuranceFundStake(spotMarketIndex)
            .accounts({
            vault: vault,
            manager: this.driftClient.wallet.publicKey,
            driftSpotMarket: spotMarket.pubkey,
            insuranceFundStake: ifStakeAccountPublicKey,
            insuranceFundVault: ifVaultPublicKey,
            driftUserStats: vaultAccount.userStats,
            driftProgram: this.driftClient.program.programId,
        })
            .rpc();
    }
    async removeInsuranceFundStake(vault, spotMarketIndex, managerTokenAccount) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const ifStakeAccountPublicKey = (0, sdk_1.getInsuranceFundStakeAccountPublicKey)(this.driftClient.program.programId, vault, spotMarketIndex);
        const ifVaultPublicKey = await (0, sdk_1.getInsuranceFundVaultPublicKey)(this.driftClient.program.programId, spotMarketIndex);
        const spotMarket = this.driftClient.getSpotMarketAccount(spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${spotMarketIndex} not found on driftClient`);
        }
        if (!managerTokenAccount) {
            managerTokenAccount = (0, spl_token_1.getAssociatedTokenAddressSync)(spotMarket.mint, this.driftClient.wallet.publicKey);
        }
        const ifVaultTokenAccount = (0, addresses_1.getInsuranceFundTokenVaultAddressSync)(this.program.programId, vault, spotMarketIndex);
        return await this.program.methods
            .removeInsuranceFundStake(spotMarketIndex)
            .accounts({
            vault: vault,
            driftSpotMarket: spotMarket.pubkey,
            insuranceFundStake: ifStakeAccountPublicKey,
            insuranceFundVault: ifVaultPublicKey,
            managerTokenAccount,
            vaultIfTokenAccount: ifVaultTokenAccount,
            driftState: await this.driftClient.getStatePublicKey(),
            driftUserStats: vaultAccount.userStats,
            driftSigner: this.driftClient.getStateAccount().signer,
            driftProgram: this.driftClient.program.programId,
            tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
        })
            .rpc();
    }
    async protocolRequestWithdraw(vault, amount, withdrawUnit) {
        // @ts-ignore
        const vaultAccount = (await this.program.account.vault.fetch(vault));
        const vp = this.getVaultProtocolAddress(vault);
        const vpAccount = (await this.program.account.vaultProtocol.fetch(vp));
        if (!this.driftClient.wallet.publicKey.equals(vpAccount.protocol)) {
            throw new Error(`Only the protocol of the vault can request a withdraw.`);
        }
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault);
        const accounts = {
            vault,
            driftUserStats: userStatsKey,
            driftUser: vaultAccount.user,
        };
        if (this.cliMode) {
            return await this.program.methods
                // @ts-ignore, 0.29.0 anchor issues..
                .managerRequestWithdraw(amount, withdrawUnit)
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .rpc();
        }
        else {
            const requestWithdrawIx = this.program.instruction.managerRequestWithdraw(
            // @ts-ignore
            amount, withdrawUnit, {
                accounts: {
                    manager: this.driftClient.wallet.publicKey,
                    ...accounts,
                },
                remainingAccounts,
            });
            return await this.createAndSendTxn([requestWithdrawIx]);
        }
    }
    async protocolCancelWithdrawRequest(vault) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        const userStatsKey = (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault);
        const accounts = {
            manager: this.driftClient.wallet.publicKey,
            vault,
            driftUserStats: userStatsKey,
            driftUser: vaultAccount.user,
        };
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        if (this.cliMode) {
            return await this.program.methods
                .mangerCancelWithdrawRequest()
                .accounts(accounts)
                .remainingAccounts(remainingAccounts)
                .rpc();
        }
        else {
            const cancelRequestWithdrawIx = this.program.instruction.mangerCancelWithdrawRequest({
                accounts: {
                    ...accounts,
                    manager: this.driftClient.wallet.publicKey,
                },
                remainingAccounts,
            });
            return await this.createAndSendTxn([cancelRequestWithdrawIx]);
        }
    }
    async protocolWithdraw(vault) {
        const vaultAccount = await this.program.account.vault.fetch(vault);
        if (!this.driftClient.wallet.publicKey.equals(vaultAccount.manager)) {
            throw new Error(`Only the manager of the vault can request a withdraw.`);
        }
        const user = await this.getSubscribedVaultUser(vaultAccount.user);
        const remainingAccounts = this.driftClient.getRemainingAccounts({
            userAccounts: [user.getUserAccount()],
            writableSpotMarketIndexes: [vaultAccount.spotMarketIndex],
        });
        if (vaultAccount.vaultProtocol) {
            const vaultProtocol = this.getVaultProtocolAddress(vault);
            remainingAccounts.push({
                pubkey: vaultProtocol,
                isSigner: false,
                isWritable: true,
            });
        }
        const spotMarket = this.driftClient.getSpotMarketAccount(vaultAccount.spotMarketIndex);
        if (!spotMarket) {
            throw new Error(`Spot market ${vaultAccount.spotMarketIndex} not found on driftClient`);
        }
        const ix = this.program.instruction.managerWithdraw({
            accounts: {
                vault,
                manager: this.driftClient.wallet.publicKey,
                vaultTokenAccount: vaultAccount.tokenAccount,
                driftUser: await (0, sdk_1.getUserAccountPublicKey)(this.driftClient.program.programId, vault),
                driftProgram: this.driftClient.program.programId,
                driftUserStats: (0, sdk_1.getUserStatsAccountPublicKey)(this.driftClient.program.programId, vault),
                driftState: await this.driftClient.getStatePublicKey(),
                driftSpotMarketVault: spotMarket.vault,
                userTokenAccount: (0, spl_token_1.getAssociatedTokenAddressSync)(spotMarket.mint, this.driftClient.wallet.publicKey),
                driftSigner: this.driftClient.getStateAccount().signer,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
            },
            remainingAccounts,
        });
        return this.createAndSendTxn([ix], {
            cuLimit: 1000000,
        });
    }
    async getOracleFeedsToCrank(oracleFeedsToCrank) {
        const oracleFeedsToCrankIxs = oracleFeedsToCrank
            ? (await Promise.all(oracleFeedsToCrank.map(async (feedConfig) => {
                if (JSON.stringify(feedConfig.oracleSource) !==
                    JSON.stringify(sdk_1.OracleSource.SWITCHBOARD_ON_DEMAND)) {
                    throw new Error('Only SWITCHBOARD_ON_DEMAND oracle feeds are supported for cranking');
                }
                return this.driftClient.getPostSwitchboardOnDemandUpdateAtomicIx(feedConfig.feed);
            })))
            : [];
        return oracleFeedsToCrankIxs;
    }
}
exports.VaultClient = VaultClient;
