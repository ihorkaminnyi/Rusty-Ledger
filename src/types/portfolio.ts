export interface PositionSummary {
    symbol: string;
    quantity: number;
    price: number;
    marketValue: number;
    allocationPercent: number;
    unrealizedPl: number;
    roiPercent: number;
}

export interface PortfolioTotals {
    marketValue: number;
    costBasis: number;
    unrealizedPl: number;
    unrealizedPlPercent: number;
    totalPositions: number;
}

export interface AccountInfo {
    accountCapabilities?: string;
    accountType?: string;
    baseCurrency?: string;
    customerType?: string;
    name?: string;
}

export interface StatementInfo {
    title?: string;
    brokerName?: string;
    brokerAddress?: string;
    period?: string;
    whenGenerated?: string;
}

export interface PortfolioSummary {
    statement: StatementInfo;
    accountInfo: AccountInfo;
    totals: PortfolioTotals;
    positions: PositionSummary[];
}
