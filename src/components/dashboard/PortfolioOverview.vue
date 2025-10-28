<script setup lang="ts">
import { useAppStore } from '../../stores/appStore.ts';
import { storeToRefs } from 'pinia';
import { computed } from 'vue';

const appStore = useAppStore();
const { portfolio, hasPortfolio } = storeToRefs(appStore);

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

const formattedTotalValue = computed(() => {
  if (hasPortfolio.value) {
    const currency = portfolio.value!.accountInfo.baseCurrency;
    return formatCurrency(portfolio.value!.totals.marketValue, currency);
  }
});

const formattedUnrealizedPL = computed(() => {
  if (hasPortfolio.value) {
    const currency = portfolio.value!.accountInfo.baseCurrency;
    return formatCurrency(portfolio.value!.totals.unrealizedPl, currency);
  }
});

const formattedUnrealizedPLPercent = computed(() => {
  return formatPercentage(portfolio.value!.totals.unrealizedPlPercent);
});

const plColorClass = computed(() => {
  if (portfolio.value!.totals.unrealizedPl > 0) return 'profit';
  if (portfolio.value!.totals.unrealizedPl < 0) return 'loss';
  return 'neutral';
});

const plIcon = computed(() => {
  if (portfolio.value!.totals.unrealizedPl > 0) return 'pi pi-arrow-up';
  if (portfolio.value!.totals.unrealizedPl < 0) return 'pi pi-arrow-down';
  return 'pi pi-minus';
});
</script>

<template>
  <div class="w-full mb-2">
    <TransitionGroup name="fade-stagger" tag="div" class="grid align-items-stretch" appear>
      <div class="col-12 md:col-6 lg:col-4" :style="{ '--stagger-index': 0 }" key="market-value">
        <div
            class="surface-0 border-1 surface-border border-round-lg p-3 transition-all transition-duration-300 hover:shadow-3 hover:-translate-y-1 flex flex-column gap-2">
          <div class="flex align-items-center gap-3">
            <div
                class="flex align-items-center justify-content-center w-3rem h-3rem bg-blue-50 text-blue-500 border-round-lg">
              <i class="pi pi-wallet text-xl"></i>
            </div>
            <span class="text-sm font-medium text-600 uppercase">Market Value</span>
          </div>
          <div class="text-3xl font-bold text-900">
            {{ formattedTotalValue }}
          </div>
        </div>
      </div>

      <div class="col-12 md:col-6 lg:col-4" :style="{ '--stagger-index': 1 }" key="unrealized-pl">
        <div
            class="surface-0 border-1 surface-border border-round-lg p-3 transition-all transition-duration-300 hover:shadow-3 hover:-translate-y-1 flex flex-column gap-2">
          <div class="flex align-items-center gap-3">
            <div
                :class="`flex align-items-center justify-content-center w-3rem h-3rem border-round-lg ${plColorClass === 'profit' ? 'bg-teal-50 text-teal-600' : plColorClass === 'loss' ? 'bg-red-50 text-red-600' : 'bg-gray-50 text-gray-500'}`">
              <i :class="`${plIcon} text-xl`"></i>
            </div>
            <div class="flex flex-column">
              <span class="text-sm font-medium text-600 uppercase">Unrealized P&L</span>
              <span
                  :class="`text-base font-medium opacity-80 mt-1 ${plColorClass === 'profit' ? 'text-teal-600' : plColorClass === 'loss' ? 'text-red-600' : 'text-gray-500'}`">
                {{ formattedUnrealizedPLPercent }}
              </span>
            </div>
          </div>
          <div
              :class="`text-3xl font-bold ${plColorClass === 'profit' ? 'text-teal-600' : plColorClass === 'loss' ? 'text-red-600' : 'text-gray-500'}`">
            <div>{{ formattedUnrealizedPL }}</div>
          </div>
        </div>
      </div>

      <div class="col-12 md:col-6 lg:col-4" :style="{ '--stagger-index': 2 }" key="total-positions">
        <div
            class="surface-0 border-1 surface-border border-round-lg p-3 transition-all transition-duration-300 hover:shadow-3 hover:-translate-y-1 flex flex-column gap-2">
          <div class="flex align-items-center gap-3">
            <div
                class="flex align-items-center justify-content-center w-3rem h-3rem bg-purple-50 text-purple-500 border-round-lg">
              <i class="pi pi-chart-pie text-xl"></i>
            </div>
            <span class="text-sm font-medium text-600 uppercase">Total Positions</span>
          </div>
          <div class="text-3xl font-bold text-purple-500">
            {{ hasPortfolio && portfolio!.totals.totalPositions }}
          </div>
        </div>
      </div>
    </TransitionGroup>
  </div>
</template>
