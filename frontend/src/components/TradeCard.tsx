import { useState, useEffect } from "react";
import { ArrowBigDown } from "lucide-react";
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
import { useToast } from "@/components/ui/use-toast";
import { cn } from "@/lib/utils";
import { useObsidianTrade } from "@/hooks/useObsidianTrade";
import { useWethPrice } from "@/hooks/useWethPrice";
import { Settings } from "lucide-react";

const USDC = {
  id: "usdc",
  symbol: "USDC",
  name: "USD Coin",
  logo: "https://cryptologos.cc/logos/usd-coin-usdc-logo.svg?v=024",
  balance: "1500.00",
};

const WETH = {
  id: "weth",
  symbol: "WETH",
  name: "Wrapped Ether",
  logo: "https://cryptologos.cc/logos/ethereum-eth-logo.svg?v=024",
  balance: "2300.00",
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
  const [slippageTolerance, setSlippageTolerance] = useState<number>(0.5);
  const [loading, setLoading] = useState<boolean>(false);
  const [settingsOpen, setSettingsOpen] = useState<boolean>(false);
  const [autoSlippage, setAutoSlippage] = useState<boolean>(true);
  const [activeStep, setActiveStep] = useState<number>(-1);
  const [txHash, setTxHash] = useState<string>("");
  const [progress, setProgress] = useState<number>(0);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [blockNumber, setBlockNumber] = useState<number | null>(null);
  const [orderServerUrl, setOrderServerUrl] = useState<string>(() => {
    return localStorage.getItem("orderServerUrl") || "http://127.0.0.1:8069";
  });

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

  const { ready, authenticated } = usePrivy();
  const { address } = useAccount();
  const chainId = useChainId();
  const { data: walletClient } = useWalletClient();

  const solverAddress = "0xd4f23AfEAcfc05399E58e122B9a23cD04FA02C3B";

  const { toast } = useToast();

  const { signOrder } = useObsidianTrade({
    authenticated,
    blockNumber,
    addLog,
    orderServerUrl,
  });

  const { exchangeRate, lastUpdated } = useWethPrice();

  useEffect(() => {
    if (chainId && chainId !== base.id) {
      walletClient?.switchChain({ chainId: base.id }).catch((error) => {
        console.error("Failed to switch network:", error);
      });
    }
  }, [chainId, walletClient]);

  useEffect(() => {
    if (
      fromAmount &&
      parseFloat(fromAmount) > 0 &&
      parseFloat(exchangeRate) > 0
    ) {
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

  useEffect(() => {
    localStorage.setItem("orderServerUrl", orderServerUrl);
  }, [orderServerUrl]);

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
    setLoading(true);
    setActiveStep(0);
    setLogs([]);

    try {
      const result = await signOrder(fromAmount);
      if (result) {
        // Continue with the trade flow using result.signature, result.typedData
        advanceTimeline(0);
      }
    } catch (error) {
      console.error("Trade error:", error);
    } finally {
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

  const renderServerSettings = () => {
    if (!settingsOpen) return null;

    return (
      <div className="space-y-4 p-4 border-t border-neutral-100">
        <div className="space-y-2">
          <label className="text-sm font-medium">Order Server URL</label>
          <div className="flex gap-2">
            <Input
              value={orderServerUrl}
              onChange={(e) => setOrderServerUrl(e.target.value)}
              placeholder="Enter order server URL"
              className="flex-1"
            />
            <Button
              variant="outline"
              size="sm"
              onClick={() => setOrderServerUrl("http://127.0.0.1:8069")}
            >
              Reset
            </Button>
          </div>
          <p className="text-xs text-neutral-500">
            Currently using: {orderServerUrl}
          </p>
        </div>
      </div>
    );
  };

  return (
    <div className="glass card-shadow w-full border border-neutral-100">
      <div className="px-4 py-3 flex justify-between items-center border-b border-neutral-100">
        <h2 className="text-lg font-medium">TRADE</h2>

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

        <Button
          variant="ghost"
          size="sm"
          onClick={() => setSettingsOpen(!settingsOpen)}
        >
          <Settings className="h-4 w-4" />
        </Button>
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
            <ArrowBigDown className="h-4 w-4" />
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
                  src={WETH.logo}
                  alt={WETH.symbol}
                  className="h-full w-full rounded-full object-contain"
                />
              </div>
              <span>{WETH.symbol}</span>
            </div>
          </div>
        </div>

        <div className="animate-fade-in">
          <div className="text-xs flex justify-between items-center p-1.5 text-[#8be9fd]/90">
            <div className="flex items-center gap-1">
              <span>1 USDC = {parseFloat(exchangeRate).toFixed(6)} WETH</span>
            </div>
            {lastUpdated && (
              <div className="text-white/50 text-[10px]">
                Updated: {lastUpdated.toLocaleTimeString()}
              </div>
            )}
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

        {/* Updated Proof Explorer Link with better styling */}
        <div className="flex justify-center">
          <a
            href={`https://testnet.succinct.xyz/explorer/requester/${
              import.meta.env.VITE_PROVER_KEY
            }`}
            target="_blank"
            rel="noopener noreferrer"
            className="group flex items-center gap-2 text-xs text-[#8be9fd]/70 hover:text-[#8be9fd] transition-colors py-2 px-4 rounded-md bg-black/20 border border-[#8be9fd]/10 hover:border-[#8be9fd]/30"
          >
            <span className="font-medium">Proof explorer:</span>
            <span className="text-white/50 group-hover:text-white/70 transition-colors">
              testnet.succinct.xyz/explorer
            </span>
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="relative top-[0.5px] ml-0.5 opacity-70 group-hover:opacity-100 transition-opacity"
            >
              <path d="M7 17L17 7" />
              <path d="M7 7h10v10" />
            </svg>
          </a>
        </div>
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

      {renderServerSettings()}

      <useObsidianTrade
        authenticated={authenticated}
        blockNumber={blockNumber}
        addLog={addLog}
        orderServerUrl={orderServerUrl}
      />
    </div>
  );
};

export default TradeCard;
