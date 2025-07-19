/**
 * Solana Agent Kit Bridge for Rust FFI Integration
 * 
 * This module provides a TypeScript bridge between Rust and the Solana Agent Kit
 * for Metaplex Core operations in the Banshee emotional AI framework.
 */

import { SolanaAgentKit, KeypairWallet } from "solana-agent-kit";
import { Keypair } from "@solana/web3.js";
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
 * NFT deployment options for Metaplex Core
 */
interface NFTDeployOptions {
  name: string;
  uri: string;
  royaltyBasisPoints: number;
  creators: Array<{
    address: string;
    percentage: number;
  }>;
}

/**
 * Collection deployment options
 */
interface CollectionDeployOptions {
  name: string;
  uri: string;
  royaltyBasisPoints: number;
  creators: Array<{
    address: string;
    percentage: number;
  }>;
}

/**
 * Result type for deployment operations
 */
interface DeploymentResult {
  success: boolean;
  signature?: string;
  mint?: string;
  data?: any;
  error?: string;
}

/**
 * Bridge class for Solana Agent Kit operations
 */
class SolanaAgentBridge {
  private agent: SolanaAgentKit | null = null;

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
   * Deploy a token (placeholder - to be implemented with proper action system)
   */
  async deployToken(
    name: string,
    symbol: string,
    decimals: number = 9,
    initialSupply: number = 1000000
  ): Promise<DeploymentResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      // TODO: Implement token deployment using the action system
      const publicKey = this.agent.wallet.publicKey.toString();
      
      return {
        success: true,
        data: {
          message: "Token deployment not yet implemented in action system",
          wallet: publicKey,
          tokenParams: { name, symbol, decimals, initialSupply }
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
   * Get wallet balance
   */
  async getBalance(): Promise<DeploymentResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      // Use agent's wallet to get balance
      const balance = await this.agent.connection.getBalance(this.agent.wallet.publicKey);
      return {
        success: true,
        data: { balance: balance / 1e9 } // Convert lamports to SOL
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  /**
   * Get the current wallet's public key
   */
  getWalletAddress(): string | null {
    if (!this.agent) {
      return null;
    }
    // Access the wallet's public key through the agent
    return this.agent.wallet.publicKey.toString();
  }

  /**
   * Deploy a collection (placeholder - to be implemented with proper action system)
   */
  async deployCollection(options: any): Promise<DeploymentResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      // TODO: Implement collection deployment using the action system
      const publicKey = this.agent.wallet.publicKey.toString();
      
      return {
        success: true,
        data: {
          message: "Collection deployment not yet implemented in action system",
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
   * Get asset info (placeholder - to be implemented with proper action system)
   */
  async getAsset(assetId: string): Promise<DeploymentResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      // TODO: Implement asset info retrieval using the action system
      const publicKey = this.agent.wallet.publicKey.toString();
      
      return {
        success: true,
        data: {
          message: "Asset info retrieval not yet implemented in action system",
          wallet: publicKey,
          assetId: assetId
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
   * Check if agent is initialized
   */
  isInitialized(): boolean {
    return this.agent !== null;
  }
}

// Global bridge instance
let bridgeInstance: SolanaAgentBridge | null = null;

/**
 * Get or create the global bridge instance
 */
function getBridge(): SolanaAgentBridge {
  if (!bridgeInstance) {
    bridgeInstance = new SolanaAgentBridge();
  }
  return bridgeInstance;
}

// Export functions for FFI binding
export {
  SolanaAgentBridge,
  getBridge,
  type SolanaAgentConfig,
  type NFTDeployOptions,
  type CollectionDeployOptions,
  type DeploymentResult
};

// FFI-compatible functions
export async function initializeSolanaAgent(configJson: string): Promise<boolean> {
  const config: SolanaAgentConfig = JSON.parse(configJson);
  const bridge = getBridge();
  return await bridge.initialize(config);
}

export async function deployCollectionFFI(optionsJson: string): Promise<string> {
  const options: CollectionDeployOptions = JSON.parse(optionsJson);
  const bridge = getBridge();
  const result = await bridge.deployCollection(options);
  return JSON.stringify(result);
}

export async function getAssetFFI(assetId: string): Promise<string> {
  const bridge = getBridge();
  try {
    const result = await bridge.getAsset(assetId);
    return JSON.stringify({ success: true, data: result });
  } catch (error) {
    return JSON.stringify({
      success: false,
      error: error instanceof Error ? error.message : "Unknown error"
    });
  }
}

export function getWalletAddressFFI(): string {
  const bridge = getBridge();
  const address = bridge.getWalletAddress();
  return JSON.stringify({ success: true, address });
}

export function isInitializedFFI(): boolean {
  const bridge = getBridge();
  return bridge.isInitialized();
}