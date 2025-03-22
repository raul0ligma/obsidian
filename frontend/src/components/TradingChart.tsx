import React from "react";

const TradingChart = () => {
  return (
    <div className="glass card-shadow w-full h-full border border-neutral-100">
      <div className="px-4 py-3 flex justify-between items-center border-b border-neutral-100">
        <h2 className="text-lg font-medium">Market Data</h2>
        <div className="text-xs">Powered by GeckoTerminal</div>
      </div>

      <div className="w-full h-[600px]">
        <iframe
          height="100%"
          width="100%"
          id="geckoterminal-embed"
          title="GeckoTerminal Embed"
          src="https://www.geckoterminal.com/base/pools/0x88a43bbdf9d098eec7bceda4e2494615dfd9bb9c?embed=1&info=1&swaps=0&grayscale=1&chart_type=price&resolution=15m"
          frameBorder="0"
          allow="clipboard-write"
          allowFullScreen
        ></iframe>
      </div>
    </div>
  );
};

export default TradingChart;
