export interface ObsidianTypedData {
  domain: {
    name: string;
    version: string;
    chainId: number;
    verifyingContract: string;
  };
  primaryType: string;
  types: {
    EIP712Domain: Array<{ name: string; type: string }>;
    ObsidianOrder: Array<{ name: string; type: string }>;
  };
  message: {
    blockNumber: bigint;
    sellAmount: bigint;
    buyToken: string;
    sellToken: string;
  };
}
