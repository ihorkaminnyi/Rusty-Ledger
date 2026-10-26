import assert from 'node:assert/strict';
import test, { type TestContext } from 'node:test';
import { createPinia, setActivePinia } from 'pinia';
import { effectScope } from 'vue';
import { useAppStore } from '../src/stores/appStore.ts';
import { usePortfolioReports } from '../src/composables/usePortfolioReports.ts';
import { useRebalancing } from '../src/composables/useRebalancing.ts';
import { TauriService } from '../src/services/tauri.ts';
import type { StoredPortfolioSummary, StoredReportListItem } from '../src/types/portfolio.ts';
import type { RebalancePlan } from '../src/types/rebalance.ts';

function stored(id: number): StoredPortfolioSummary {
    return {
        id,
        portfolio: {
            accountInfo: { name: `Account ${id}`, baseCurrency: 'USD' },
            statement: { period: `2026-06-0${id}` },
            totals: { marketValue: 100, costBasis: 100, unrealizedPl: 0, unrealizedPlPercent: 0, totalPositions: 1 },
            positions: [{ symbol: 'VTI', quantity: 2, price: 50, marketValue: 100, allocationPercent: 100, unrealizedPl: 0, roiPercent: 0 }],
        },
    };
}

function row(id: number): StoredReportListItem {
    return { id, accountName: `Account ${id}`, statementPeriod: `2026-06-0${id}`, createdAt: '2026-06-03T10:00:00Z', totalPositions: 1 };
}

const plan: RebalancePlan = {
    totalValue: 100,
    trades: [{ symbol: 'VTI', action: 'Buy', valueDelta: 10, quantityDelta: 0.2, priceUsed: 50 }],
};

function setup(t: TestContext) {
    setActivePinia(createPinia());
    t.mock.method(TauriService, 'isTauriAvailable', () => true);
    const scope = effectScope();
    t.after(() => scope.stop());
    const notices: Array<{ severity: string; message: string }> = [];
    const controller = scope.run(() => usePortfolioReports((severity, message) => {
        notices.push({ severity, message });
    }))!;
    return { store: useAppStore(), controller, notices, scope };
}

function deferred<T>() {
    let resolve!: (value: T) => void;
    let reject!: (error: Error) => void;
    const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
    return { promise, resolve, reject };
}

test('failed list loading can be retried without showing stale errors', async (t) => {
    const { controller } = setup(t);
    let attempts = 0;
    t.mock.method(TauriService, 'listPortfolioSummaries', async () => {
        if (attempts++ === 0) throw new Error('Database unavailable');
        return [row(2), row(1)];
    });

    await controller.loadReports();
    assert.equal(controller.listError.value, 'Database unavailable');
    assert.equal(controller.isLoadingList.value, false);
    await controller.loadReports();
    assert.equal(controller.listError.value, null);
    assert.deepEqual(controller.reports.value, [row(2), row(1)]);
});

test('opening latest reloads the same report and resets its rebalance', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(2), '/original.csv');
    store.setRebalance(plan);
    const revision = store.portfolioRevision;
    t.mock.method(TauriService, 'getLatestPortfolioSummary', async () => stored(2));

    await controller.openLatestReport();

    assert.equal(store.activeReportId, 2);
    assert.equal(store.portfolioRevision, revision + 1);
    assert.equal(store.reportFilePath, null);
    assert.equal(store.rebalance, null);
    assert.equal(store.isLoading, false);
});

test('failed latest loading preserves the current portfolio and releases loading state', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(1));
    store.setRebalance(plan);
    const revision = store.portfolioRevision;
    t.mock.method(TauriService, 'getLatestPortfolioSummary', async () => { throw new Error('Read failed'); });

    await controller.openLatestReport();

    assert.equal(store.activeReportId, 1);
    assert.equal(store.portfolioRevision, revision);
    assert.deepEqual(store.rebalance, plan);
    assert.equal(controller.operationError.value, 'Read failed');
    assert.equal(store.isLoading, false);
});

test('closing a loading list allows opening the latest report immediately', async (t) => {
    const { store, controller } = setup(t);
    const response = deferred<StoredReportListItem[]>();
    t.mock.method(TauriService, 'listPortfolioSummaries', () => response.promise);
    t.mock.method(TauriService, 'getLatestPortfolioSummary', async () => stored(2));
    store.isSavedReportsOpen = true;
    const loading = controller.loadReports();

    controller.setReportsVisible(false);
    await controller.openLatestReport();

    assert.equal(store.activeReportId, 2);
    assert.equal(store.isSavedReportsOpen, false);
    response.resolve([row(2), row(1)]);
    await loading;
    assert.equal(store.activeReportId, 2);
});

test('an empty database closes the current portfolio and shows the welcome message', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(1));
    t.mock.method(TauriService, 'getLatestPortfolioSummary', async () => null);

    await controller.openLatestReport();

    assert.equal(store.activeReportId, null);
    assert.equal(store.isDataLoaded, false);
    assert.equal(store.showGettingStarted, true);
    assert.equal(controller.noSavedReport.value, true);
});

test('opening a selected report replaces the portfolio and closes the list', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(2));
    store.isSavedReportsOpen = true;
    t.mock.method(TauriService, 'getPortfolioSummary', async (id: number) => stored(id));

    await controller.openReport(1);

    assert.equal(store.activeReportId, 1);
    assert.equal(store.isSavedReportsOpen, false);
    assert.equal(store.reportFilePath, null);
});

test('opening the current row only closes the list and preserves rebalance settings', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(1));
    store.setRebalance(plan);
    store.isSavedReportsOpen = true;
    const revision = store.portfolioRevision;
    const load = t.mock.method(TauriService, 'getPortfolioSummary', async () => stored(1));

    await controller.openReport(1);

    assert.equal(load.mock.callCount(), 0);
    assert.equal(store.portfolioRevision, revision);
    assert.deepEqual(store.rebalance, plan);
    assert.equal(store.isSavedReportsOpen, false);
});

test('a missing selected report refreshes the list and preserves the open portfolio', async (t) => {
    const { store, controller, notices } = setup(t);
    store.setStoredPortfolio(stored(1));
    store.isSavedReportsOpen = true;
    t.mock.method(TauriService, 'getPortfolioSummary', async () => null);
    t.mock.method(TauriService, 'listPortfolioSummaries', async () => [row(1)]);

    await controller.openReport(2);

    assert.equal(store.activeReportId, 1);
    assert.equal(store.isSavedReportsOpen, true);
    assert.deepEqual(controller.reports.value, [row(1)]);
    assert.equal(notices[0].severity, 'info');
});

test('cancelling a deletion never calls the backend', async (t) => {
    const { controller } = setup(t);
    controller.reports.value = [row(1)];
    const remove = t.mock.method(TauriService, 'deletePortfolioSummary', async () => true);

    controller.requestDelete(row(1));
    assert.equal(remove.mock.callCount(), 0);
    controller.cancelConfirmation();
    await controller.confirmDeletion();

    assert.equal(remove.mock.callCount(), 0);
    assert.deepEqual(controller.reports.value, [row(1)]);
});

test('deleting another report preserves the current portfolio and keeps the list open', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(1));
    store.setRebalance(plan);
    store.isSavedReportsOpen = true;
    const revision = store.portfolioRevision;
    controller.reports.value = [row(2), row(1)];
    t.mock.method(TauriService, 'deletePortfolioSummary', async () => true);

    controller.requestDelete(row(2));
    await controller.confirmDeletion();

    assert.deepEqual(controller.reports.value, [row(1)]);
    assert.equal(store.activeReportId, 1);
    assert.equal(store.portfolioRevision, revision);
    assert.deepEqual(store.rebalance, plan);
    assert.equal(store.isSavedReportsOpen, true);
    assert.equal(controller.confirmation.value, null);
});

test('deleting the active report clears all portfolio state and leaves other reports available', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(2), '/original.csv');
    store.setRebalance(plan);
    store.isSavedReportsOpen = true;
    controller.reports.value = [row(2), row(1)];
    t.mock.method(TauriService, 'deletePortfolioSummary', async () => true);

    controller.requestDelete(row(2));
    await controller.confirmDeletion();

    assert.equal(store.portfolio, null);
    assert.equal(store.activeReportId, null);
    assert.equal(store.rebalance, null);
    assert.equal(store.reportFilePath, null);
    assert.equal(store.isDataLoaded, false);
    assert.equal(store.showGettingStarted, true);
    assert.equal(store.isSavedReportsOpen, true);
    assert.deepEqual(controller.reports.value, [row(1)]);
});

test('an already deleted active report is removed from the stale list', async (t) => {
    const { store, controller, notices } = setup(t);
    store.setStoredPortfolio(stored(1));
    store.isSavedReportsOpen = true;
    controller.reports.value = [row(1)];
    t.mock.method(TauriService, 'deletePortfolioSummary', async () => false);

    controller.requestDelete(row(1));
    await controller.confirmDeletion();

    assert.deepEqual(controller.reports.value, []);
    assert.equal(store.activeReportId, null);
    assert.equal(notices[0].severity, 'info');
    assert.equal(controller.noSavedReport.value, true);
});

test('failed deletion preserves data and confirmation for retry', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(1));
    controller.reports.value = [row(1)];
    t.mock.method(TauriService, 'deletePortfolioSummary', async () => { throw new Error('Delete failed'); });

    controller.requestDelete(row(1));
    await controller.confirmDeletion();

    assert.deepEqual(controller.reports.value, [row(1)]);
    assert.equal(store.activeReportId, 1);
    assert.equal(controller.confirmation.value?.kind, 'delete');
    assert.equal(controller.operationError.value, 'Delete failed');
    assert.equal(store.isLoading, false);
});

test('clearing all reports clears the portfolio but keeps the empty list open', async (t) => {
    const { store, controller, notices } = setup(t);
    store.setStoredPortfolio(stored(1));
    store.isSavedReportsOpen = true;
    controller.reports.value = [row(2), row(1)];
    t.mock.method(TauriService, 'clearPortfolioSummaries', async () => 2);

    controller.requestClearAll();
    assert.deepEqual(controller.confirmation.value, { kind: 'clear', count: 2 });
    await controller.confirmDeletion();

    assert.deepEqual(controller.reports.value, []);
    assert.equal(store.activeReportId, null);
    assert.equal(store.isSavedReportsOpen, true);
    assert.equal(controller.confirmation.value, null);
    assert.equal(notices[0].message, '2 saved reports deleted.');
});

test('failed clear preserves all reports and the open portfolio', async (t) => {
    const { store, controller } = setup(t);
    store.setStoredPortfolio(stored(1));
    controller.reports.value = [row(2), row(1)];
    t.mock.method(TauriService, 'clearPortfolioSummaries', async () => { throw new Error('Clear failed'); });

    controller.requestClearAll();
    await controller.confirmDeletion();

    assert.deepEqual(controller.reports.value, [row(2), row(1)]);
    assert.equal(store.activeReportId, 1);
    assert.equal(controller.confirmation.value?.kind, 'clear');
    assert.equal(controller.operationError.value, 'Clear failed');
    assert.equal(store.isLoading, false);
});

test('a pending deletion blocks duplicate operations, closing the list and importing', async (t) => {
    const { store, controller } = setup(t);
    store.isSavedReportsOpen = true;
    controller.reports.value = [row(1)];
    const response = deferred<boolean>();
    const remove = t.mock.method(TauriService, 'deletePortfolioSummary', () => response.promise);
    const latest = t.mock.method(TauriService, 'getLatestPortfolioSummary', async () => stored(1));
    controller.requestDelete(row(1));
    const deleting = controller.confirmDeletion();

    assert.equal(store.canImport, false);
    controller.setReportsVisible(false);
    controller.cancelConfirmation();
    await controller.confirmDeletion();
    await controller.openLatestReport();
    assert.equal(store.isSavedReportsOpen, true);
    assert.equal(controller.confirmation.value?.kind, 'delete');
    assert.equal(remove.mock.callCount(), 1);
    assert.equal(latest.mock.callCount(), 0);

    response.resolve(true);
    await deleting;
    assert.equal(store.isLoading, false);
});

test('rebalance uses the saved report ID without an original CSV path', async (t) => {
    const { store, scope } = setup(t);
    store.setStoredPortfolio(stored(2));
    const rebalance = scope.run(() => useRebalancing())!;
    rebalance.initializeTargets();
    const calculate = t.mock.method(TauriService, 'calcRebalanceForSummary', async () => plan);

    await rebalance.calculateRebalancing();

    assert.equal(store.reportFilePath, null);
    assert.equal(calculate.mock.calls[0].arguments[0], 2);
    assert.deepEqual(store.rebalance, plan);
    assert.equal(rebalance.state.hasCalculated, true);
});

for (const outcome of ['success', 'failure'] as const) {
    test(`a stale rebalance ${outcome} is ignored after reopening the same report`, async (t) => {
        const { store, scope } = setup(t);
        store.setStoredPortfolio(stored(1));
        const rebalance = scope.run(() => useRebalancing())!;
        rebalance.initializeTargets();
        const response = deferred<RebalancePlan>();
        t.mock.method(TauriService, 'calcRebalanceForSummary', () => response.promise);
        const calculating = rebalance.calculateRebalancing();
        store.setStoredPortfolio(stored(1));

        if (outcome === 'success') response.resolve(plan);
        else response.reject(new Error('Old calculation failed'));
        await calculating;

        assert.equal(store.rebalance, null);
        assert.equal(rebalance.state.hasCalculated, false);
        assert.equal(store.errors.length, 0);
        assert.equal(rebalance.state.isCalculating, false);
    });
}

test('clearing rebalance after closing a report keeps the welcome state', (t) => {
    const { store } = setup(t);
    store.setStoredPortfolio(stored(1));
    store.clearPortfolio();
    store.setRebalance(null);

    assert.equal(store.isDataLoaded, false);
    assert.equal(store.showGettingStarted, true);
    assert.equal(store.activeReportId, null);
});
