<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import { TauriService } from '../../services/tauri';

interface Props {
  modelValue: boolean;
}

const props = defineProps<Props>();

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
}

const emit = defineEmits<Emits>();

const appVersion = ref('0.0.1');

const isVisible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
});

onMounted(async () => {
  try {
    appVersion.value = await TauriService.getAppVersion();
  } catch (error) {
    console.warn('Failed to get app version:', error);
  }
});

const closeModal = () => {
  isVisible.value = false;
};

const openGitHub = () => {
  void TauriService.openExternalLink('https://github.com/ihorkaminnyi/Rusty-Ledger');
};

const openIssues = () => {
  void TauriService.openExternalLink('https://github.com/ihorkaminnyi/Rusty-Ledger/issues/new/choose');
};
</script>

<template>
  <Dialog
      v-model:visible="isVisible"
      modal
      :closable="false"
      :draggable="false"
      class="about-modal"
      :style="{ width: '500px' }"
  >
    <div class="flex flex-column gap-4">
      <div class="text-center pb-3 border-bottom-1 surface-border">
        <div class="flex align-items-center justify-content-center gap-3 mb-3">
          <img src="/64x64.png" alt="Rusty Ledger logo" width="64" height="64" />
          <div class="text-left">
            <h2 class="text-2xl font-bold text-900 m-0">Rusty Ledger</h2>
            <p class="text-sm text-600 m-0 mt-1">Version {{ appVersion }}</p>
          </div>
        </div>

        <p class="text-sm text-600 line-height-3 m-0">
          A privacy-first desktop app for portfolio analysis, built with Tauri + Vue.js.
          Keep every insight fully on your machine.
        </p>
      </div>

      <div class="pb-3 border-bottom-1 surface-border text-sm text-600 line-height-3">
        <h3 class="flex align-items-center gap-2 text-base font-semibold text-900 m-0 mb-3">
          <i class="pi pi-user text-primary"></i>
          Author
        </h3>
        <p class="m-0">
          Rusty Ledger currently supports account statements from Interactive Brokers only,
          because it is the largest and most reliable U.S. broker available to Ukrainians.
        </p>
        <p class="m-0 mt-2">
          Download the latest CSV statement from your Interactive Brokers account via
          <strong>Performance &amp; Reports → Statements → Activity Statement → Download CSV</strong>
          (a daily statement is sufficient; ensure the language is set to English), then
          upload that CSV file into the app.
        </p>
      </div>

      <div class="pb-3 border-bottom-1 surface-border">
        <h3 class="flex align-items-center gap-2 text-base font-semibold text-900 m-0 mb-3">
          <i class="pi pi-external-link text-primary"></i>
          Links
        </h3>
        <div class="grid">
          <div class="col-6">
            <Button
                label="GitHub Repository"
                icon="pi pi-github"
                @click="openGitHub"
                outlined
                class="w-full"
                size="small"
            />
          </div>
          <div class="col-6">
            <Button
                label="Report an Issue"
                icon="pi pi-exclamation-triangle"
                @click="openIssues"
                outlined
                class="w-full"
                size="small"
            />
          </div>
        </div>
      </div>

      <div class="pb-3 border-bottom-1 surface-border">
        <h3 class="flex align-items-center gap-2 text-base font-semibold text-900 m-0 mb-3">
          <i class="pi pi-shield text-primary"></i>
          Privacy & Security
        </h3>
        <div class="flex flex-column gap-2">
          <div class="flex align-items-center gap-2 text-sm text-600">
            <i class="pi pi-check text-green-500 text-sm"></i>
            <span>All data is processed locally on your computer</span>
          </div>
          <div class="flex align-items-center gap-2 text-sm text-600">
            <i class="pi pi-check text-green-500 text-sm"></i>
            <span>No network requests while processing your data</span>
          </div>
          <div class="flex align-items-center gap-2 text-sm text-600">
            <i class="pi pi-check text-green-500 text-sm"></i>
            <span>Data is cleared automatically when you close the app</span>
          </div>
          <div class="flex align-items-center gap-2 text-sm text-600">
            <i class="pi pi-check text-green-500 text-sm"></i>
            <span>Open-source code ensures full transparency</span>
          </div>
        </div>
      </div>

      <div>
        <h3 class="flex align-items-center gap-2 text-base font-semibold text-900 m-0 mb-3">
          <i class="pi pi-cog text-primary"></i>
          Technical Details
        </h3>
        <div class="flex flex-column gap-2">
          <div class="flex justify-content-between align-items-center text-sm">
            <span class="text-600 font-medium">Frontend:</span>
            <span class="text-900 font-semibold">Vue.js 3 + PrimeVue</span>
          </div>
          <div class="flex justify-content-between align-items-center text-sm">
            <span class="text-600 font-medium">Backend:</span>
            <span class="text-900 font-semibold">Rust + Tauri</span>
          </div>
          <div class="flex justify-content-between align-items-center text-sm">
            <span class="text-600 font-medium">Visualizations:</span>
            <span class="text-900 font-semibold">Chart.js</span>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-content-end">
        <Button
            label="Close"
            @click="closeModal"
        />
      </div>
    </template>
  </Dialog>
</template>

<style scoped>
@media (max-width: 600px) {
  :deep(.p-dialog) {
    width: 95vw !important;
    margin: 1rem;
  }

  .flex.align-items-center.justify-content-center {
    flex-direction: column;
    gap: 0.5rem;
  }

  .text-left {
    text-align: center;
  }

  .grid .col-6 {
    width: 100%;
  }

  .flex.justify-content-between.align-items-center {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
  }
}
</style>
