<script setup lang="ts">
import { useFileUpload } from '../composables/useFileUpload.ts';

const {
  isDragging,
  handleDragEnter,
  handleDragOver,
  handleDragLeave,
  handleDrop,
  selectFile,
} = useFileUpload();

</script>

<template>
  <div class="min-h-screen flex flex-column justify-content-start align-items-center p-4 bg-gray-50">
    <div class="text-center mb-4">
      <div class="flex align-items-center justify-content-center gap-3 mb-1">
        <i class="pi pi-chart-pie text-6xl text-primary"></i>
        <h1 class="text-5xl font-bold text-900 mb-2">Rusty Ledger</h1>
      </div>
      <p class="text-xl font-medium text-700">
        Приватний аналіз інвестиційного портфелю
      </p>
    </div>

    <div class="w-full max-w-4xl mb-4">
      <div class="grid align-content-end">
        <div class="col-12 md:col-4">
          <div class="text-center p-1">
            <div
                class="flex align-items-center justify-content-center w-5rem h-5rem bg-blue-50 text-primary border-round-2xl mx-auto mb-4">
              <i class="pi pi-chart-bar text-3xl"></i>
            </div>
            <h4 class="text-lg font-semibold text-900">Ключові дані</h4>
          </div>
        </div>
        <div class="col-12 md:col-4">
          <div class="text-center p-1">
            <div
                class="flex align-items-center justify-content-center w-5rem h-5rem bg-purple-50 text-purple-500 border-round-2xl mx-auto mb-4">
              <i class="pi pi-chart-pie text-3xl"></i>
            </div>
            <h4 class="text-lg font-semibold text-900">Розподіл активів</h4>
          </div>
        </div>
        <div class="col-12 md:col-4">
          <div class="text-center p-1">
            <div
                class="flex align-items-center justify-content-center w-5rem h-5rem bg-green-50 text-green-500 border-round-2xl mx-auto mb-4">
              <i class="pi pi-calculator text-3xl"></i>
            </div>
            <h4 class="text-lg font-semibold text-900">Ребалансування</h4>
          </div>
        </div>
      </div>
    </div>

    <div class="w-full max-w-30rem mb-6">
      <div
          class="upload-zone surface-0 border-2 border-dashed border-300 border-round-xl p-4 text-center transition-all transition-duration-300 cursor-pointer hover:border-primary hover:bg-primary-25"
          :class="{ 'is-dragging': isDragging }"
          @dragenter.prevent="handleDragEnter"
          @dragover.prevent="handleDragOver"
          @dragleave.prevent="handleDragLeave"
          @drop.prevent="handleDrop"
          @click="selectFile"
      >
        <div class="flex flex-column align-items-center gap-5">
          <i class="pi pi-cloud-upload text-7xl text-400"></i>
          <h3 class="text-xl font-semibold text-900 m-0">Перетягніть ваш CSV-звіт сюди</h3>
          <p class="text-base text-600 m-0">
            <span class="hidden md:inline">...або натисніть щоб обрати файл</span>
          </p>
        </div>
      </div>
    </div>

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

  </div>
</template>

<style scoped>
.upload-zone.is-dragging {
  border-color: var(--primary-color, #6366f1);
  background-color: rgba(99, 102, 241, 0.08);
}
</style>
