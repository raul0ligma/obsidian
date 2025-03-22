import { createWalletClient, http, parseEther, createPublicClient } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { base } from "viem/chains";

if (!import.meta.env.VITE_RELAYER_PRIVATE_KEY) {
  throw new Error("RELAYER_PRIVATE_KEY environment variable is not set");
}

const account = privateKeyToAccount(
  import.meta.env.VITE_RELAYER_PRIVATE_KEY as `0x${string}`
);

const publicClient = createPublicClient({
  chain: base,
  transport: http(),
});

const walletClient = createWalletClient({
  account,
  chain: base,
  transport: http(),
});

interface RelayerTransactionParams {
  to: `0x${string}`;
  data: `0x${string}`;
  value?: bigint;
}

export class Relayer {
  private static instance: Relayer;
  private nonce: number = 0;

  private constructor() {
    // Initialize nonce
    this.updateNonce();
  }

  public static getInstance(): Relayer {
    if (!Relayer.instance) {
      Relayer.instance = new Relayer();
    }
    return Relayer.instance;
  }

  private async updateNonce() {
    this.nonce = Number(
      await publicClient.getTransactionCount({
        address: account.address,
      })
    );
  }

  public async sendTransaction({
    to,
    data,
    value = 0n,
  }: RelayerTransactionParams) {
    try {
      // Get latest gas price
      const gasPrice = await publicClient.getGasPrice();

      // Estimate gas
      const gasEstimate = await publicClient.estimateGas({
        account: account.address,
        to,
        data,
        value,
      });

      // Send transaction
      const hash = await walletClient.sendTransaction({
        to,
        data,
        value,
        gasPrice,
        gas: gasEstimate,
        nonce: this.nonce++,
      });

      // Wait for transaction
      const receipt = await publicClient.waitForTransactionReceipt({ hash });

      return {
        hash,
        receipt,
        success: receipt.status === "success",
      };
    } catch (error) {
      console.error("Relayer transaction failed:", error);
      // Reset nonce on failure
      await this.updateNonce();
      throw error;
    }
  }

  public getAddress(): `0x${string}` {
    return account.address;
  }

  public async getBalance(): Promise<bigint> {
    return await publicClient.getBalance({
      address: account.address,
    });
  }
}

// Usage example:
// const relayer = Relayer.getInstance();
// await relayer.sendTransaction({
//   to: "0x...",
//   data: "0x...",
//   value: parseEther("0.1")
// });
