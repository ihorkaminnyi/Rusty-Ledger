import { PortfolioSummary } from './portfolio.ts';
import type { RebalancePlan } from './rebalance.ts';

export interface AppState {
    isDataLoaded: boolean;
    isLoading: boolean;
    showGettingStarted: boolean;
    showAboutModal: boolean;
    portfolio: PortfolioSummary | null;
    rebalance: RebalancePlan | null;
    error: string | null;
    reportFilePath: string | null;
}
