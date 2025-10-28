<script setup lang="ts">
import WelcomeScreen from "./components/WelcomeScreen.vue";
import { computed } from 'vue';
import { storeToRefs } from 'pinia';
import DashboardView from './components/dashboard/DashboardView.vue';
import { useAppStore } from './stores/appStore.ts';
import AppHeader from './components/layout/AppHeader.vue';

const appStore = useAppStore();
const { hasPortfolio } = storeToRefs(appStore);

const showWelcomeScreen = computed(() => !hasPortfolio.value);
</script>

<template>
  <main class="min-h-screen w-full">
    <AppHeader v-if="!showWelcomeScreen" />
    <WelcomeScreen v-if="showWelcomeScreen" key="welcome" />
    <DashboardView v-else />
  </main>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html {
  -webkit-text-size-adjust: 100%;
  scroll-behavior: smooth;
}

body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 16px;
  line-height: 1.5;
  color: #1e293b;
  background-color: #f8fafc;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  overflow-x: hidden;
  -webkit-overflow-scrolling: touch;
}
</style>
