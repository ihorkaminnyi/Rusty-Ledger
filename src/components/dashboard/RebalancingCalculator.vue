<script setup lang="ts">
import { computed, watch } from 'vue';
import { storeToRefs } from 'pinia';
import Card from 'primevue/card';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import InputNumber from 'primevue/inputnumber';
import Button from 'primevue/button';
import Message from 'primevue/message';
import Tag from 'primevue/tag';
import ProgressBar from 'primevue/progressbar';
import { useRebalancing } from '../../composables/useRebalancing';
import { useAppStore } from '../../stores/appStore';

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

watch(positions, (currentPositions) => {
  if (currentPositions.length > 0 && rebalancingState.targets.length === 0) {
    initializeTargets();
  }
}, { immediate: true });

const formatCurrency = (value: number, currency: string = 'USD'): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value);
};

const formatPercentage = (value: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'percent',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(value / 100);
};

const enhancedTargets = computed(() => {
  return rebalancingState.targets.map(target => {
    const position = positions.value.find(p => p.symbol === target.symbol);
    const calculatedPercent = position && totalValue.value > 0
        ? (position.marketValue / totalValue.value) * 100
        : 0;

    return {
      ...target,
      currentValue: position?.marketValue || 0,
      currentPercent: calculatedPercent,
      currency: accountInfo.value?.baseCurrency || 'USD',
      description: '',
      assetCategory: '',
    };
  });
});

const validationErrors = computed(() => validateTargets());
const hasValidationErrors = computed(() => validationErrors.value.length > 0);

const getProgressBarColor = computed(() => {
  const total = totalTargetPercent.value;
  if (Math.abs(total - 100) < 0.01) return 'success';
  if (total > 100) return 'danger';
  return 'info';
});

const handleTargetUpdate = (symbol: string, value: number | null) => {
  const targetValue = value || 0;
  updateTarget(symbol, Math.max(0, Math.min(100, targetValue)));
};

const handleCalculateRebalancing = async () => {
  if (!canCalculate.value) return;
  await calculateRebalancing();
};

const handleResetTargets = () => {
  resetTargets();
};

const formattedActions = computed(() => {
  if (!rebalancingState.hasCalculated) return [];
  return formatActions();
});

const getActionIcon = (action: string): string => {
  return action.toLowerCase() === 'buy' ? 'pi pi-plus-circle' : 'pi pi-minus-circle';
};

const getActionSeverity = (action: string): 'success' | 'danger' => {
  return action.toLowerCase() === 'buy' ? 'success' : 'danger';
};
</script>

<template>
  <div class="flex flex-column gap-4">
    <Card class="surface-0 border-1 surface-border border-round-xl shadow-1">
      <template #header>
        <div class="flex justify-content-between align-items-center p-4 pb-0 mb-3">
          <h3 class="text-xl font-semibold text-900 m-0 flex align-items-center gap-2">
            <i class="pi pi-calculator text-primary"></i>
            Rebalancing Calculator
          </h3>
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
      </template>

      <template #content>
        <div class="flex flex-column gap-4">
          <div class="flex justify-content-between align-items-center">
            <h4 class="text-lg font-semibold text-700 m-0">Target Allocation</h4>
            <div class="flex align-items-center gap-3">
              <div
                  :class="`text-lg font-semibold px-3 py-2 border-round ${isValidTargetAllocation ? 'bg-green-50 text-green-600 border-1 border-green-200' : 'bg-red-50 text-red-600 border-1 border-red-200'}`">
                Total: {{ totalTargetPercent.toFixed(2) }}%
              </div>
            </div>
          </div>

          <!-- Progress bar showing total percentage -->
          <div class="flex flex-column gap-2">
            <ProgressBar
                :value="totalTargetPercent"
                :severity="getProgressBarColor"
                :showValue="false"
                class="h-3 border-round"
            />
            <div class="flex justify-content-between text-xs text-600">
              <span>0%</span>
              <span class="font-semibold text-700">100%</span>
            </div>
          </div>

          <div v-if="hasValidationErrors" class="flex flex-column gap-2">
            <Message
                v-for="error in validationErrors"
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
              class="targets-table"
              :scrollable="true"
              scrollHeight="400px"
              v-if="enhancedTargets.length > 0"
          >
            <Column field="symbol" header="Asset" :style="{ minWidth: '5rem' }" frozen>
              <template #body="{ data }">
                <div class="symbol-cell">
                  <div class="symbol-main">{{ data.symbol }}</div>
                  <div v-if="data.description" class="symbol-description">{{ data.description }}</div>
                </div>
              </template>
            </Column>

            <Column field="currentPercent" header="Current %" :style="{ minWidth: '8rem' }">
              <template #body="{ data }">
                <div class="current-percent">
                  {{ formatPercentage(data.currentPercent) }}
                </div>
              </template>
            </Column>

            <Column field="targetPercent" header="Target %" :style="{ minWidth: '10px' }">
              <template #body="{ data }">
                <InputNumber
                    :modelValue="data.targetPercent"
                    @update:modelValue="(value: number | null) => handleTargetUpdate(data.symbol, value)"
                    mode="decimal"
                    :minFractionDigits="2"
                    :maxFractionDigits="2"
                    :min="0"
                    :max="100"
                    suffix="%"
                    :disabled="rebalancingState.isCalculating"
                    class="target-input"
                    :class="{ 'p-invalid': !isValidTargetAllocation }"
                />
              </template>
            </Column>

            <Column field="difference" header="Difference" :style="{ minWidth: '5rem' }">
              <template #body="{ data }">
                <Tag
                    :value="formatPercentage(data.targetPercent - data.currentPercent)"
                    :severity="data.targetPercent > data.currentPercent ? 'success' : data.targetPercent < data.currentPercent ? 'danger' : 'secondary'"
                    class="difference-tag"
                >
                  <template #default>
                    <i
                        :class="data.targetPercent > data.currentPercent ? 'pi pi-arrow-up' : data.targetPercent < data.currentPercent ? 'pi pi-arrow-down' : 'pi pi-minus'"
                        class="difference-icon"
                    ></i>
                    {{ formatPercentage(Math.abs(data.targetPercent - data.currentPercent)) }}
                  </template>
                </Tag>
              </template>
            </Column>

            <Column field="currentValue" header="Current Value" :style="{ minWidth: '5rem' }">
              <template #body="{ data }">
                <div class="current-value">
                  {{ formatCurrency(data.currentValue, data.currency) }}
                </div>
              </template>
            </Column>
          </DataTable>

          <div v-else class="text-center py-6 text-600">
            <i class="pi pi-calculator text-4xl text-300 mb-3"></i>
            <h4 class="text-lg font-semibold text-500 mb-2">No positions available</h4>
            <p class="text-sm line-height-3">Load a portfolio to start rebalancing calculations.</p>
          </div>

          <div class="flex justify-content-center pt-3 border-top-1 surface-border">
            <Button
                label="Calculate Rebalancing Plan"
                icon="pi pi-calculator"
                @click="handleCalculateRebalancing"
                :disabled="!canCalculate"
                :loading="rebalancingState.isCalculating"
                size="large"
                class="min-w-max"
            />
          </div>
        </div>
      </template>
    </Card>

    <Card v-if="rebalancingState.hasCalculated" class="surface-0 border-1 surface-border border-round-xl shadow-1">
      <template #header>
        <div class="flex justify-content-between align-items-center p-4 pb-0 mb-3">
          <h3 class="text-xl font-semibold text-900 m-0 flex align-items-center gap-2">
            <i class="pi pi-list text-primary"></i>
            Rebalancing Plan
          </h3>
          <div class="text-sm font-medium text-600 bg-gray-50 px-3 py-2 border-round">
            {{ formattedActions.length }} actions required
          </div>
        </div>
      </template>

      <template #content>
        <div v-if="formattedActions.length > 0">
          <DataTable
              :value="formattedActions"
              class="border-1 surface-border border-round"
              :scrollable="true"
              scrollHeight="300px"
          >
            <Column field="action" header="Action" :style="{ minWidth: '100px' }">
              <template #body="{ data }">
                <Tag
                    :value="data.actionText"
                    :severity="getActionSeverity(data.action)"
                    class="action-tag"
                >
                  <template #default>
                    <i :class="getActionIcon(data.action)" class="action-icon"></i>
                    {{ data.actionText }}
                  </template>
                </Tag>
              </template>
            </Column>

            <!-- Symbol Column -->
            <Column field="symbol" header="Asset" :style="{ minWidth: '120px' }">
              <template #body="{ data }">
                <div class="symbol-main">{{ data.symbol }}</div>
              </template>
            </Column>

            <!-- Amount Column -->
            <Column field="amount" header="Amount" :style="{ minWidth: '140px' }">
              <template #body="{ data }">
                <div class="amount-cell" :class="data.actionColor">
                  {{ data.formattedAmount }}
                </div>
              </template>
            </Column>

            <!-- Shares Column -->
            <Column field="shares" header="Shares" :style="{ minWidth: '120px' }">
              <template #body="{ data }">
                <div class="shares-cell" :class="data.actionColor">
                  {{ data.formattedShares }}
                </div>
              </template>
            </Column>

            <!-- Current % Column -->
            <Column field="currentPercent" header="Current %" :style="{ minWidth: '120px' }">
              <template #body="{ data }">
                <div class="current-percent">
                  {{ formatPercentage(data.currentPercent) }}
                </div>
              </template>
            </Column>

            <!-- Target % Column -->
            <Column field="targetPercent" header="Target %" :style="{ minWidth: '120px' }">
              <template #body="{ data }">
                <div class="target-percent">
                  {{ formatPercentage(data.targetPercent) }}
                </div>
              </template>
            </Column>

            <!-- Instructions Column -->
            <Column field="instructions" header="Instructions" :style="{ minWidth: '250px' }">
              <template #body="{ data }">
                <div class="instructions-cell">
                  {{ data.action === 'buy' ? 'Buy' : 'Sell' }} {{ data.formattedShares }} shares
                  ({{ data.formattedAmount }}) of {{ data.symbol }}
                </div>
              </template>
            </Column>
          </DataTable>
        </div>

        <!-- No actions needed -->
        <div v-else class="text-center py-6 text-600">
          <i class="pi pi-check-circle text-4xl text-green-500 mb-3"></i>
          <h4 class="text-lg font-semibold text-500 mb-2">Portfolio is already balanced</h4>
          <p class="text-sm line-height-3">No rebalancing actions are required based on your target allocation.</p>
        </div>
      </template>
    </Card>
  </div>
</template>

<style scoped>
/* Custom table styling */
:deep(.p-datatable-thead > tr > th) {
  background: #f8fafc;
  color: #374151;
  font-weight: 600;
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.75rem;
  border-bottom: 2px solid #e2e8f0;
}

:deep(.p-datatable-tbody > tr > td) {
  padding: 0.75rem;
  border-bottom: 1px solid #f1f5f9;
  vertical-align: middle;
}

:deep(.p-datatable-tbody > tr:hover) {
  background: #f8fafc;
}

/* Cell styling */
.symbol-cell {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.symbol-main {
  font-weight: 600;
  color: #1e293b;
  font-size: 0.875rem;
}

.symbol-description {
  font-size: 0.75rem;
  color: #64748b;
  line-height: 1.2;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.current-percent,
.current-value,
.target-percent {
  font-weight: 500;
  color: #374151;
  text-align: right;
}

.current-value {
  font-weight: 600;
  color: #1e293b;
}

.target-input {
  width: 100%;
}

/* Tags */
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

.difference-icon,
.action-icon {
  font-size: 0.625rem;
}

/* Amount and shares cells */
.amount-cell,
.shares-cell {
  font-weight: 600;
  font-size: 0.875rem;
  text-align: right;
}

.amount-cell.success,
.shares-cell.success {
  color: #16a34a;
}

.amount-cell.danger,
.shares-cell.danger {
  color: #dc2626;
}

.instructions-cell {
  font-size: 0.875rem;
  color: #374151;
  line-height: 1.4;
}

/* Animation */
.flex.flex-column.gap-4 {
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

/* Responsive adjustments */
@media (max-width: 1024px) {
  .flex.justify-content-between.align-items-center {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.75rem;
  }
}

@media (max-width: 768px) {
  .symbol-description {
    max-width: 150px;
  }
}

@media (max-width: 480px) {
  .text-xl {
    font-size: 1.125rem;
  }

  .symbol-description {
    max-width: 120px;
  }

  .difference-tag,
  .action-tag {
    font-size: 0.625rem;
    padding: 0.25rem 0.5rem;
  }
}

/* Accessibility */
@media (prefers-reduced-motion: reduce) {
  .flex.flex-column.gap-4 {
    animation: none;
  }
}

/* High contrast mode */
@media (prefers-contrast: high) {
  .border-1 {
    border-width: 2px;
  }

  .difference-tag,
  .action-tag {
    border-width: 2px;
  }
}
</style>
