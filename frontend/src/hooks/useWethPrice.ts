import { useState, useEffect } from "react";

interface GeckoTerminalResponse {
  data: {
    attributes: {
      base_token_price_quote_token: string;
      quote_token_price_base_token: string;
    };
  };
}

export const useWethPrice = () => {
  const [exchangeRate, setExchangeRate] = useState<string>("0");
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  useEffect(() => {
    const fetchPrice = async () => {
      try {
        const response = await fetch(
          "https://api.geckoterminal.com/api/v2/networks/base/pools/0x88a43bbdf9d098eec7bceda4e2494615dfd9bb9c"
        );
        const data: GeckoTerminalResponse = await response.json();
        setExchangeRate(data.data.attributes.quote_token_price_base_token);
        setLastUpdated(new Date());
      } catch (error) {
        console.error("Failed to fetch WETH price:", error);
      }
    };

    // Fetch immediately
    fetchPrice();

    // Then fetch every minute
    const interval = setInterval(fetchPrice, 60000);

    return () => clearInterval(interval);
  }, []);

  return { exchangeRate, lastUpdated };
};
