<script setup lang="ts">
import PortfolioOverview from './PortfolioOverview.vue';
import AssetAllocation from './AssetAllocation.vue';
import PortfolioDetails from './PortfolioDetails.vue';
</script>

<template>
  <div class="p-4 min-h-screen flex flex-column">
    <PortfolioOverview />
    <div class="grid align-items-stretch flex-1">
      <div class="col-12 lg:col-6 flex flex-column h-full">
        <AssetAllocation class="w-full flex-1" />
      </div>
      <div class="col-12 lg:col-6 flex flex-column h-full">
        <PortfolioDetails class="w-full flex-1" />
      </div>
    </div>
  </div>

</template>

<style scoped>

</style>
