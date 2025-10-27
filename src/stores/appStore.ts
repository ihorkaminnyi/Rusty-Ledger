import { defineStore } from 'pinia';
import type { AppState } from '../types/app.ts';
import type { PortfolioSummary } from '../types/portfolio.ts';
import type { RebalancePlan } from '../types/rebalance.ts';

export const useAppStore = defineStore('app', {
    state: (): AppState => ({
        isDataLoaded: false,
        isLoading: false,
        showGettingStarted: true,
        showAboutModal: false,
        portfolio: null,
        rebalance: null,
        error: null,
        reportFilePath: null,
    }),
    getters: {
        hasPortfolio: (state) => state.portfolio !== null,
    },
    actions: {
        setLoading(isLoading: boolean) {
            this.isLoading = isLoading;
        },
        setPortfolio(portfolio: PortfolioSummary) {
            this.portfolio = portfolio;
            this.isDataLoaded = true;
            this.showGettingStarted = false;
            this.error = null;
        },
        setRebalance(rebalance: RebalancePlan | null) {
            this.rebalance = rebalance;
            this.isDataLoaded = true;
            this.showGettingStarted = false;
            this.error = null;
        },
        clearPortfolio() {
            this.portfolio = null;
            this.isDataLoaded = false;
            this.reportFilePath = null;
        },
        setError(message: string | null) {
            this.error = message;
        },
        toggleAboutModal(show?: boolean) {
            this.showAboutModal = typeof show === 'boolean' ? show : !this.showAboutModal;
        },
        setReportFilePath(filePath: string | null) {
            this.reportFilePath = filePath;
        },
    },
});
