/**
 * Simple Web3 Bridge
 * Uses only verified APIs that exist in the actual package
 */

import { SolanaAgentKit, KeypairWallet } from "solana-agent-kit";
import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";

interface SolanaConfig {
  privateKey: string;
  rpcUrl: string;
  openaiApiKey?: string;
}

interface OperationResult {
  success: boolean;
  data?: any;
  error?: string;
}

class SimpleWeb3Bridge {
  private agent: SolanaAgentKit | null = null;

  async initialize(config: SolanaConfig): Promise<boolean> {
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
      console.error("Failed to initialize:", error);
      return false;
    }
  }

  async getWalletInfo(): Promise<OperationResult> {
    if (!this.agent) {
      return { success: false, error: "Agent not initialized" };
    }

    try {
      const publicKey = this.agent.wallet.publicKey.toString();
      const balance = await this.agent.connection.getBalance(this.agent.wallet.publicKey);
      
      return {
        success: true,
        data: {
          publicKey,
          balance: balance / 1e9 // Convert to SOL
        }
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : "Unknown error"
      };
    }
  }

  isInitialized(): boolean {
    return this.agent !== null;
  }
}

// Global instance
let bridge: SimpleWeb3Bridge | null = null;

function getBridge(): SimpleWeb3Bridge {
  if (!bridge) {
    bridge = new SimpleWeb3Bridge();
  }
  return bridge;
}

// FFI exports
export async function initializeAgent(configJson: string): Promise<boolean> {
  const config = JSON.parse(configJson);
  return await getBridge().initialize(config);
}

export async function getWalletInfo(): Promise<string> {
  const result = await getBridge().getWalletInfo();
  return JSON.stringify(result);
}

export function isAgentInitialized(): boolean {
  return getBridge().isInitialized();
}