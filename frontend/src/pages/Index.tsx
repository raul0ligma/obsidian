import { useEffect } from "react";
import { usePrivy } from "@privy-io/react-auth";
import TradeCard from "@/components/TradeCard";
import TradingChart from "@/components/TradingChart";
import { Button } from "@/components/ui/button";
import { LogOut } from "lucide-react";

const Index = () => {
  const { login, logout, ready, authenticated } = usePrivy();

  useEffect(() => {
    console.log("Obsidian loaded successfully! 🚀");
  }, []);

  return (
    <div className="min-h-screen flex flex-col relative bg-dark-bg retro-grid">
      <div className="bg-gradient-overlay" />

      <main className="flex-1 flex flex-col items-center p-4 sm:p-6 md:p-8 relative z-10">
        {authenticated && (
          <div className="absolute top-4 right-4 z-20">
            <Button
              onClick={logout}
              variant="outline"
              size="sm"
              className="flex items-center gap-2 text-[#ff5555] border-[#ff5555] hover:bg-[#ff5555]/10"
            >
              <LogOut className="h-4 w-4" />
              Disconnect
            </Button>
          </div>
        )}

        <div className="w-full max-w-[1920px] grid grid-cols-1 lg:grid-cols-12 gap-3 animate-fade-in pl-[160px]">
          <div className="lg:col-span-4 lg:-ml-8">
            {authenticated ? (
              <TradeCard />
            ) : (
              <div className="glass card-shadow w-full border border-neutral-100 p-4 flex flex-col items-center justify-center space-y-4">
                <h2 className="text-lg font-medium text-[#8be9fd]/90">
                  Connect Wallet to Trade
                </h2>
                <Button
                  onClick={login}
                  className="w-full h-12 font-medium transition-all animate-scale"
                  disabled={!ready}
                >
                  Connect Wallet
                </Button>
              </div>
            )}
          </div>

          <div className="lg:col-span-7 lg:ml-4">
            <TradingChart />
          </div>
        </div>
      </main>

      <img
        src="/pep.png"
        alt="Mascot"
        className="floating-character relative"
      />
    </div>
  );
};

export default Index;
