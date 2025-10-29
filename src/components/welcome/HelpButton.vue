<script setup lang="ts">
interface Emits {
  (e: 'open-about'): void;
}

const emit = defineEmits<Emits>();

const handleClick = () => {
  emit('open-about');
};
</script>

<template>
  <div class="fixed bottom-0 right-0 m-4 z-4">
    <Button
        icon="pi pi-question"
        @click="handleClick"
        rounded
        :aria-label="'About Rusty Ledger'"
        v-tooltip.left="'About Rusty Ledger'"
        class="w-3rem h-3rem shadow-4"
    />
  </div>
</template>

<style scoped>
:deep(.p-button:hover) {
  transform: translateY(-2px);
}

:deep(.p-button:active) {
  transform: translateY(0);
}

:deep(.p-button .p-button-icon) {
  font-size: 1.25rem;
}

@media (max-width: 768px) {
  .fixed.bottom-0.right-0.m-4 {
    margin: 1.5rem;
  }

  .w-3rem.h-3rem {
    width: 2.75rem;
    height: 2.75rem;
  }

  :deep(.p-button .p-button-icon) {
    font-size: 1.125rem;
  }
}

@media (max-width: 480px) {
  .fixed.bottom-0.right-0.m-4 {
    margin: 1rem;
  }

  .w-3rem.h-3rem {
    width: 2.5rem;
    height: 2.5rem;
  }

  :deep(.p-button .p-button-icon) {
    font-size: 1rem;
  }
}
</style>
