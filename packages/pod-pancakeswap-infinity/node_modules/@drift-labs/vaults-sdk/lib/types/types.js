"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.VaultDepositorAction = exports.WithdrawUnit = exports.VAULT_PROGRAM_ID = void 0;
const web3_js_1 = require("@solana/web3.js");
exports.VAULT_PROGRAM_ID = new web3_js_1.PublicKey('vAuLTsyrvSfZRuRB3XgvkPwNGgYSs9YRYymVebLKoxR');
class WithdrawUnit {
}
exports.WithdrawUnit = WithdrawUnit;
WithdrawUnit.SHARES = { shares: {} };
WithdrawUnit.TOKEN = { token: {} };
WithdrawUnit.SHARES_PERCENT = { sharesPercent: {} };
// Logs/Records
class VaultDepositorAction {
}
exports.VaultDepositorAction = VaultDepositorAction;
VaultDepositorAction.DEPOSIT = { deposit: {} };
VaultDepositorAction.WITHDRAW = { withdraw: {} };
VaultDepositorAction.WITHDRAW_REQUEST = { withdrawRequest: {} };
VaultDepositorAction.CANCEL_WITHDRAW_REQUEST = { cancelWithdrawRequest: {} };
VaultDepositorAction.FEE_PAYMENT = { feePayment: {} };
