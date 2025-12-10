import { computed, reactive, readonly } from "vue";
import type { TargetAllocation, TradeInstruction } from "../types/rebalance.ts";
import { useAppStore } from "../stores/appStore.ts";
import { TauriService, isTauriCommandError } from "../services/tauri.ts";
import { type NonNegativeNumber, nonNegative } from "../types/numeric.ts";

interface RebalancingState {
  targets: TargetAllocation[];
  actions: TradeInstruction[];
  depositAmount: NonNegativeNumber;
  isCalculating: boolean;
  hasCalculated: boolean;
}

interface TargetValidationResult {
  global: string[];
  perSymbol: Record<string, string[]>;
}

export function useRebalancing() {
  const appStore = useAppStore();
  const portfolio = computed(() => appStore.portfolio);
  const reportFilePath = computed(() => appStore.reportFilePath);

  const state = reactive<RebalancingState>({
    targets: [],
    actions: [],
    depositAmount: nonNegative(0),
    isCalculating: false,
    hasCalculated: false,
  });

  const initializeTargets = () => {
    if (!portfolio.value?.positions?.length) {
      state.targets = [];
      return;
    }

    state.targets = portfolio.value.positions.map((position) => ({
      symbol: position.symbol,
      targetPercent: position.allocationPercent ?? 0,
    }));
  };

  const totalTargetPercent = computed(() =>
    state.targets.reduce((sum, target) => sum + target.targetPercent, 0),
  );

  const isValidTargetAllocation = computed(
    () => Math.abs(totalTargetPercent.value - 100) < 0.01,
  );

  const canCalculate = computed(
    () =>
      portfolio.value !== null &&
      state.targets.length > 0 &&
      isValidTargetAllocation.value &&
      !state.isCalculating,
  );

  const updateTarget = (symbol: string, targetPercent: number) => {
    const target = state.targets.find((t) => t.symbol === symbol);
    if (target) {
      target.targetPercent = targetPercent;
    }
  };

  const addTarget = (symbol: string, targetPercent: number = 0) => {
    const existingTarget = state.targets.find((t) => t.symbol === symbol);
    if (!existingTarget) {
      state.targets.push({ symbol, targetPercent });
    }
  };

  const removeTarget = (symbol: string) => {
    const index = state.targets.findIndex((t) => t.symbol === symbol);
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
      appStore.pushError({
        scope: "rebalancing",
        message: "Cannot calculate rebalancing: invalid portfolio or targets",
      });
      return;
    }

    if (!reportFilePath.value) {
      appStore.pushError({
        scope: "rebalancing",
        message:
          "Original report file is required for rebalancing. Please import the portfolio again.",
      });
      return;
    }

    try {
      state.isCalculating = true;
      const plan = await TauriService.calcRebalance(
        reportFilePath.value,
        state.targets,
        state.depositAmount,
      );
      state.actions = plan.trades;
      state.hasCalculated = true;
      appStore.setRebalance(plan);
      appStore.clearErrorsByScope("rebalancing");
    } catch (error) {
      console.error("Rebalancing calculation error:", error);
      if (isTauriCommandError(error)) {
        appStore.pushError({
          scope: "rebalancing",
          message: error.message,
          details: error.details,
          code: error.code,
        });
      } else {
        const errorMessage =
          error instanceof Error
            ? error.message
            : `Failed to calculate rebalancing: ${JSON.stringify(error)}`;
        appStore.pushError({
          scope: "rebalancing",
          message: errorMessage,
          details: error instanceof Error ? error.stack : undefined,
        });
      }
    } finally {
      state.isCalculating = false;
    }
  };

  const getTargetForSymbol = (symbol: string): TargetAllocation | undefined =>
    state.targets.find((t) => t.symbol === symbol);

  const getActionForSymbol = (symbol: string): TradeInstruction | undefined =>
    state.actions.find((a) => a.symbol === symbol);

  const formatActions = () => {
    const currency = portfolio.value?.accountInfo.baseCurrency || "USD";
    const positions = portfolio.value?.positions ?? [];
    const targetMap = new Map(
      state.targets.map((target) => [target.symbol, target.targetPercent]),
    );
    const shareFormatter = new Intl.NumberFormat("en-US", {
      minimumFractionDigits: 0,
      maximumFractionDigits: 4,
    });

    return state.actions.map((action) => {
      const position = positions.find((pos) => pos.symbol === action.symbol);
      const currentPercent = position?.allocationPercent ?? 0;
      const targetPercent = targetMap.get(action.symbol) ?? 0;
      const formattedShares =
        action.quantityDelta === null
          ? "N/A"
          : shareFormatter.format(Math.abs(action.quantityDelta));

      return {
        ...action,
        formattedAmount: new Intl.NumberFormat("en-US", {
          style: "currency",
          currency,
        }).format(Math.abs(action.valueDelta)),
        formattedShares,
        actionText: action.action,
        actionColor: action.action === "Buy" ? "success" : "danger",
        currentPercent,
        targetPercent,
      };
    });
  };

  const validateTargets = (): TargetValidationResult => {
    const errors: string[] = [];
    const perSymbol: Record<string, string[]> = {};

    if (state.targets.length === 0) {
      errors.push("No target allocations defined");
    }

    const negativeTargets = state.targets.filter((t) => t.targetPercent < 0);
    if (negativeTargets.length > 0) {
      negativeTargets.forEach((target) => {
        if (!perSymbol[target.symbol]) {
          perSymbol[target.symbol] = [];
        }
        perSymbol[target.symbol].push("Target allocation cannot be negative.");
      });
      errors.push("Some targets have negative allocations.");
    }

    const duplicateSymbols = state.targets
      .map((t) => t.symbol)
      .filter((symbol, index, arr) => arr.indexOf(symbol) !== index);

    const uniqueDuplicateSymbols = Array.from(new Set(duplicateSymbols));

    if (uniqueDuplicateSymbols.length > 0) {
      errors.push(
        `Duplicate symbols found: ${uniqueDuplicateSymbols.join(", ")}`,
      );
      uniqueDuplicateSymbols.forEach((symbol) => {
        if (!perSymbol[symbol]) {
          perSymbol[symbol] = [];
        }
        perSymbol[symbol].push("Symbol is duplicated in target allocations.");
      });
    }

    state.targets.forEach((target) => {
      if (!perSymbol[target.symbol]) {
        perSymbol[target.symbol] = [];
      }

      if (!Number.isFinite(target.targetPercent)) {
        perSymbol[target.symbol].push(
          "Target allocation must be a valid number.",
        );
      }

      if (target.targetPercent > 100) {
        perSymbol[target.symbol].push("Target allocation cannot exceed 100%.");
      }
    });

    Object.keys(perSymbol).forEach((symbol) => {
      if (perSymbol[symbol].length === 0) {
        delete perSymbol[symbol];
      }
    });

    return {
      global: errors,
      perSymbol,
    };
  };

  const autoBalanceTargets = () => {
    if (state.targets.length === 0) return;

    const currentTotal = totalTargetPercent.value;
    const remaining = 100 - currentTotal;

    if (Math.abs(remaining) < 0.01) return;

    const adjustment = remaining / state.targets.length;

    state.targets.forEach((target) => {
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
