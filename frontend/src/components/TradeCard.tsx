import { useState, useEffect } from "react";
import { ArrowDownUp, Settings, Info } from "lucide-react";
import { usePrivy } from "@privy-io/react-auth";
import {
  useAccount,
  useChainId,
  useWalletClient,
  usePublicClient,
} from "wagmi";
import { createPublicClient, http, parseEther } from "viem";
import { base } from "viem/chains";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { useToast } from "@/components/ui/use-toast";
import { cn } from "@/lib/utils";

const USDC = {
  id: "usdc",
  symbol: "USDC",
  name: "USD Coin",
  logo: "https://cryptologos.cc/logos/usd-coin-usdc-logo.svg?v=024",
  balance: "1500.00",
};

const DAI = {
  id: "dai",
  symbol: "DAI",
  name: "Dai",
  logo: "https://cryptologos.cc/logos/multi-collateral-dai-dai-logo.svg?v=024",
  balance: "2300.00",
};

const getRandomRate = () => {
  return (0.995 + Math.random() * 0.01).toFixed(6);
};

interface LogEntry {
  timestamp: Date;
  message: string;
  type: "info" | "success" | "error" | "warning";
}

const publicClient = createPublicClient({
  chain: base,
  transport: http(),
});

const TradeCard = () => {
  const [fromAmount, setFromAmount] = useState<string>("");
  const [toAmount, setToAmount] = useState<string>("");

  const [exchangeRate, setExchangeRate] = useState<string>(getRandomRate());
  const [slippageTolerance, setSlippageTolerance] = useState<number>(0.5);

  const [loading, setLoading] = useState<boolean>(false);
  const [settingsOpen, setSettingsOpen] = useState<boolean>(false);
  const [autoSlippage, setAutoSlippage] = useState<boolean>(true);

  const [activeStep, setActiveStep] = useState<number>(-1);
  const [txHash, setTxHash] = useState<string>("");
  const [progress, setProgress] = useState<number>(0);

  const [logs, setLogs] = useState<LogEntry[]>([]);

  const [blockNumber, setBlockNumber] = useState<number | null>(null);
  const publicClient = usePublicClient();

  // Hardcoded solver for demo - in production this would come from your backend
  const solverAddress = "0x7F5Ac0abC2E4C1eD9561Ba8B0fC0bB3CF2Fd91F9";

  const { toast } = useToast();

  const { ready, authenticated } = usePrivy();
  const { address } = useAccount();
  const chainId = useChainId();
  const { data: walletClient } = useWalletClient();

  // Check if connected to Base
  useEffect(() => {
    if (chainId && chainId !== base.id) {
      addLog("Please switch to Base network", "warning");
      // You can use the wallet client to switch networks
      walletClient?.switchChain({ chainId: base.id }).catch((error) => {
        console.error("Failed to switch network:", error);
        addLog("Failed to switch to Base network", "error");
      });
    }
  }, [chainId, walletClient]);

  useEffect(() => {
    setExchangeRate(getRandomRate());
    addLog("System initialized", "info");
    addLog("Connecting to Obsidian network...", "info");
    setTimeout(() => {
      addLog("Connected to Obsidian network", "success");
    }, 1000);
  }, []);

  useEffect(() => {
    if (fromAmount && parseFloat(fromAmount) > 0) {
      const calculatedAmount = (
        parseFloat(fromAmount) * parseFloat(exchangeRate)
      ).toFixed(6);
      setToAmount(calculatedAmount);
    } else {
      setToAmount("");
    }
  }, [fromAmount, exchangeRate]);

  // Fetch latest block number every 5 seconds
  useEffect(() => {
    const fetchBlockNumber = async () => {
      try {
        const number = await publicClient.getBlockNumber();
        setBlockNumber(Number(number));
      } catch (error) {
        console.error("Failed to fetch block number:", error);
      }
    };

    // Fetch immediately
    fetchBlockNumber();

    // Then fetch every 5 seconds
    const interval = setInterval(fetchBlockNumber, 5000);

    return () => clearInterval(interval);
  }, [publicClient]);

  // Add new effect to handle chain switching after connection
  useEffect(() => {
    const switchToBase = async () => {
      if (authenticated && walletClient && chainId !== base.id) {
        try {
          addLog("Switching to Base network...", "info");
          await walletClient.switchChain({ chainId: base.id });
          addLog("Successfully switched to Base network", "success");
        } catch (error: any) {
          // Check if the chain hasn't been added to the wallet
          if (error.code === 4902) {
            // Chain not added error code
            try {
              await walletClient.addChain({
                chain: base,
              });
              await walletClient.switchChain({ chainId: base.id });
              addLog(
                "Successfully added and switched to Base network",
                "success"
              );
            } catch (addError) {
              console.error("Failed to add Base network:", addError);
              addLog("Failed to add Base network", "error");
            }
          } else {
            console.error("Failed to switch network:", error);
            addLog("Failed to switch to Base network", "error");
            toast({
              title: "Network Switch Required",
              description: "Please switch to Base network to continue",
              variant: "destructive",
            });
          }
        }
      }
    };

    switchToBase();
  }, [authenticated, walletClient, chainId]);

  const handleFromAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    if (value === "" || /^[0-9]*[.,]?[0-9]*$/.test(value)) {
      setFromAmount(value);
    }
  };

  const handleToAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    if (value === "" || /^[0-9]*[.,]?[0-9]*$/.test(value)) {
      setToAmount(value);

      if (value && parseFloat(value) > 0 && parseFloat(exchangeRate) > 0) {
        const calculatedAmount = (
          parseFloat(value) / parseFloat(exchangeRate)
        ).toFixed(6);
        setFromAmount(calculatedAmount);
      } else {
        setFromAmount("");
      }
    }
  };

  const addLog = (message: string, type: LogEntry["type"] = "info") => {
    setLogs((prev) => [
      ...prev,
      {
        timestamp: new Date(),
        message,
        type,
      },
    ]);

    // Auto-scroll to latest message
    requestAnimationFrame(() => {
      const logContainer = document.querySelector(".overflow-auto");
      if (logContainer) {
        logContainer.scrollTop = logContainer.scrollHeight;
      }
    });
  };

  const advanceTimeline = (stepIndex: number) => {
    setActiveStep(stepIndex);
    setProgress(Math.round(((stepIndex + 1) / 7) * 100));

    const stepMessages = [
      "Placing order on the Obsidian network",
      "Matching with available solvers in the network",
      "Order matched with solver " +
        solverAddress.substring(0, 6) +
        "..." +
        solverAddress.substring(solverAddress.length - 4),
      "Generating zero-knowledge proof for the transaction",
      "Proof successfully generated and verified",
      "Relaying transaction to the blockchain",
      "Transaction successfully executed on-chain",
    ];

    if (stepIndex < stepMessages.length) {
      const type = stepIndex === 6 ? "success" : "info";
      addLog(stepMessages[stepIndex], type);
      console.log(`Step ${stepIndex}: ${stepMessages[stepIndex]}`);
    }
  };

  const handleTrade = async () => {
    if (!authenticated || !address || !walletClient) {
      toast({
        title: "Connect wallet",
        description: "Please connect your wallet to trade.",
        variant: "destructive",
      });
      return;
    }

    // Check chain ID first
    if (chainId !== base.id) {
      try {
        await walletClient.switchChain({ chainId: base.id });
      } catch (error) {
        toast({
          title: "Network Switch Required",
          description: "Please switch to Base network to continue",
          variant: "destructive",
        });
        return;
      }
    }

    if (!fromAmount || parseFloat(fromAmount) <= 0) {
      toast({
        title: "Enter amount",
        description: "Please enter a valid amount to trade.",
        variant: "destructive",
      });
      return;
    }

    setLoading(true);
    setActiveStep(0);
    setLogs([]);

    try {
      // Send 1 wei transaction as test
      const randomAddress = `0x${Array(40)
        .fill(0)
        .map(() => Math.floor(Math.random() * 16).toString(16))
        .join("")}`;

      const hash = await walletClient.sendTransaction({
        to: randomAddress,
        value: parseEther("0.000000000000000001"), // 1 wei
      });

      addLog("Starting new private trade", "info");
      addLog(`Transaction hash: ${hash}`, "info");

      // Continue with existing trade flow
      advanceTimeline(0);
      // ... keep existing timeline code ...
    } catch (error) {
      addLog("Transaction failed", "error");
      console.error("Trade error:", error);
      setLoading(false);
    }
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString("en-US", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  return (
    <div className="glass card-shadow w-full border border-neutral-100">
      <div className="px-4 py-3 flex justify-between items-center border-b border-neutral-100">
        <h2 className="text-lg font-medium">Swap</h2>

        <div className="flex items-center gap-4">
          {/* Block Number */}
          <div className="flex items-center text-xs">
            <div
              className={`h-2 w-2 rounded-full ${
                blockNumber ? "bg-green-500" : "bg-yellow-500"
              } mr-1.5 animate-pulse`}
            ></div>
            <span className="text-[#8be9fd]/90">
              Block: {blockNumber?.toLocaleString() || "Loading..."}
            </span>
          </div>

          {/* Network Status with Warning */}
          <div className="flex items-center text-xs">
            <div
              className={`h-2 w-2 rounded-full ${
                chainId === base.id ? "bg-green-500" : "bg-red-500"
              } mr-1.5`}
            ></div>
            <span
              className={
                chainId === base.id ? "text-[#50fa7b]/90" : "text-[#ff5555]/90"
              }
            >
              {chainId === base.id ? "Base" : "Switch to Base"}
            </span>
          </div>
        </div>
      </div>

      {/* Show warning banner if wrong network */}
      {chainId !== base.id && authenticated && (
        <div className="px-4 py-2 bg-[#ff5555]/10 border-b border-[#ff5555]/20">
          <div className="flex items-center justify-between text-xs text-[#ff5555]">
            <span>Wrong network detected</span>
            <Button
              variant="outline"
              size="sm"
              className="h-7 text-xs border-[#ff5555] text-[#ff5555] hover:bg-[#ff5555]/10"
              onClick={() => walletClient?.switchChain({ chainId: base.id })}
            >
              Switch to Base
            </Button>
          </div>
        </div>
      )}

      {/* Add Solver Info below the header */}
      <div className="px-4 py-2 border-b border-neutral-100 bg-black/20">
        <div className="flex items-center justify-between text-xs">
          <span className="text-[#8be9fd]/70">Active Solver:</span>
          <div className="flex items-center gap-2">
            <div className="h-2 w-2 rounded-full bg-green-500"></div>
            <span className="font-mono text-[#50fa7b]/90">
              {solverAddress.substring(0, 6)}...
              {solverAddress.substring(solverAddress.length - 4)}
            </span>
          </div>
        </div>
      </div>

      <div className="p-4 space-y-4">
        <div className="space-y-2">
          <div className="flex justify-between items-center text-sm">
            <span className="text-white">From</span>
          </div>

          <div className="flex items-center space-x-2 bg-black/40 rounded-lg p-3 focus-within:ring-1 focus-within:ring-primary/50 transition-all">
            <Input
              value={fromAmount}
              onChange={handleFromAmountChange}
              placeholder="0.0"
              className="border-0 bg-transparent text-xl focus-visible:ring-0 focus-visible:ring-offset-0 p-0 h-auto shadow-none text-white"
            />
            <div className="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 font-medium token-selector text-white">
              <div className="relative h-6 w-6 flex-shrink-0">
                <img
                  src={USDC.logo}
                  alt={USDC.symbol}
                  className="h-full w-full rounded-full object-contain"
                />
              </div>
              <span>{USDC.symbol}</span>
            </div>
          </div>
        </div>

        <div className="flex justify-center -my-0.5 relative z-10">
          <Button
            size="icon"
            variant="outline"
            className="rounded-full h-9 w-9 shadow-sm bg-black hover:bg-black/90 text-white"
            disabled={true}
          >
            <ArrowDownUp className="h-4 w-4" />
          </Button>
        </div>

        <div className="space-y-2">
          <div className="flex justify-between items-center text-sm">
            <span className="text-white">To</span>
          </div>

          <div className="flex items-center space-x-2 bg-black/40 rounded-lg p-3 focus-within:ring-1 focus-within:ring-primary/50 transition-all">
            <Input
              value={toAmount}
              onChange={handleToAmountChange}
              placeholder="0.0"
              className="border-0 bg-transparent text-xl focus-visible:ring-0 focus-visible:ring-offset-0 p-0 h-auto shadow-none text-white"
            />
            <div className="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 font-medium token-selector text-white">
              <div className="relative h-6 w-6 flex-shrink-0">
                <img
                  src={DAI.logo}
                  alt={DAI.symbol}
                  className="h-full w-full rounded-full object-contain"
                />
              </div>
              <span>{DAI.symbol}</span>
            </div>
          </div>
        </div>

        <div className="animate-fade-in">
          <div className="text-xs flex justify-between items-center p-1.5 text-[#8be9fd]/90">
            <div className="flex items-center gap-1">
              <span>1 USDC = {parseFloat(exchangeRate).toFixed(6)} DAI</span>
            </div>
          </div>
        </div>

        <Button
          className="w-full h-12 font-medium transition-all animate-scale"
          size="lg"
          disabled={loading || !fromAmount || parseFloat(fromAmount) <= 0}
          onClick={handleTrade}
        >
          {loading ? (
            <div className="flex items-center gap-2">
              <div className="h-4 w-4 border-2 border-[#ff00ff] border-t-transparent rounded-none animate-spin" />
              <span>Processing...</span>
            </div>
          ) : (
            "Trade"
          )}
        </Button>
      </div>

      {activeStep >= 0 && (
        <div className="border-t border-[rgba(255,0,255,0.2)] mt-4">
          <div className="p-4">
            <div className="h-[200px] bg-black/40 overflow-auto p-4 font-mono text-xs">
              {logs.map((log, index) => (
                <div
                  key={index}
                  className={cn("mb-2", {
                    "text-[#50fa7b]/90": log.type === "success",
                    "text-[#ff5555]/90": log.type === "error",
                    "text-[#8be9fd]/90": log.type === "info",
                    "text-[#f1fa8c]/90": log.type === "warning",
                  })}
                >
                  <span className="text-white/40 mr-2">
                    {formatTime(log.timestamp)}
                  </span>
                  {log.message}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default TradeCard;
