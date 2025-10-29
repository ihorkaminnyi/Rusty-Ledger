import { PortfolioSummary } from './portfolio.ts';
import type { RebalancePlan } from './rebalance.ts';
import type { AppError } from './errors.ts';

export interface AppState {
    isDataLoaded: boolean;
    isLoading: boolean;
    showGettingStarted: boolean;
    showAboutModal: boolean;
    portfolio: PortfolioSummary | null;
    rebalance: RebalancePlan | null;
    errors: AppError[];
    reportFilePath: string | null;
}
