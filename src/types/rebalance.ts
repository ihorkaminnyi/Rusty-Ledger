export type TradeAction = "Buy" | "Sell";
export type RebalanceStrategy = "Buy Only" | "Full Rebalance";

export const REBALANCE_STRATEGIES = ["Buy Only", "Full Rebalance"] as const;
export const DEFAULT_REBALANCE_STRATEGY: RebalanceStrategy = "Buy Only";

export interface TargetAllocation {
  symbol: string;
  targetPercent: number;
}

export interface TradeInstruction {
  symbol: string;
  action: TradeAction;
  valueDelta: number;
  quantityDelta: number | null;
  priceUsed: number | null;
}

export interface RebalancePlan {
  totalValue: number;
  trades: TradeInstruction[];
}
