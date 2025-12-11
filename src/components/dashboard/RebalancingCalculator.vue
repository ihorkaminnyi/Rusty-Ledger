<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import InputNumber from "primevue/inputnumber";
import Button from "primevue/button";
import Message from "primevue/message";
import Tag from "primevue/tag";
import Dialog from "primevue/dialog";
import { useRebalancing } from "../../composables/useRebalancing";
import { useAppStore } from "../../stores/appStore";
import { REBALANCE_STRATEGIES, RebalanceStrategy } from "../../types/rebalance";

const appStore = useAppStore();
const { portfolio } = storeToRefs(appStore);
const positions = computed(() => portfolio.value?.positions ?? []);
const accountInfo = computed(() => portfolio.value?.accountInfo);
const totalValue = computed(() => portfolio.value?.totals.marketValue ?? 0);
const {
    state: rebalancingState,
    totalTargetPercent,
    isValidTargetAllocation,
    canCalculate,
    initializeTargets,
    updateTarget,
    calculateRebalancing,
    resetTargets,
    formatActions,
    validateTargets,
} = useRebalancing();

const showRebalancingDialog = ref(false);

watch(
    positions,
    (currentPositions) => {
        if (
            currentPositions.length > 0 &&
            rebalancingState.targets.length === 0
        ) {
            initializeTargets();
        }
    },
    { immediate: true },
);

watch(
    () => rebalancingState.hasCalculated,
    (isCalculated) => {
        showRebalancingDialog.value = isCalculated;
    },
);

const formatCurrency = (value: number, currency: string = "USD"): string => {
    return new Intl.NumberFormat("en-US", {
        style: "currency",
        currency: currency,
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
    }).format(value);
};

const formatPercentage = (value: number): string => {
    return new Intl.NumberFormat("en-US", {
        style: "percent",
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
    }).format(value / 100);
};

const DIFFERENCE_EPSILON = 0.005;

const normalizeDifference = (difference: number): number => {
    return Math.abs(difference) <= DIFFERENCE_EPSILON ? 0 : difference;
};

const getDifferenceSeverity = (
    targetPercent: number,
    currentPercent: number,
): "success" | "danger" | "secondary" => {
    const normalized = normalizeDifference(targetPercent - currentPercent);
    if (normalized === 0) {
        return "secondary";
    }
    return normalized > 0 ? "success" : "danger";
};

const getDifferenceIcon = (
    targetPercent: number,
    currentPercent: number,
): string => {
    const normalized = normalizeDifference(targetPercent - currentPercent);
    if (normalized === 0) {
        return "pi pi-minus";
    }
    return normalized > 0 ? "pi pi-arrow-up" : "pi pi-arrow-down";
};

const getDifferenceLabel = (
    targetPercent: number,
    currentPercent: number,
): string => {
    const normalized = normalizeDifference(targetPercent - currentPercent);
    return formatPercentage(Math.abs(normalized));
};

const enhancedTargets = computed(() => {
    return rebalancingState.targets.map((target) => {
        const position = positions.value.find(
            (p) => p.symbol === target.symbol,
        );
        const calculatedPercent =
            position && totalValue.value > 0
                ? (position.marketValue / totalValue.value) * 100
                : 0;

        return {
            ...target,
            currentValue: position?.marketValue || 0,
            currentPercent: calculatedPercent,
            currency: accountInfo.value?.baseCurrency || "USD",
            description: "",
            assetCategory: "",
        };
    });
});

const validationResult = computed(() => validateTargets());
const globalValidationErrors = computed(() => validationResult.value.global);
const perTargetValidationErrors = computed(
    () => validationResult.value.perSymbol,
);

const allocationTooltipMessage = computed(() => {
    if (isValidTargetAllocation.value) {
        return null;
    }
    return `Total target allocation must equal 100%. Currently at ${totalTargetPercent.value.toFixed(2)}%.`;
});

const hasValidationErrors = computed(
    () =>
        globalValidationErrors.value.length > 0 ||
        Object.keys(perTargetValidationErrors.value).length > 0 ||
        Boolean(allocationTooltipMessage.value),
);

const canSubmitTargets = computed(
    () => canCalculate.value && !hasValidationErrors.value,
);

const handleTargetUpdate = (symbol: string, value: number | null) => {
    const targetValue = value || 0;
    updateTarget(symbol, Math.max(0, Math.min(100, targetValue)));
};

const handleCalculateRebalancing = async () => {
    if (!canSubmitTargets.value) return;
    await calculateRebalancing();
    showRebalancingDialog.value = rebalancingState.hasCalculated;
};

const handleResetTargets = () => {
    resetTargets();
    showRebalancingDialog.value = false;
};

const formattedActions = computed(() => {
    if (!rebalancingState.hasCalculated) return [];
    return formatActions();
});

const getActionIcon = (action: string): string => {
    return action.toLowerCase() === "buy"
        ? "pi pi-plus-circle"
        : "pi pi-minus-circle";
};

const getActionSeverity = (action: string): "success" | "danger" => {
    return action.toLowerCase() === "buy" ? "success" : "danger";
};

const hasTargetError = (symbol: string) => {
    const symbolErrors = perTargetValidationErrors.value[symbol] ?? [];
    if (symbolErrors.length > 0) return true;
    return Boolean(allocationTooltipMessage.value);
};

const targetTooltip = (symbol: string) => {
    const symbolErrors = perTargetValidationErrors.value[symbol] ?? [];
    const messages = [...symbolErrors];
    if (allocationTooltipMessage.value) {
        messages.push(allocationTooltipMessage.value);
    }
    return messages.join("\n");
};

const rebalanceStrategies = ref<RebalanceStrategy[]>([...REBALANCE_STRATEGIES]);
</script>

<template>
    <div class="flex flex-column gap-4">
        <div class="flex justify-content-between align-items-center">
            <div class="flex align-items-center gap-3">
                <h5 class="text-lg font-semibold text-700 m-0">
                    Target Allocation
                </h5>
                <div
                    :class="`text-lg font-semibold px-3 py-1 border-round ${isValidTargetAllocation ? 'bg-green-50 text-green-600 border-1 border-green-200' : 'bg-red-50 text-red-600 border-1 border-red-200'}`"
                >
                    {{ totalTargetPercent.toFixed(2) }}%
                </div>
            </div>
            <div class="flex gap-2">
                <Button
                    label="Reset to Current"
                    icon="pi pi-refresh"
                    severity="secondary"
                    size="small"
                    @click="handleResetTargets"
                    :disabled="rebalancingState.isCalculating"
                />
            </div>
        </div>

        <div class="flex justify-content-between gap-4">
            <FloatLabel variant="on">
                <InputNumber
                    v-model.number="rebalancingState.depositAmount"
                    mode="currency"
                    :min="0"
                    currency="USD"
                    locale="en-US"
                    inputId="deposit"
                />
                <label for="deposit">Top-up amount</label>
            </FloatLabel>
            <div class="card flex justify-center">
                <SelectButton
                    v-model="rebalancingState.rebalanceStrategy"
                    :options="rebalanceStrategies"
                />
            </div>
        </div>

        <div class="flex flex-column">
            <div v-if="globalValidationErrors.length" class="flex flex-column">
                <Message
                    v-for="error in globalValidationErrors"
                    :key="error"
                    severity="error"
                    :closable="false"
                    class="m-0"
                >
                    {{ error }}
                </Message>
            </div>

            <DataTable
                :value="enhancedTargets"
                class="targets-table text-sm"
                :scrollable="true"
                scrollHeight="22rem"
                v-if="enhancedTargets.length > 0"
            >
                <Column
                    field="symbol"
                    header="Asset"
                    :style="{ minWidth: '5rem' }"
                    frozen
                >
                    <template #body="{ data }">
                        <div class="symbol-cell">
                            <div class="symbol-main">{{ data.symbol }}</div>
                            <div
                                v-if="data.description"
                                class="symbol-description"
                            >
                                {{ data.description }}
                            </div>
                        </div>
                    </template>
                </Column>

                <Column
                    field="currentPercent"
                    header="Current %"
                    :style="{ minWidth: '6.5rem' }"
                >
                    <template #body="{ data }">
                        <div class="current-percent">
                            {{ formatPercentage(data.currentPercent) }}
                        </div>
                    </template>
                </Column>

                <Column
                    field="targetPercent"
                    header="Target %"
                    :style="{ width: '6.5rem', minWidth: '6.5rem' }"
                >
                    <template #body="{ data }">
                        <InputNumber
                            :modelValue="data.targetPercent"
                            @update:modelValue="
                                (value: number | null) =>
                                    handleTargetUpdate(data.symbol, value)
                            "
                            mode="decimal"
                            :minFractionDigits="2"
                            :maxFractionDigits="2"
                            :min="0"
                            :max="100"
                            suffix="%"
                            :disabled="rebalancingState.isCalculating"
                            class="target-input p-inputnumber-sm w-full"
                            inputClass="p-inputtext-sm w-full"
                            :class="{
                                'p-invalid': hasTargetError(data.symbol),
                            }"
                            v-tooltip.bottom="targetTooltip(data.symbol)"
                        />
                    </template>
                </Column>

                <Column
                    field="difference"
                    header="Difference"
                    :style="{ minWidth: '5rem' }"
                >
                    <template #body="{ data }">
                        <Tag
                            :value="
                                getDifferenceLabel(
                                    data.targetPercent,
                                    data.currentPercent,
                                )
                            "
                            :severity="
                                getDifferenceSeverity(
                                    data.targetPercent,
                                    data.currentPercent,
                                )
                            "
                            class="difference-tag"
                        >
                            <template #default>
                                <i
                                    :class="[
                                        getDifferenceIcon(
                                            data.targetPercent,
                                            data.currentPercent,
                                        ),
                                        'difference-icon',
                                    ]"
                                ></i>
                                {{
                                    getDifferenceLabel(
                                        data.targetPercent,
                                        data.currentPercent,
                                    )
                                }}
                            </template>
                        </Tag>
                    </template>
                </Column>

                <Column
                    field="currentValue"
                    header="Current Value"
                    :style="{ minWidth: '5rem' }"
                >
                    <template #body="{ data }">
                        <div class="current-value">
                            {{
                                formatCurrency(data.currentValue, data.currency)
                            }}
                        </div>
                    </template>
                </Column>
            </DataTable>

            <div v-else class="text-center py-6 text-600">
                <i class="pi pi-calculator text-4xl text-300 mb-3"></i>
                <h4 class="text-lg font-semibold text-500 mb-2">
                    No positions available
                </h4>
                <p class="text-sm line-height-3">
                    Load a portfolio to start rebalancing calculations.
                </p>
            </div>

            <div
                class="flex justify-content-center gap-2 pt-3 border-top-1 surface-border flex-wrap"
            >
                <Button
                    label="Calculate Rebalancing Plan"
                    icon="pi pi-calculator"
                    @click="handleCalculateRebalancing"
                    :disabled="!canSubmitTargets"
                    :loading="rebalancingState.isCalculating"
                    size="small"
                    class="p-button-sm px-3"
                />
                <Button
                    v-if="rebalancingState.hasCalculated"
                    label="View Plan"
                    icon="pi pi-list"
                    severity="secondary"
                    size="small"
                    class="p-button-sm px-3"
                    @click="showRebalancingDialog = true"
                    :disabled="rebalancingState.isCalculating"
                />
            </div>
        </div>

        <Dialog
            v-if="rebalancingState.hasCalculated"
            v-model:visible="showRebalancingDialog"
            modal
            header="Rebalancing Plan"
            dismissableMask
            :draggable="false"
            class="rebalancing-dialog w-15rem sm:w-10 md:w-9 lg:w-8"
        >
            <div class="flex justify-content-between align-items-center mb-3">
                <div
                    class="text-sm font-medium text-600 bg-gray-50 px-3 py-2 border-round"
                >
                    {{ formattedActions.length }} actions required
                </div>
            </div>

            <div v-if="formattedActions.length">
                <DataTable
                    :value="formattedActions"
                    class="border-1 surface-border border-round text-sm"
                    :scrollable="true"
                    scrollHeight="300px"
                    size="small"
                >
                    <Column
                        field="action"
                        header="Action"
                        :style="{ minWidth: '6rem' }"
                    >
                        <template #body="{ data }">
                            <Tag
                                :value="data.actionText"
                                :severity="getActionSeverity(data.action)"
                                class="action-tag"
                            >
                                <template #default>
                                    <i
                                        :class="getActionIcon(data.action)"
                                        class="action-icon"
                                    ></i>
                                    {{ data.actionText }}
                                </template>
                            </Tag>
                        </template>
                    </Column>

                    <Column
                        field="symbol"
                        header="Asset"
                        :style="{ minWidth: '5rem' }"
                    >
                        <template #body="{ data }">
                            <div class="symbol-main">{{ data.symbol }}</div>
                        </template>
                    </Column>

                    <Column
                        field="amount"
                        header="Amount"
                        :style="{ minWidth: '6rem' }"
                    >
                        <template #body="{ data }">
                            <div class="amount-cell" :class="data.actionColor">
                                {{ data.formattedAmount }}
                            </div>
                        </template>
                    </Column>

                    <Column
                        field="shares"
                        header="Shares"
                        :style="{ minWidth: '5rem' }"
                    >
                        <template #body="{ data }">
                            <div class="shares-cell" :class="data.actionColor">
                                {{ data.formattedShares }}
                            </div>
                        </template>
                    </Column>

                    <Column
                        field="currentPercent"
                        header="Current %"
                        :style="{ minWidth: '6rem' }"
                    >
                        <template #body="{ data }">
                            <div class="current-percent">
                                {{ formatPercentage(data.currentPercent) }}
                            </div>
                        </template>
                    </Column>

                    <Column
                        field="targetPercent"
                        header="Target %"
                        :style="{ minWidth: '5rem' }"
                    >
                        <template #body="{ data }">
                            <div class="target-percent">
                                {{ formatPercentage(data.targetPercent) }}
                            </div>
                        </template>
                    </Column>

                    <Column
                        field="instructions"
                        header="Instructions"
                        :style="{ minWidth: '250px' }"
                    >
                        <template #body="{ data }">
                            <div class="instructions-cell">
                                {{ data.action === "buy" ? "Buy" : "Sell" }}
                                {{ data.formattedShares }} shares ({{
                                    data.formattedAmount
                                }}) of {{ data.symbol }}
                            </div>
                        </template>
                    </Column>
                </DataTable>
            </div>

            <div v-else class="text-center py-6 text-600">
                <i class="pi pi-check-circle text-4xl text-green-500 mb-3"></i>
                <h4 class="text-lg font-semibold text-500 mb-2">
                    Portfolio is already balanced
                </h4>
                <p class="text-sm line-height-3">
                    No rebalancing actions are required based on your target
                    allocation.
                </p>
            </div>
        </Dialog>
    </div>
</template>

<style scoped>
.difference-tag,
.action-tag {
    font-weight: 600;
    font-size: 0.75rem;
    padding: 0.375rem 0.75rem;
    border-radius: 6px;
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
}

.action-tag {
    text-transform: capitalize;
}

.flex {
    animation: fadeIn 0.5s ease-in-out;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>
