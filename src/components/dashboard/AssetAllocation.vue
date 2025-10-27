<script setup lang="ts">
import { computed, ref } from 'vue';
import { storeToRefs } from 'pinia';
import Card from 'primevue/card';
import {
  Chart as ChartJS,
  ArcElement,
  Tooltip,
  Legend,
  type ChartData,
  type ChartOptions,
} from 'chart.js';
import { Pie } from 'vue-chartjs';
import { useAppStore } from '../../stores/appStore';

ChartJS.register(ArcElement, Tooltip, Legend);

const appStore = useAppStore();
const { portfolio } = storeToRefs(appStore);

const chartRef = ref();

const generateColors = (count: number): string[] => {
  const baseColors = [
    '#3b82f6',
    '#ef4444',
    '#10b981',
    '#f59e0b',
    '#8b5cf6',
    '#06b6d4',
    '#f97316',
    '#84cc16',
    '#ec4899',
    '#6b7280',
    '#14b8a6',
    '#f43f5e',
  ];

  if (count <= baseColors.length) {
    return baseColors.slice(0, count);
  }

  const colors = [...baseColors];
  for (let i = baseColors.length; i < count; i++) {
    const hue = (i * 137.508) % 360;
    colors.push(`hsl(${hue}, 70%, 50%)`);
  }
  return colors;
};

const chartPositions = computed(() => {
  const positions = portfolio.value?.positions ?? [];
  const totalValue = portfolio.value?.totals.marketValue ?? 0;

  if (!positions.length || totalValue === 0) return [];

  const enriched = positions.map(position => ({
    ...position,
    calculatedPercent: (position.marketValue / totalValue) * 100,
  }));

  const threshold = 2;
  const large = enriched.filter(p => p.calculatedPercent >= threshold);
  const small = enriched.filter(p => p.calculatedPercent < threshold);

  if (!small.length) {
    return large.sort((a, b) => b.calculatedPercent - a.calculatedPercent);
  }

  const othersValue = small.reduce((sum, p) => sum + p.marketValue, 0);
  const othersPercent = small.reduce((sum, p) => sum + p.calculatedPercent, 0);
  const othersUnrealized = small.reduce((sum, p) => sum + p.unrealizedPl, 0);

  const others = {
    symbol: 'Others',
    quantity: small.length,
    price: 0,
    marketValue: othersValue,
    allocationPercent: othersPercent,
    unrealizedPl: othersUnrealized,
    roiPercent: 0,
    calculatedPercent: othersPercent,
  };

  return [...large, others].sort((a, b) => b.calculatedPercent - a.calculatedPercent);
});

const chartData = computed<ChartData<'pie'>>(() => {
  const data = chartPositions.value;
  const colors = generateColors(data.length);

  return {
    labels: data.map(p => p.symbol),
    datasets: [
      {
        data: data.map(p => p.calculatedPercent),
        backgroundColor: colors,
        borderColor: colors,
        borderWidth: 2,
        hoverBorderWidth: 3,
        hoverOffset: 8,
      },
    ],
  };
});

const chartOptions = computed<ChartOptions<'pie'>>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      position: 'right',
      labels: {
        padding: 20,
        usePointStyle: true,
        pointStyle: 'circle',
        font: {
          size: 12,
          family: 'Inter, sans-serif',
        },
        generateLabels: (chart) => {
          const { labels, datasets } = chart.data;
          if (!labels?.length || !datasets.length) return [];

          return labels.map((label, index) => {
            const position = chartPositions.value[index];
            const percentage = position?.calculatedPercent.toFixed(1) ?? '0.0';
            const backgroundColor = datasets[0].backgroundColor;
            const borderColor = datasets[0].borderColor;

            return {
              text: `${label} (${percentage}%)`,
              fillStyle: Array.isArray(backgroundColor) ? backgroundColor[index] : backgroundColor as string,
              strokeStyle: Array.isArray(borderColor) ? borderColor[index] : borderColor as string,
              lineWidth: 2,
              hidden: false,
              index,
              pointStyle: 'circle',
            };
          });
        },
      },
    },
    tooltip: {
      backgroundColor: 'rgba(0, 0, 0, 0.8)',
      titleColor: 'white',
      bodyColor: 'white',
      borderColor: 'rgba(255, 255, 255, 0.1)',
      borderWidth: 1,
      cornerRadius: 8,
      displayColors: true,
      callbacks: {
        title: (context) => {
          const position = chartPositions.value[context[0].dataIndex];
          return position?.symbol ?? 'Position';
        },
        label: (context) => {
          const position = chartPositions.value[context.dataIndex];
          if (!position) return '';

          const currency = portfolio.value?.accountInfo.baseCurrency ?? 'USD';
          const value = formatCurrency(position.marketValue, currency);
          const percentage = position.calculatedPercent.toFixed(2);

          return [
            `Value: ${value}`,
            `Allocation: ${percentage}%`,
            `Unrealized P/L: ${formatCurrency(position.unrealizedPl, currency)}`,
          ];
        },
      },
    },
  },
  elements: {
    arc: {
      borderWidth: 2,
    },
  },
  interaction: {
    intersect: false,
    mode: 'index',
  },
  animation: {
    animateRotate: true,
    animateScale: false,
    duration: 1000,
    easing: 'easeOutQuart',
  },
}));

const formatCurrency = (value: number, currency: string = 'USD'): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(value);
};
</script>

<template>
  <Card class="surface-0 border-1 surface-border border-round-xl shadow-2 overflow-hidden">
    <template #header>
      <div class="flex justify-content-between align-items-center p-3 pb-0 mb-2 header-compact">
        <h3 class="text-xl font-semibold text-900 m-0 flex align-items-center gap-2">
          <i class="pi pi-chart-pie text-primary"></i>
          Asset Allocation
        </h3>
        <div class="text-sm font-medium text-600 bg-gray-50 px-3 py-1 border-round">
          Total: {{ formatCurrency(portfolio?.totals.marketValue ?? 0, portfolio?.accountInfo.baseCurrency) }}
        </div>
      </div>
    </template>

    <template #content>
      <div class="flex flex-column gap-4" v-if="portfolio?.positions.length">
        <div class="relative h-22rem w-full">
          <Pie
              ref="chartRef"
              :data="chartData"
              :options="chartOptions"
          />
        </div>
      </div>

      <div v-else class="text-center py-6 text-600">
        <i class="pi pi-chart-pie text-4xl text-300 mb-3"></i>
        <h4 class="text-lg font-semibold text-500 mb-2">No positions to display</h4>
        <p class="text-sm line-height-3">Upload a portfolio CSV file to see your asset allocation.</p>
      </div>
    </template>
  </Card>
</template>

<style scoped>
.relative.h-25rem {
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

.header-compact {
  flex-wrap: wrap;
  gap: 0.75rem;
}

@media (max-width: 1024px) {
  .h-25rem {
    height: 22rem;
  }
  
  .header-compact {
    justify-content: space-between;
    gap: 0.5rem;
  }
}

@media (max-width: 768px) {
  .h-25rem {
    height: 19rem;
  }

  .flex.justify-content-center.gap-4 {
    flex-direction: column;
    gap: 1rem;
  }
}

@media (max-width: 480px) {
  .h-25rem {
    height: 16rem;
  }

  .text-xl {
    font-size: 1.125rem;
  }

  .text-sm {
    font-size: 0.75rem;
  }
}

@media (prefers-reduced-motion: reduce) {
  .relative.h-25rem {
    animation: none;
  }
}

@media (prefers-contrast: high) {
  .border-1 {
    border-width: 2px;
  }
}
</style>
