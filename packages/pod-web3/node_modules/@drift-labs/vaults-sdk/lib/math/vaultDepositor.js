"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.calculateApplyProfitShare = calculateApplyProfitShare;
exports.calculateProfitShare = calculateProfitShare;
exports.calculateRealizedVaultDepositorEquity = calculateRealizedVaultDepositorEquity;
const sdk_1 = require("@drift-labs/sdk");
/**
 * Calculates the unrealized profitShare for a vaultDepositor
 * @param vaultDepositor
 * @param vaultEquity
 * @param vault
 * @returns
 */
function calculateApplyProfitShare(vaultDepositor, vaultEquity, vault) {
    const amount = (0, sdk_1.unstakeSharesToAmount)(vaultDepositor.vaultShares, vault.totalShares, vaultEquity);
    const profitShareAmount = calculateProfitShare(vaultDepositor, amount, vault);
    const profitShareShares = (0, sdk_1.stakeAmountToShares)(profitShareAmount, vault.totalShares, vaultEquity);
    return {
        profitShareAmount,
        profitShareShares,
    };
}
function calculateProfitShare(vaultDepositor, totalAmount, vault, vaultProtocol) {
    const profit = totalAmount.sub(vaultDepositor.netDeposits.add(vaultDepositor.cumulativeProfitShareAmount));
    let profitShare = vault.profitShare;
    if (vaultProtocol) {
        profitShare += vaultProtocol.protocolProfitShare;
    }
    if (profit.gt(sdk_1.ZERO)) {
        const profitShareAmount = profit
            .mul(new sdk_1.BN(profitShare))
            .div(sdk_1.PERCENTAGE_PRECISION);
        return profitShareAmount;
    }
    return sdk_1.ZERO;
}
/**
 * Calculates the equity across deposits and realized profit for a vaultDepositor
 * @param vaultDepositor vault depositor account
 * @param vaultEquity total vault equity
 * @param vault vault account
 * @param vaultProtocol if vault account has "vaultProtocol" then this is needed
 * @returns
 */
function calculateRealizedVaultDepositorEquity(vaultDepositor, vaultEquity, vault, vaultProtocol) {
    const vdAmount = (0, sdk_1.unstakeSharesToAmount)(vaultDepositor.vaultShares, vault.totalShares, vaultEquity);
    const profitShareAmount = calculateProfitShare(vaultDepositor, vdAmount, vault, vaultProtocol);
    return vdAmount.sub(profitShareAmount);
}
