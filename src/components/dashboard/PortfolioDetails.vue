<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';
import Card from 'primevue/card';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import { useAppStore } from '../../stores/appStore';
import RebalancingCalculator from './RebalancingCalculator.vue';

const appStore = useAppStore();
const { portfolio } = storeToRefs(appStore);

const hasPositions = computed(() => (portfolio.value?.positions.length ?? 0) > 0);

const tableRows = computed(() => {
  if (!portfolio.value) return [];
  const currency = portfolio.value.accountInfo.baseCurrency ?? 'USD';
  return portfolio.value.positions.map(position => ({
    symbol: position.symbol,
    quantity: position.quantity,
    price: position.price,
    marketValue: position.marketValue,
    allocationPercent: position.allocationPercent,
    unrealizedPl: position.unrealizedPl,
    roiPercent: position.roiPercent,
    currency,
  }));
});

const formatCurrency = (value: number, currency: string = 'USD') =>
    new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency,
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);

const formatPercent = (value: number) =>
    new Intl.NumberFormat('en-US', {
      style: 'percent',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value / 100);

const getPLClass = (value: number) => {
  if (value > 0) return 'text-teal-600';
  if (value < 0) return 'text-red-500';
  return 'text-600';
};
</script>

<template>
  <Card class="surface-0 border-1 surface-border border-round-2xl shadow-2 overflow-hidden">
    <template #content>
      <div v-if="hasPositions" class="overflow-auto border-1 surface-border border-round-xl">

        <TabView class="full-width-tabs">

          <TabPanel value="details" header="Portfolio Details">
            <DataTable
                :value="tableRows"
                dataKey="symbol"
                :paginator="true"
                :rows="10"
                :rowsPerPageOptions="[10, 25, 50]"
                removableSort
                scrollable
                scrollDirection="both"
                class="text-sm"
            >
              <Column
                  field="symbol"
                  header="Symbol"
                  sortable
                  frozen
                  alignFrozen="left"
                  :style="{ minWidth: '5rem' }"
              />
              <Column field="allocationPercent" header="Allocation %" sortable :style="{ minWidth: '5rem' }">
                <template #body="{ data }">
                  {{ formatPercent(data.allocationPercent) }}
                </template>
              </Column>
              <Column field="unrealizedPl" header="Unrealized P&L" sortable :style="{ minWidth: '5rem' }">
                <template #body="{ data }">
              <span :class="getPLClass(data.unrealizedPl)">
                {{ formatCurrency(data.unrealizedPl, data.currency) }}
              </span>
                </template>
              </Column>
              <Column field="roiPercent" header="ROI %" sortable :style="{ minWidth: '5rem' }">
                <template #body="{ data }">
              <span :class="getPLClass(data.roiPercent)">
                {{ formatPercent(data.roiPercent) }}
              </span>
                </template>
              </Column>
              <Column field="marketValue" header="Market Value" sortable :style="{ minWidth: '5rem' }">
                <template #body="{ data }">
                  {{ formatCurrency(data.marketValue, data.currency) }}
                </template>
              </Column>
              <Column field="quantity" header="Quantity" sortable :style="{ minWidth: '5rem' }">
                <template #body="{ data }">
                  {{ data.quantity.toLocaleString(undefined, { maximumFractionDigits: 2 }) }}
                </template>
              </Column>
              <Column field="price" header="Price" sortable :style="{ minWidth: '5rem' }">
                <template #body="{ data }">
                  {{ formatCurrency(data.price, data.currency) }}
                </template>
              </Column>
            </DataTable>
          </TabPanel>

          <TabPanel value="rebalancing" header="Rebalancing Calculator">
            <RebalancingCalculator />
          </TabPanel>
        </TabView>
      </div>

      <div v-else class="text-center py-6 text-600">
        <i class="pi pi-table text-4xl text-300 mb-3"></i>
        <h4 class="text-lg font-semibold text-500 mb-2">No positions to display</h4>
        <p class="text-sm line-height-3">Upload a portfolio CSV file to see your detailed positions.</p>
      </div>
    </template>
  </Card>
</template>

<style scoped>
.portfolio-table {
  border-spacing: 0;
  border-collapse: separate;
}

.portfolio-table th {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 600;
}

.portfolio-table tbody tr:hover {
  background-color: rgba(148, 163, 184, 0.08);
}

:deep(.full-width-tabs .p-tabview-nav) {
  display: flex;
  width: 100%;
}

:deep(.full-width-tabs .p-tabview-nav li) {
  flex: 1 1 0;
}

:deep(.full-width-tabs .p-tabview-nav li .p-tabview-nav-link) {
  width: 100%;
  justify-content: center;
}

</style>
