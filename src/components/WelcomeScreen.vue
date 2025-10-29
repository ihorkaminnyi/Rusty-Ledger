<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';

import { useFileUpload } from '../composables/useFileUpload.ts';
import { useAppStore } from '../stores/appStore.ts';
import Skeleton from 'primevue/skeleton';
import Message from 'primevue/message';

interface FeatureHighlight {
  key: string;
  icon: string;
  wrapperClass: string;
  title: string;
  titleClass: string;
}

const featureHighlights: FeatureHighlight[] = [
  {
    key: 'core-data',
    icon: 'pi pi-chart-bar',
    wrapperClass: 'bg-blue-50 text-primary',
    title: 'Ключові дані',
    titleClass: 'text-900',
  },
  {
    key: 'distribution',
    icon: 'pi pi-chart-pie',
    wrapperClass: 'bg-purple-50 text-purple-500',
    title: 'Розподіл активів',
    titleClass: 'text-900',
  },
  {
    key: 'rebalance',
    icon: 'pi pi-calculator',
    wrapperClass: 'bg-green-50 text-green-500',
    title: 'Ребалансування',
    titleClass: 'text-900',
  },
];

const {
  isDragging,
  handleDragEnter,
  handleDragOver,
  handleDragLeave,
  handleDrop,
  selectFile,
} = useFileUpload();

const appStore = useAppStore();
const { isLoading } = storeToRefs(appStore);

const uploadErrors = computed(() => appStore.errorsByScope('upload'));

const handleDismissUploadError = (id: string) => {
  appStore.dismissError(id);
};

const handleUploadClick = () => {
  if (!isLoading.value) {
    void selectFile();
  }
};

</script>

<template>
  <div class="min-h-screen flex flex-column justify-content-start align-items-center p-4 bg-gray-50">
    <Transition name="fade-up" appear>
      <div class="text-center mb-4">
        <div class="flex align-items-center justify-content-center gap-3 mb-1">
          <i class="pi pi-chart-pie text-6xl text-primary"></i>
          <h1 class="text-5xl font-bold text-900 mb-2">Rusty Ledger</h1>
        </div>
        <p class="text-xl font-medium text-700">
          Приватний аналіз інвестиційного портфелю
        </p>
      </div>
    </Transition>

    <div class="w-full max-w-4xl mb-4">
      <TransitionGroup name="fade-stagger" appear tag="div" class="grid align-content-end">
        <div
            v-for="(feature, index) in featureHighlights"
            :key="feature.key"
            class="col-12 md:col-4"
            :style="{ '--stagger-index': index }"
        >
          <div class="text-center p-1">
            <div
                class="flex align-items-center justify-content-center w-5rem h-5rem border-round-2xl mx-auto mb-4"
                :class="feature.wrapperClass"
            >
              <i :class="`${feature.icon} text-3xl`"></i>
            </div>
            <h4 class="text-lg font-semibold" :class="feature.titleClass">{{ feature.title }}</h4>
          </div>
        </div>
      </TransitionGroup>
    </div>

    <Transition name="scale-in" appear>
      <div class="w-full max-w-30rem mb-6">
        <div
            class="upload-zone surface-0 border-2 border-dashed border-300 border-round-xl p-4 text-center transition-all transition-duration-300 cursor-pointer hover:border-primary hover:bg-primary-25"
            :class="{
              'is-dragging': isDragging,
              'is-loading': isLoading,
            }"
            @dragenter.prevent="handleDragEnter"
            @dragover.prevent="handleDragOver"
            @dragleave.prevent="handleDragLeave"
            @drop.prevent="handleDrop"
            @click="handleUploadClick"
            :aria-busy="isLoading"
        >
          <Transition name="fade-in" mode="out-in">
            <div v-if="!isLoading" key="upload-content" class="flex flex-column align-items-center gap-5">
              <i class="pi pi-cloud-upload text-7xl text-400"></i>
              <h3 class="text-xl font-semibold text-900 m-0">Перетягніть ваш CSV-звіт сюди</h3>
              <p class="text-base text-600 m-0">
                <span class="hidden md:inline">...або натисніть щоб обрати файл</span>
              </p>
            </div>
            <div v-else key="upload-skeleton" class="flex flex-column align-items-center gap-4 w-full">
              <Skeleton shape="circle" size="5rem" animation="wave"/>
              <Skeleton width="80%" height="1.5rem" animation="wave"/>
              <Skeleton width="60%" height="1rem" animation="wave"/>
            </div>
          </Transition>
        </div>
      </div>
    </Transition>

    <Transition name="fade-in">
      <div v-if="uploadErrors.length" class="w-full max-w-30rem mb-4">
        <TransitionGroup name="fade-in" tag="div" class="flex flex-column gap-2">
          <Message
              v-for="error in uploadErrors"
              :key="error.id"
              severity="error"
              :closable="true"
              @close="handleDismissUploadError(error.id)"
          >
            <div class="flex flex-column gap-1">
            <span class="font-semibold text-sm text-900">
              Неможливо обробити файл
            </span>
              <span class="text-sm line-height-3 text-700">
              {{ error.message }}
            </span>
            </div>
          </Message>
        </TransitionGroup>
      </div>
    </Transition>

    <Transition name="fade-in" appear>
      <div class="w-full max-w-30rem">
        <div class="flex align-items-center gap-4 p-4 bg-teal-50 border-1 border-teal-200 border-round-lg">
          <div
              class="flex align-items-center justify-content-center w-3rem h-3rem bg-teal-100 text-teal-600 border-round-lg flex-shrink-0">
            <i class="pi pi-shield text-xl"></i>
          </div>
          <div class="text-sm text-teal-800 line-height-3">
            <strong>Повна приватність:</strong>
            Всі дані обробляються локально на вашому комп'ютері.
            Ніяка інформація не передається на зовнішні сервери.
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.upload-zone.is-dragging {
  border-color: var(--primary-color, #6366f1);
  background-color: rgba(99, 102, 241, 0.08);
}

.upload-zone.is-loading {
  position: relative;
  pointer-events: none;
  opacity: 0.85;
}
</style>
