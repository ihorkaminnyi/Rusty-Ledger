<template>
  <header class="app-header surface-0 border-bottom-1 surface-border shadow-1 sticky top-0 z-5">
    <div class="max-w-screen-xl mx-auto px-4 py-2 flex align-items-center justify-content-between gap-4">
      <div class="flex align-items-center gap-4 flex-2">
        <div class="flex align-items-center gap-3">
          <i class="pi pi-chart-pie text-primary text-2xl"></i>
          <h1 class="text-2xl font-bold text-900 m-0">Rusty Ledger</h1>
        </div>
        <div class="flex flex-column" v-if="showAccountMeta">
          <span class="text-sm font-semibold text-600" v-if="accountName">{{ accountName }}</span>
          <span class="text-xs text-500" v-if="statementPeriod">
            Period: {{ statementPeriod }}
          </span>
          <span class="text-xs text-500" v-if="statementGenerated">
            Generated: {{ statementGenerated }}
          </span>
          <span class="text-xs text-500" v-if="baseCurrency">
            Base currency: {{ baseCurrency }}
          </span>
        </div>
      </div>

      <div class="flex align-items-center gap-3">
        <Button
            label="Завантажити новий звіт"
            icon="pi pi-upload"
            @click="handleLoadNewReport"
            :loading="isLoading"
            outlined
            size="small"
        />
        <Button
            label="Скинути"
            icon="pi pi-refresh"
            @click="handleReset"
            :loading="isLoading"
            severity="secondary"
            outlined
            size="small"
        />
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';
import { useAppStore } from '../../stores/appStore.ts';
import { useFileUpload } from '../../composables/useFileUpload.ts';

const appStore = useAppStore();
const { portfolio, isLoading } = storeToRefs(appStore);
const { selectFile } = useFileUpload();

const accountInfo = computed(() => portfolio.value?.accountInfo ?? null);
const statementInfo = computed(() => portfolio.value?.statement ?? null);

const accountName = computed(() => accountInfo.value?.name ?? '');
const baseCurrency = computed(() => accountInfo.value?.baseCurrency ?? '');
const statementPeriod = computed(() => statementInfo.value?.period ?? '');
const statementGenerated = computed(() =>
    statementInfo.value?.whenGenerated ? formatReportDate(statementInfo.value.whenGenerated) : '',
);

const showAccountMeta = computed(() => Boolean(
    accountName.value ||
    statementPeriod.value ||
    statementGenerated.value ||
    baseCurrency.value,
));

const formatReportDate = (dateString?: string | null): string => {
  if (!dateString) return '';
  try {
    const cleanDateString = dateString
    .replace(/&quot;/g, '"')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/^["']|["']$/g, '');

    let date = new Date(cleanDateString);

    if (isNaN(date.getTime())) {
      const monthNames = {
        'January': '01', 'February': '02', 'March': '03', 'April': '04',
        'May': '05', 'June': '06', 'July': '07', 'August': '08',
        'September': '09', 'October': '10', 'November': '11', 'December': '12',
      };

      const match = cleanDateString.match(/(\w+)\s+(\d{1,2}),\s+(\d{4})/);
      if (match) {
        const [, monthName, day, year] = match;
        const monthNum = monthNames[monthName as keyof typeof monthNames];
        if (monthNum) {
          date = new Date(`${year}-${monthNum}-${day.padStart(2, '0')}`);
        }
      }
    }

    if (isNaN(date.getTime())) {
      return cleanDateString;
    }

    return date.toLocaleDateString('uk-UA', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  } catch (error) {
    console.error('Date parsing error:', error);
    return dateString;
  }
};

const handleLoadNewReport = async () => {
  try {
    await selectFile();
  } catch (error) {
    console.error('Failed to load new report:', error);
  }
};

const handleReset = () => {
  appStore.clearPortfolio();
  appStore.setRebalance(null);
  appStore.clearAllErrors();
};
</script>

<style scoped>
@media (max-width: 768px) {
  .max-w-screen-xl {
    flex-direction: column;
    align-items: stretch;
    gap: 1rem;
  }

  .flex-1 {
    justify-content: center;
    gap: 1rem;
  }

  .flex.align-items-center.gap-3:first-child {
    justify-content: center;
  }

  .flex.flex-column.gap-1 {
    text-align: center;
  }

  .flex.align-items-center.gap-3:last-child {
    justify-content: center;
    flex-wrap: wrap;
  }
}

@media (max-width: 480px) {
  .px-4 {
    padding-left: 0.75rem;
    padding-right: 0.75rem;
  }

  .flex-1 {
    flex-direction: column;
    gap: 0.5rem;
  }

  .text-2xl {
    font-size: 1.25rem;
  }

  .text-2xl.pi {
    font-size: 1.5rem;
  }

  .flex.align-items-center.gap-3:last-child {
    flex-direction: column;
    gap: 0.5rem;
  }

  .flex.align-items-center.gap-3:last-child :deep(.p-button) {
    width: 100%;
    padding: 0.75rem;
  }
}
</style>
