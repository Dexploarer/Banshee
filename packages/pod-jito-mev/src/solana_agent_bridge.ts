/**
 * Solana Agent Kit Bridge for Jito MEV Operations
 * 
 * This module provides a TypeScript bridge between Rust and the Solana Agent Kit
 * for Jito MEV and TipRouter operations in the Banshee emotional AI framework.
 */

import { SolanaAgentKit, KeypairWallet } from "solana-agent-kit";
import { PublicKey, Keypair } from "@solana/web3.js";
import bs58 from "bs58";

/**
 * Configuration for Solana Agent Kit initialization
 */
interface SolanaAgentConfig {
  privateKey: string;  // base58 encoded
  rpcUrl: string;
  openaiApiKey?: string;
}

/**
 * Staking options
 */
interface StakingOptions {
  amount: number; // Amount in SOL
  validatorVoteAccount?: string;
}

/**
 * Restaking options for Solayer
 */
interface RestakingOptions {
  amount: number; // Amount in SOL
}

/**
 * Trading options for perpetuals
 */
interface PerpTradingOptions {
  price: number;
  collateralAmount: number;
  collateralMint: string;
  leverage: number;
  tradeMint: string;
  slippage: number;
}

/**
 * Result type for operations
 */
interface OperationResult {
  success: boolean;
  signature?: string;
  data?: any;
  error?: string;
}

/**
 * Bridge class for Jito MEV operations
 */
class JitoMevBridge {
  private agent: SolanaAgentKit | null = null;
  private tools: any[] = [];

  /**
   * Initialize the Solana Agent Kit
   */
  async initialize(config: SolanaAgentConfig): Promise<boolean> {
    try {
      // Convert private key to Keypair
      let privateKeyBytes: Uint8Array;
      
      try {
        // Try to decode as base58 first (most common format)
        privateKeyBytes = bs58.decode(config.privateKey);
      } catch {
        try {
          // If base58 decode fails, try converting from base64
          privateKeyBytes = Uint8Array.from(Buffer.from(config.privateKey, 'base64'));
        } catch {
          throw new Error("Invalid private key format. Must be base58 or base64 encoded.");
        }
      }
      
      // Create Keypair from private key bytes
      const keypair = Keypair.fromSecretKey(privateKeyBytes);
      
      // Create KeypairWallet (implements BaseWallet interface)
      const wallet = new KeypairWallet(keypair, config.rpcUrl);
      
      // Initialize SolanaAgentKit with BaseWallet
      this.agent = new SolanaAgentKit(
        wallet,
        config.rpcUrl,
        config.openaiApiKey ? { OPENAI_API_KEY: config.openaiApiKey } : {}
      );
      
      return true;
    } catch (error) {
      console.error("Failed to initialize Solana Agent Kit:", error);
      return false;
    }
  }

  /**
   * Stake SOL on Solana (placeholder - to be implemented with proper action system)
   */
  async stakeSOL(options: StakingOptions): Promise<OperationResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      // TODO: Implement staking using the action system
      const publicKey = this.agent.wallet.publicKey.toString();
      
      return {
        success: true,
        data: {
          message: "SOL staking not yet implemented in action system",
          wallet: publicKey,
          options: options
        }
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  /**
   * Restake SOL on Solayer
   */
  async restakeSOL(options: RestakingOptions): Promise<OperationResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      const signature = await this.agent.restake(options.amount);
      return {
        success: true,
        signature: signature
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  /**
   * Open a perpetual long trade
   */
  async openPerpTradeLong(options: PerpTradingOptions): Promise<OperationResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      const signature = await this.agent.openPerpTradeLong({
        price: options.price,
        collateralAmount: options.collateralAmount,
        collateralMint: new PublicKey(options.collateralMint),
        leverage: options.leverage,
        tradeMint: new PublicKey(options.tradeMint),
        slippage: options.slippage,
      });
      
      return {
        success: true,
        signature: signature
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  /**
   * Close a perpetual long trade
   */
  async closePerpTradeLong(price: number, tradeMint: string): Promise<OperationResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      const signature = await this.agent.closePerpTradeLong({
        price: price,
        tradeMint: new PublicKey(tradeMint),
      });
      
      return {
        success: true,
        signature: signature
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  /**
   * Get Pyth price feed ID
   */
  async getPythPriceFeedID(symbol: string): Promise<OperationResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      const priceFeedID = await this.agent.getPythPriceFeedID(symbol);
      return {
        success: true,
        data: priceFeedID
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  /**
   * Get Pyth price
   */
  async getPythPrice(priceFeedID: string): Promise<OperationResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      const price = await this.agent.getPythPrice(priceFeedID);
      return {
        success: true,
        data: price
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  /**
   * Close empty token accounts to reclaim SOL rent
   */
  async closeEmptyTokenAccounts(): Promise<OperationResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      const result = await this.agent.closeEmptyTokenAccounts();
      return {
        success: true,
        signature: result.signature
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  /**
   * Check if agent is initialized
   */
  isInitialized(): boolean {
    return this.agent !== null;
  }
}

// Global bridge instance
let bridgeInstance: JitoMevBridge | null = null;

/**
 * Get or create the global bridge instance
 */
function getBridge(): JitoMevBridge {
  if (!bridgeInstance) {
    bridgeInstance = new JitoMevBridge();
  }
  return bridgeInstance;
}

// Export functions for FFI binding
export {
  JitoMevBridge,
  getBridge,
  type SolanaAgentConfig,
  type StakingOptions,
  type RestakingOptions,
  type PerpTradingOptions,
  type OperationResult
};

// FFI-compatible functions
export async function initializeSolanaAgent(configJson: string): Promise<boolean> {
  const config: SolanaAgentConfig = JSON.parse(configJson);
  const bridge = getBridge();
  return await bridge.initialize(config);
}

export async function stakeSOLFFI(optionsJson: string): Promise<string> {
  const options: StakingOptions = JSON.parse(optionsJson);
  const bridge = getBridge();
  const result = await bridge.stakeSOL(options);
  return JSON.stringify(result);
}

export async function restakeSOLFFI(optionsJson: string): Promise<string> {
  const options: RestakingOptions = JSON.parse(optionsJson);
  const bridge = getBridge();
  const result = await bridge.restakeSOL(options);
  return JSON.stringify(result);
}

export async function openPerpTradeLongFFI(optionsJson: string): Promise<string> {
  const options: PerpTradingOptions = JSON.parse(optionsJson);
  const bridge = getBridge();
  const result = await bridge.openPerpTradeLong(options);
  return JSON.stringify(result);
}

export async function closePerpTradeLongFFI(price: number, tradeMint: string): Promise<string> {
  const bridge = getBridge();
  const result = await bridge.closePerpTradeLong(price, tradeMint);
  return JSON.stringify(result);
}

export async function getPythPriceFFI(symbol: string): Promise<string> {
  const bridge = getBridge();
  const priceFeedResult = await bridge.getPythPriceFeedID(symbol);
  if (!priceFeedResult.success) {
    return JSON.stringify(priceFeedResult);
  }
  
  const priceResult = await bridge.getPythPrice(priceFeedResult.data);
  return JSON.stringify(priceResult);
}

export function isInitializedFFI(): boolean {
  const bridge = getBridge();
  return bridge.isInitialized();
}