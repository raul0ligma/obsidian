import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter } from "react-router-dom";
import Index from "./pages/Index";
import { PrivyProvider } from "@privy-io/react-auth";
import { WagmiProvider } from "@privy-io/wagmi";
import { wagmiConfig } from "./config/wagmiConfig";

const queryClient = new QueryClient();

const App = () => (
  <PrivyProvider
    appId="clu6ow4s506j9i1lec46y3ah5"
    clientId="client-WY2jPSS99RmCJeUNc6NYkQH5wdxvvTLS2M97wW34S9fhR"
    config={{
      appearance: {
        loginMethods: ["wallet"],
        theme: "dark",
        accentColor: "#676FFF",
        logo: "https://your-logo-url",
        defaultChainId: 8453,
      },
    }}
  >
    <QueryClientProvider client={queryClient}>
      <WagmiProvider config={wagmiConfig}>
        <TooltipProvider>
          <Toaster />
          <Sonner />
          <BrowserRouter>
            <Index />
          </BrowserRouter>
        </TooltipProvider>
      </WagmiProvider>
    </QueryClientProvider>
  </PrivyProvider>
);

export default App;
