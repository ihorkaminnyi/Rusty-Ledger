<script setup lang="ts">
import PortfolioOverview from './PortfolioOverview.vue';
import AssetAllocation from './AssetAllocation.vue';
import PortfolioDetails from './PortfolioDetails.vue';
</script>

<template>
  <div class="p-4 min-h-screen flex flex-column">
    <Transition name="fade-up" appear>
      <PortfolioOverview />
    </Transition>

    <TransitionGroup
        name="fade-stagger"
        tag="div"
        class="grid align-items-stretch flex-1"
        appear
    >
      <div
          key="asset-allocation"
          class="col-12 lg:col-6 flex flex-column h-full"
          :style="{ '--stagger-index': 0 }"
      >
        <AssetAllocation class="w-full flex-1" />
      </div>
      <div
          key="portfolio-details"
          class="col-12 lg:col-6 flex flex-column h-full"
          :style="{ '--stagger-index': 1 }"
      >
        <PortfolioDetails class="w-full flex-1" />
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>

</style>
