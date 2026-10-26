<script setup lang="ts">
import WelcomeScreen from "./components/welcome/WelcomeScreen.vue";
import { computed, onMounted, watch } from "vue";
import { storeToRefs } from "pinia";
import DashboardView from "./components/dashboard/DashboardView.vue";
import { useAppStore } from "./stores/appStore.ts";
import AppHeader from "./components/layout/AppHeader.vue";
import SavedReportsModal from "./components/layout/SavedReportsModal.vue";
import { usePortfolioReports } from "./composables/usePortfolioReports.ts";
import { useToast } from "primevue/usetoast";

const appStore = useAppStore();
const { hasPortfolio, errors, activeReportId, portfolioRevision, isSavedReportsOpen, isLoading } = storeToRefs(appStore);

const showWelcomeScreen = computed(() => !hasPortfolio.value);

const severityByScope: Record<string, "error" | "warn" | "info"> = {
    upload: "error",
    rebalancing: "error",
    tauri: "warn",
    portfolio: "error",
    reports: "error",
};

const summaryByScope: Record<string, string> = {
    upload: "Import Error",
    rebalancing: "Rebalancing Issue",
    tauri: "Desktop Integration",
    portfolio: "Portfolio Issue",
    reports: "Saved Reports",
    global: "Application Error",
};

const toast = useToast();
const {
    reports, isLoadingList, listError, operationError, pendingAction,
    confirmation, noSavedReport, loadReports, setReportsVisible,
    openLatestReport, openReport, requestDelete, requestClearAll,
    confirmDeletion, cancelConfirmation,
} = usePortfolioReports((severity, detail) => {
    toast.add({ severity, summary: 'Saved Reports', detail, life: 4000 });
});

watch(
    errors,
    (current, previous = []) => {
        const previousIds = new Set(previous.map((error) => error.id));
        current.forEach((error) => {
            if (previousIds.has(error.id)) {
                return;
            }
            const summary = summaryByScope[error.scope] ?? "Error";
            const decoratedSummary = error.code
                ? `${summary} (${error.code})`
                : summary;
            toast.add({
                severity: severityByScope[error.scope] ?? "error",
                summary: decoratedSummary,
                detail: error.message,
                life: 5000,
            });
        });
    },
    { deep: true },
);

onMounted(() => {
    void openLatestReport();
});

</script>

<template>
    <Transition name="screen-fade" mode="out-in">
        <main class="min-h-screen w-full">
            <Toast />
            <AppHeader v-if="!showWelcomeScreen"
                @open-latest-report="openLatestReport"
                @open-saved-reports="setReportsVisible(true)" />
            <WelcomeScreen
                v-if="showWelcomeScreen"
                key="welcome"
                :no-saved-report="noSavedReport"
                @open-latest-report="openLatestReport"
                @open-saved-reports="setReportsVisible(true)"
            />
            <DashboardView v-else :key="portfolioRevision" />
            <SavedReportsModal
                :model-value="isSavedReportsOpen"
                :reports="reports"
                :active-report-id="activeReportId"
                :loading="isLoadingList"
                :busy="isLoading"
                :list-error="listError"
                :operation-error="operationError"
                :pending-action="pendingAction"
                :confirmation="confirmation"
                @update:model-value="setReportsVisible"
                @open-report="openReport"
                @request-delete="requestDelete"
                @request-clear-all="requestClearAll"
                @confirm-deletion="confirmDeletion"
                @cancel-confirmation="cancelConfirmation"
                @retry="loadReports"
            />
        </main>
    </Transition>
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
    font-family:
        "Inter",
        -apple-system,
        BlinkMacSystemFont,
        "Segoe UI",
        Roboto,
        sans-serif;
    font-size: 16px;
    line-height: 1.5;
    color: #1e293b;
    background-color: #f8fafc;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    overflow-x: hidden;
    -webkit-overflow-scrolling: touch;
}

.screen-fade-enter-active,
.screen-fade-leave-active {
    transition:
        opacity 0.4s ease,
        transform 0.4s ease;
}

.screen-fade-enter-from,
.screen-fade-leave-to {
    opacity: 0;
    transform: translateY(1.25rem);
}

.fade-in-enter-active,
.fade-in-appear-active,
.fade-in-leave-active {
    transition: opacity 0.35s ease;
}

.fade-in-enter-from,
.fade-in-appear-from,
.fade-in-leave-to {
    opacity: 0;
}

.fade-up-enter-active,
.fade-up-appear-active,
.fade-up-leave-active {
    transition:
        opacity 0.45s ease,
        transform 0.45s ease;
}

.fade-up-enter-from,
.fade-up-appear-from,
.fade-up-leave-to {
    opacity: 0;
    transform: translateY(1.5rem);
}

.scale-in-enter-active,
.scale-in-appear-active,
.scale-in-leave-active {
    transition:
        opacity 0.4s ease,
        transform 0.4s ease;
}

.scale-in-enter-from,
.scale-in-appear-from,
.scale-in-leave-to {
    opacity: 0;
    transform: scale(0.96);
}

.fade-stagger-enter-active,
.fade-stagger-appear-active {
    transition:
        opacity 0.45s ease,
        transform 0.45s ease;
    transition-delay: calc(var(--stagger-index, 0) * 80ms);
}

.fade-stagger-enter-from,
.fade-stagger-appear-from,
.fade-stagger-leave-to {
    opacity: 0;
    transform: translateY(1.25rem);
}

.fade-stagger-leave-active {
    transition:
        opacity 0.3s ease,
        transform 0.3s ease;
}
</style>
