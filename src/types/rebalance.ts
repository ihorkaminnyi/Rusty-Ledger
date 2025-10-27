export type TradeAction = 'Buy' | 'Sell';

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
