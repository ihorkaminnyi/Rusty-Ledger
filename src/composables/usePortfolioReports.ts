import { computed, ref, watch } from 'vue';
import { useAppStore } from '../stores/appStore.ts';
import { TauriService, isTauriCommandError } from '../services/tauri.ts';
import type { StoredReportListItem } from '../types/portfolio.ts';

export type ReportConfirmation =
    | { kind: 'delete'; report: StoredReportListItem }
    | { kind: 'clear'; count: number };

export interface ReportPendingAction {
    kind: 'open' | 'latest' | 'delete' | 'clear';
    id?: number;
}

type Notify = (severity: 'success' | 'info', message: string) => void;

export function usePortfolioReports(notify: Notify = () => {}) {
    const appStore = useAppStore();
    const reports = ref<StoredReportListItem[]>([]);
    const isLoadingList = ref(false);
    const listError = ref<string | null>(null);
    const operationError = ref<string | null>(null);
    const pendingAction = ref<ReportPendingAction | null>(null);
    const confirmation = ref<ReportConfirmation | null>(null);
    const noSavedReport = ref(false);
    const isBusy = computed(() =>
        appStore.isLoading || (appStore.isSavedReportsOpen && isLoadingList.value));

    watch(() => appStore.portfolioRevision, () => {
        if (appStore.hasPortfolio) noSavedReport.value = false;
    });

    const errorMessage = (error: unknown): string =>
        error instanceof Error ? error.message : String(error);

    const loadReports = async (): Promise<void> => {
        if (isLoadingList.value || !TauriService.isTauriAvailable()) return;
        isLoadingList.value = true;
        listError.value = null;
        try {
            reports.value = await TauriService.listPortfolioSummaries();
        } catch (error) {
            listError.value = errorMessage(error);
        } finally {
            isLoadingList.value = false;
        }
    };

    const setReportsVisible = (visible: boolean): void => {
        if (appStore.isLoading || pendingAction.value) return;
        appStore.isSavedReportsOpen = visible;
        confirmation.value = null;
        operationError.value = null;
        if (visible) void loadReports();
    };

    const runOperation = async (
        action: ReportPendingAction,
        operation: () => Promise<void>,
    ): Promise<void> => {
        if (isBusy.value || pendingAction.value || !TauriService.isTauriAvailable()) return;
        pendingAction.value = action;
        operationError.value = null;
        appStore.clearErrorsByScope('reports');
        appStore.setLoading(true);
        try {
            await operation();
        } catch (error) {
            operationError.value = errorMessage(error);
            appStore.pushError({
                scope: action.kind === 'open' || action.kind === 'latest' ? 'portfolio' : 'reports',
                message: operationError.value,
                code: isTauriCommandError(error) ? error.code : undefined,
                details: isTauriCommandError(error) ? error.details : undefined,
            });
        } finally {
            appStore.setLoading(false);
            pendingAction.value = null;
        }
    };

    const openLatestReport = async (): Promise<void> => {
        await runOperation({ kind: 'latest' }, async () => {
            noSavedReport.value = false;
            appStore.clearErrorsByScope('portfolio');
            const stored = await TauriService.getLatestPortfolioSummary();
            if (stored) {
                appStore.setStoredPortfolio(stored);
                appStore.isSavedReportsOpen = false;
            } else {
                appStore.clearPortfolio();
                noSavedReport.value = true;
            }
        });
    };

    const openReport = async (id: number): Promise<void> => {
        if (isBusy.value || confirmation.value) return;
        if (id === appStore.activeReportId) {
            setReportsVisible(false);
            return;
        }
        await runOperation({ kind: 'open', id }, async () => {
            const stored = await TauriService.getPortfolioSummary(id);
            if (!stored) {
                notify('info', 'The saved report no longer exists. The list has been refreshed.');
                await loadReports();
                return;
            }
            appStore.setStoredPortfolio(stored);
            appStore.isSavedReportsOpen = false;
        });
    };

    const requestDelete = (report: StoredReportListItem): void => {
        if (isBusy.value || listError.value || confirmation.value) return;
        operationError.value = null;
        confirmation.value = { kind: 'delete', report };
    };

    const requestClearAll = (): void => {
        if (isBusy.value || listError.value || confirmation.value || !reports.value.length) return;
        operationError.value = null;
        confirmation.value = { kind: 'clear', count: reports.value.length };
    };

    const cancelConfirmation = (): void => {
        if (pendingAction.value) return;
        confirmation.value = null;
        operationError.value = null;
    };

    const confirmDeletion = async (): Promise<void> => {
        const requested = confirmation.value;
        if (!requested) return;
        const action: ReportPendingAction = requested.kind === 'delete'
            ? { kind: 'delete', id: requested.report.id }
            : { kind: 'clear' };
        await runOperation(action, async () => {
            if (requested.kind === 'delete') {
                const id = requested.report.id;
                const deleted = await TauriService.deletePortfolioSummary(id);
                reports.value = reports.value.filter(report => report.id !== id);
                if (appStore.activeReportId === id) appStore.clearPortfolio();
                notify(deleted ? 'success' : 'info', deleted
                    ? `Report #${id} deleted.`
                    : `Report #${id} was already deleted.`);
            } else {
                const count = await TauriService.clearPortfolioSummaries();
                reports.value = [];
                appStore.clearPortfolio();
                notify('success', `${count} saved report${count === 1 ? '' : 's'} deleted.`);
            }
            confirmation.value = null;
            noSavedReport.value = reports.value.length === 0;
        });
    };

    return {
        reports, isLoadingList, listError, operationError, pendingAction,
        confirmation, noSavedReport, loadReports, setReportsVisible,
        openLatestReport, openReport, requestDelete, requestClearAll,
        confirmDeletion, cancelConfirmation,
    };
}
