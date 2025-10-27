<script setup lang="ts">
import PortfolioOverview from './PortfolioOverview.vue';
import AssetAllocation from './AssetAllocation.vue';
import PortfolioDetails from './PortfolioDetails.vue';
</script>

<template>
  <div class="p-4">
    <PortfolioOverview/>
    <div class="grid">
      <div class="col-12 lg:col-6">
        <AssetAllocation/>
      </div>
      <div class="col-12 lg:col-6">
        <PortfolioDetails/>
      </div>
    </div>
  </div>

</template>

<style scoped>

</style>
