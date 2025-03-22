import {
  useAccount,
  useChainId,
  useWalletClient,
  usePublicClient,
} from "wagmi";
import { base } from "viem/chains";
import { useToast } from "@/components/ui/use-toast";
import { TypedDataDefinition } from "viem";
import axios from "axios";
import { encodeAbiParameters, encodeFunctionData } from "viem";
import { Relayer } from "@/services/relayer";
const USDC_ADDRESS = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913" as const;
const WETH_ADDRESS = "0x4200000000000000000000000000000000000006" as const;
const OBSIDIAN_ROUTER_ADDRESS =
  "0xEe395f9489bdC1b74A1BFc3B3164a5FEFb7146AE" as const;
const POOL_ADDRESS = "0x88a43bbdf9d098eec7bceda4e2494615dfd9bb9c" as const;

// Add the router ABI for the solve function
const ROUTER_ABI = [
  {
    name: "solve",
    type: "function",
    stateMutability: "nonpayable",
    inputs: [
      { name: "_publicValues", type: "bytes" },
      { name: "_proofBytes", type: "bytes" },
      { name: "_orderSignature", type: "bytes" },
    ],
    outputs: [],
  },
] as const;

const obsidianOrderTypedData = (
  chainId: number,
  verifyingContractAddress: `0x${string}`,
  blockNumber: bigint,
  sellAmount: bigint,
  buyToken: `0x${string}`,
  sellToken: `0x${string}`
): TypedDataDefinition => {
  return {
    domain: {
      name: "ObsidianRouter",
      version: "1",
      verifyingContract: verifyingContractAddress,
      chainId,
    },
    primaryType: "ObsidianOrder",
    types: {
      EIP712Domain: [
        { name: "name", type: "string" },
        { name: "version", type: "string" },
        { name: "chainId", type: "uint256" },
        { name: "verifyingContract", type: "address" },
      ],
      ObsidianOrder: [
        { name: "blockNumber", type: "uint256" },
        { name: "sellAmount", type: "uint256" },
        { name: "buyToken", type: "address" },
        { name: "sellToken", type: "address" },
      ],
    },
    message: {
      blockNumber,
      sellAmount,
      buyToken,
      sellToken,
    },
  };
};

interface UseObsidianTradeProps {
  authenticated: boolean;
  blockNumber: number | null;
  addLog: (
    message: string,
    type: "info" | "success" | "error" | "warning"
  ) => void;
  orderServerUrl?: string;
}

interface ObsidianOrderResponse {
  block: number;
  proof: string;
  public_values: string;
  error: string | null;
}

const BASESCAN_URL = "https://basescan.org/tx";

const DEFAULT_ORDER_SERVER = "http://127.0.0.1:8069" as const;

export const useObsidianTrade = ({
  authenticated,
  blockNumber,
  addLog,
  orderServerUrl = DEFAULT_ORDER_SERVER,
}: UseObsidianTradeProps) => {
  const { address } = useAccount();
  const chainId = useChainId();
  const { data: walletClient } = useWalletClient();
  const { toast } = useToast();

  const signOrder = async (amount: string) => {
    if (!authenticated || !address || !walletClient) {
      toast({
        title: "Connect wallet",
        description: "Please connect your wallet to trade.",
        variant: "destructive",
      });
      return null;
    }

    if (chainId !== base.id) {
      toast({
        title: "Network Switch Required",
        description: "Please switch to Base network to continue",
        variant: "destructive",
      });
      return null;
    }

    if (!amount || parseFloat(amount) <= 0) {
      toast({
        title: "Enter amount",
        description: "Please enter a valid amount to trade.",
        variant: "destructive",
      });
      return null;
    }

    if (!blockNumber) {
      toast({
        title: "Error",
        description: "Waiting for block number...",
        variant: "destructive",
      });
      return null;
    }

    try {
      addLog("waiting for order signature", "info");

      const sellAmountBigInt = BigInt(
        Math.floor(parseFloat(amount) * 1_000_000)
      );

      const typedData = obsidianOrderTypedData(
        base.id,
        OBSIDIAN_ROUTER_ADDRESS,
        BigInt(blockNumber),
        sellAmountBigInt,
        WETH_ADDRESS,
        USDC_ADDRESS
      );

      const signature = await walletClient.signTypedData({
        account: address,
        ...typedData,
      });

      addLog("signature found", "success");
      addLog(
        `signature: ${signature.slice(0, 10)}...${signature.slice(-8)}`,
        "info"
      );

      addLog("order submitted", "info");
      const orderPayload = {
        chain_id: base.id,
        pool_address: POOL_ADDRESS,
        sell_token: USDC_ADDRESS,
        buy_token: WETH_ADDRESS,
        address: address,
        amount: sellAmountBigInt.toString(),
        swap_venue: "uniswap",
        commit_block: blockNumber,
        signature,
      };

      addLog("waiting for solver to accept order", "info");
      addLog("solver accepted order", "success");
      addLog("waiting to generate trade proof", "info");

      const response = await fetch(`${orderServerUrl}/v1/order`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(orderPayload),
      });

      if (!response.ok) {
        throw new Error("Failed to submit order");
      }

      let elapsedTime = 0;
      const interval = setInterval(() => {
        elapsedTime += 30;
        const minutes = Math.floor(elapsedTime / 60);
        const seconds = elapsedTime % 60;
        addLog(`generating proof... (${minutes}m ${seconds}s)`, "info");
      }, 30000);

      const responseData: ObsidianOrderResponse = await response.json();

      clearInterval(interval);

      if (responseData.error) {
        throw new Error(responseData.error);
      }

      addLog("proof received", "success");
      addLog(`block: ${responseData.block}`, "info");
      addLog(
        `proof: ${responseData.proof.slice(0, 10)}...${responseData.proof.slice(
          -8
        )}`,
        "info"
      );
      addLog(
        `public values: ${responseData.public_values.slice(
          0,
          10
        )}...${responseData.public_values.slice(-8)}`,
        "info"
      );

      const calldata = encodeFunctionData({
        abi: ROUTER_ABI,
        functionName: "solve",
        args: [
          responseData.public_values as `0x${string}`,
          responseData.proof as `0x${string}`,
          signature as `0x${string}`,
        ],
      });

      addLog("preparing solve transaction", "info");
      addLog(`public values: ${responseData.public_values}`, "info");
      addLog(`proof: ${responseData.proof}`, "info");
      addLog(`signature: ${signature}`, "info");
      addLog(`encoded calldata: ${calldata}`, "info");

      addLog("submitting solve transaction through relayer", "info");

      const relayer = Relayer.getInstance();
      addLog(`relayer address: ${relayer.getAddress()}`, "info");

      const tx = await relayer.sendTransaction({
        to: OBSIDIAN_ROUTER_ADDRESS,
        data: calldata,
        value: 0n,
      });

      addLog(`solve transaction submitted: ${tx.hash}`, "success");

      // Add toast notification with Basescan link as text
      toast({
        title: "Transaction Submitted",
        description: `Transaction has been submitted. View on Basescan: ${BASESCAN_URL}/${tx.hash}`,
        duration: 10000, // Show for 10 seconds
      });

      // Also add the Basescan link to the logs
      addLog(`view on basescan: ${BASESCAN_URL}/${tx.hash}`, "info");

      return {
        signature,
        typedData,
        sellAmountBigInt,
        orderResponse: responseData,
        solveTransaction: tx,
        calldata,
      };
    } catch (error) {
      addLog("solve transaction failed", "error");
      console.error("Trade error:", error);
      return null;
    }
  };

  return {
    signOrder,
  };
};
