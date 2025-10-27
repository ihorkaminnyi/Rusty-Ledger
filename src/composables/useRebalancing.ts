import { computed, reactive, readonly } from 'vue';
import type { TargetAllocation, TradeInstruction } from '../types/rebalance.ts';
import { useAppStore } from '../stores/appStore.ts';
import { TauriService } from '../services/tauri.ts';

interface RebalancingState {
    targets: TargetAllocation[];
    actions: TradeInstruction[];
    isCalculating: boolean;
    hasCalculated: boolean;
}

export function useRebalancing() {
    const appStore = useAppStore();
    const portfolio = computed(() => appStore.portfolio);
    const reportFilePath = computed(() => appStore.reportFilePath);

    const state = reactive<RebalancingState>({
        targets: [],
        actions: [],
        isCalculating: false,
        hasCalculated: false,
    });

    const initializeTargets = () => {
        if (!portfolio.value?.positions?.length) {
            state.targets = [];
            return;
        }

        state.targets = portfolio.value.positions.map(position => ({
            symbol: position.symbol,
            targetPercent: position.allocationPercent ?? 0,
        }));
    };

    const totalTargetPercent = computed(() =>
        state.targets.reduce((sum, target) => sum + target.targetPercent, 0),
    );

    const isValidTargetAllocation = computed(() => Math.abs(totalTargetPercent.value - 100) < 0.01);

    const canCalculate = computed(
        () =>
            portfolio.value !== null &&
            state.targets.length > 0 &&
            isValidTargetAllocation.value &&
            !state.isCalculating,
    );

    const updateTarget = (symbol: string, targetPercent: number) => {
        const target = state.targets.find(t => t.symbol === symbol);
        if (target) {
            target.targetPercent = targetPercent;
        }
    };

    const addTarget = (symbol: string, targetPercent: number = 0) => {
        const existingTarget = state.targets.find(t => t.symbol === symbol);
        if (!existingTarget) {
            state.targets.push({ symbol, targetPercent });
        }
    };

    const removeTarget = (symbol: string) => {
        const index = state.targets.findIndex(t => t.symbol === symbol);
        if (index > -1) {
            state.targets.splice(index, 1);
        }
    };

    const resetTargets = () => {
        initializeTargets();
        state.actions = [];
        state.hasCalculated = false;
        appStore.setRebalance(null);
    };

    const clearTargets = () => {
        state.targets = [];
        state.actions = [];
        state.hasCalculated = false;
        appStore.setRebalance(null);
    };

    const calculateRebalancing = async (): Promise<void> => {
        if (!portfolio.value || !canCalculate.value) {
            appStore.setError('Cannot calculate rebalancing: invalid portfolio or targets');
            return;
        }

        if (!reportFilePath.value) {
            appStore.setError(
                'Original report file is required for rebalancing. Please import the portfolio again.',
            );
            return;
        }

        try {
            state.isCalculating = true;
            const plan = await TauriService.calcRebalance(reportFilePath.value, state.targets);
            state.actions = plan.trades;
            state.hasCalculated = true;
            appStore.setRebalance(plan);
            appStore.setError(null);
        } catch (error) {
            console.error('Rebalancing calculation error:', error);
            const errorMessage =
                error instanceof Error
                    ? error.message
                    : `Failed to calculate rebalancing: ${JSON.stringify(error)}`;
            appStore.setError(errorMessage);
        } finally {
            state.isCalculating = false;
        }
    };

    const getTargetForSymbol = (symbol: string): TargetAllocation | undefined =>
        state.targets.find(t => t.symbol === symbol);

    const getActionForSymbol = (symbol: string): TradeInstruction | undefined =>
        state.actions.find(a => a.symbol === symbol);

    const formatActions = () => {
        const currency = portfolio.value?.accountInfo.baseCurrency || 'USD';
        const positions = portfolio.value?.positions ?? [];
        const targetMap = new Map(state.targets.map(target => [target.symbol, target.targetPercent]));
        const shareFormatter = new Intl.NumberFormat('en-US', {
            minimumFractionDigits: 0,
            maximumFractionDigits: 4,
        });

        return state.actions.map(action => {
            const position = positions.find(pos => pos.symbol === action.symbol);
            const currentPercent = position?.allocationPercent ?? 0;
            const targetPercent = targetMap.get(action.symbol) ?? 0;
            const formattedShares =
                action.quantityDelta === null ? 'N/A' : shareFormatter.format(Math.abs(action.quantityDelta));

            return {
                ...action,
                formattedAmount: new Intl.NumberFormat('en-US', {
                    style: 'currency',
                    currency,
                }).format(Math.abs(action.valueDelta)),
                formattedShares,
                actionText: action.action,
                actionColor: action.action === 'Buy' ? 'success' : 'danger',
                currentPercent,
                targetPercent,
            };
        });
    };

    const validateTargets = (): string[] => {
        const errors: string[] = [];

        if (state.targets.length === 0) {
            errors.push('No target allocations defined');
        }

        if (!isValidTargetAllocation.value) {
            errors.push(
                `Target allocations must sum to 100% (currently ${totalTargetPercent.value.toFixed(2)}%)`,
            );
        }

        const negativeTargets = state.targets.filter(t => t.targetPercent < 0);
        if (negativeTargets.length > 0) {
            errors.push('Target percentages cannot be negative');
        }

        const duplicateSymbols = state.targets
        .map(t => t.symbol)
        .filter((symbol, index, arr) => arr.indexOf(symbol) !== index);

        if (duplicateSymbols.length > 0) {
            errors.push(`Duplicate symbols found: ${duplicateSymbols.join(', ')}`);
        }

        return errors;
    };

    const autoBalanceTargets = () => {
        if (state.targets.length === 0) return;

        const currentTotal = totalTargetPercent.value;
        const remaining = 100 - currentTotal;

        if (Math.abs(remaining) < 0.01) return;

        const adjustment = remaining / state.targets.length;

        state.targets.forEach(target => {
            target.targetPercent = Math.max(0, target.targetPercent + adjustment);
        });
    };

    return {
        state: readonly(state),
        totalTargetPercent,
        isValidTargetAllocation,
        canCalculate,
        initializeTargets,
        updateTarget,
        addTarget,
        removeTarget,
        resetTargets,
        clearTargets,
        calculateRebalancing,
        getTargetForSymbol,
        getActionForSymbol,
        formatActions,
        validateTargets,
        autoBalanceTargets,
    };
}
