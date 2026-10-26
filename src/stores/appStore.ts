import { defineStore } from 'pinia';
import type { AppState } from '../types/app.ts';
import type { StoredPortfolioSummary } from '../types/portfolio.ts';
import type { RebalancePlan } from '../types/rebalance.ts';
import type { AppError, AppErrorInput, AppErrorScope } from '../types/errors.ts';

const createErrorId = () => {
    if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
        return crypto.randomUUID();
    }
    return `err_${Date.now()}_${Math.random().toString(16).slice(2)}`;
};

export const useAppStore = defineStore('app', {
    state: (): AppState => ({
        isDataLoaded: false,
        isLoading: false,
        showGettingStarted: true,
        showAboutModal: false,
        portfolio: null,
        rebalance: null,
        errors: [],
        reportFilePath: null,
        activeReportId: null,
        portfolioRevision: 0,
        isSavedReportsOpen: false,
    }),
    getters: {
        canImport: (state) => !state.isLoading && !state.isSavedReportsOpen,
        hasPortfolio: (state) => state.portfolio !== null,
        latestError: (state): AppError | null =>
            state.errors.length > 0 ? state.errors[state.errors.length - 1] : null,
        errorsByScope: (state) => (scope: AppErrorScope | string): AppError[] =>
            state.errors.filter(error => error.scope === scope),
    },
    actions: {
        setLoading(isLoading: boolean) {
            this.isLoading = isLoading;
        },
        setStoredPortfolio(stored: StoredPortfolioSummary, filePath: string | null = null) {
            this.activeReportId = stored.id;
            this.portfolio = stored.portfolio;
            this.reportFilePath = filePath;
            this.rebalance = null;
            this.isDataLoaded = true;
            this.showGettingStarted = false;
            this.clearErrorsByScope('upload');
            this.clearErrorsByScope('portfolio');
            this.clearErrorsByScope('rebalancing');
            this.portfolioRevision += 1;
        },
        setRebalance(rebalance: RebalancePlan | null) {
            this.rebalance = rebalance;
            this.clearErrorsByScope('rebalancing');
        },
        clearPortfolio() {
            this.portfolio = null;
            this.activeReportId = null;
            this.rebalance = null;
            this.isDataLoaded = false;
            this.showGettingStarted = true;
            this.reportFilePath = null;
            this.clearErrorsByScope('portfolio');
            this.clearErrorsByScope('rebalancing');
            this.portfolioRevision += 1;
        },
        toggleAboutModal(show?: boolean) {
            this.showAboutModal = typeof show === 'boolean' ? show : !this.showAboutModal;
        },
        setReportFilePath(filePath: string | null) {
            this.reportFilePath = filePath;
        },
        pushError(input: AppErrorInput) {
            const scope: AppErrorScope = input.scope ?? 'global';
            if (!input.append) {
                this.clearErrorsByScope(scope);
            }
            const error: AppError = {
                id: createErrorId(),
                scope,
                message: input.message,
                code: input.code,
                details: input.details,
                createdAt: Date.now(),
            };
            this.errors.push(error);
            return error;
        },
        clearErrorsByScope(scope: AppErrorScope | string) {
            this.errors = this.errors.filter(error => error.scope !== scope);
        },
        dismissError(id: string) {
            this.errors = this.errors.filter(error => error.id !== id);
        },
        clearAllErrors() {
            this.errors = [];
        },
    },
});
