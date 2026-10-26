<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import Dialog from 'primevue/dialog';
import DataTable from 'primevue/datatable';
import Column from 'primevue/column';
import Button from 'primevue/button';
import Tag from 'primevue/tag';
import Message from 'primevue/message';
import type { StoredReportListItem } from '../../types/portfolio.ts';
import type { ReportConfirmation, ReportPendingAction } from '../../composables/usePortfolioReports.ts';

const props = defineProps<{
    modelValue: boolean;
    reports: StoredReportListItem[];
    activeReportId: number | null;
    loading: boolean;
    busy: boolean;
    listError: string | null;
    operationError: string | null;
    pendingAction: ReportPendingAction | null;
    confirmation: ReportConfirmation | null;
}>();

const emit = defineEmits<{
    'update:modelValue': [visible: boolean];
    openReport: [id: number];
    requestDelete: [report: StoredReportListItem];
    requestClearAll: [];
    confirmDeletion: [];
    cancelConfirmation: [];
    retry: [];
}>();

const first = ref(0);
const actionsDisabled = computed(() => props.busy || props.loading || Boolean(props.confirmation));
watch(() => props.reports.length, () => { first.value = 0; });
watch(() => props.modelValue, (visible) => { if (visible) first.value = 0; });

const formatDate = (value: string): string => {
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
};
</script>

<template>
    <Dialog
        :visible="modelValue"
        header="Saved reports"
        modal
        :draggable="false"
        :closable="!busy && !confirmation"
        :close-on-escape="!busy && !confirmation"
        :dismissable-mask="false"
        :style="{ width: 'min(1080px, 96vw)', maxHeight: '90vh' }"
        @update:visible="emit('update:modelValue', $event)"
    >
        <div class="flex flex-column gap-3">
            <div class="flex flex-wrap align-items-center justify-content-between gap-3">
                <p class="text-600 m-0" aria-live="polite">
                    {{ loading ? 'Loading saved reports…' : `${reports.length} saved report${reports.length === 1 ? '' : 's'}` }}
                </p>
                <Button
                    label="Clear all reports"
                    icon="pi pi-trash"
                    severity="danger"
                    outlined
                    size="small"
                    :disabled="actionsDisabled || Boolean(listError) || reports.length === 0"
                    @click="emit('requestClearAll')"
                />
            </div>

            <Message v-if="operationError && !confirmation" severity="error" :closable="false">
                {{ operationError }}
            </Message>
            <div v-if="listError" class="flex flex-column gap-3">
                <Message severity="error" :closable="false">{{ listError }}</Message>
                <Button label="Retry" icon="pi pi-refresh" severity="secondary" outlined class="align-self-start"
                    :disabled="busy || loading" @click="emit('retry')" />
            </div>
            <DataTable
                v-else
                :value="reports"
                data-key="id"
                :loading="loading"
                paginator
                :rows="10"
                :rows-per-page-options="[10, 25, 50]"
                :first="first"
                scrollable
                scroll-height="50vh"
                :table-style="{ minWidth: '850px' }"
                @page="first = $event.first"
            >
                <template #empty>
                    <div class="text-center p-4 text-600">
                        {{ loading ? 'Loading saved reports…' : 'No saved reports. Import a CSV report to get started.' }}
                    </div>
                </template>
                <Column field="id" header="Report">
                    <template #body="{ data }">
                        <div class="flex flex-column align-items-start gap-2">
                            <span class="font-semibold">#{{ data.id }}</span>
                            <Tag v-if="data.id === activeReportId" value="Current" severity="info" />
                        </div>
                    </template>
                </Column>
                <Column field="accountName" header="Account">
                    <template #body="{ data }">{{ data.accountName || '—' }}</template>
                </Column>
                <Column field="statementPeriod" header="Statement period">
                    <template #body="{ data }">{{ data.statementPeriod || '—' }}</template>
                </Column>
                <Column field="createdAt" header="Imported">
                    <template #body="{ data }">{{ formatDate(data.createdAt) }}</template>
                </Column>
                <Column field="totalPositions" header="Positions" />
                <Column header="Actions">
                    <template #body="{ data }">
                        <div class="flex gap-2">
                            <Button label="Open" icon="pi pi-folder-open" severity="info" size="small" outlined
                                :aria-label="`Open report #${data.id}`"
                                :disabled="actionsDisabled"
                                :loading="pendingAction?.kind === 'open' && pendingAction.id === data.id"
                                @click="emit('openReport', data.id)" />
                            <Button label="Delete" icon="pi pi-trash" size="small" outlined severity="danger"
                                :aria-label="`Delete report #${data.id}`"
                                :disabled="actionsDisabled"
                                @click="emit('requestDelete', data)" />
                        </div>
                    </template>
                </Column>
            </DataTable>
        </div>
        <template #footer>
            <Button label="Close" severity="secondary" outlined :disabled="busy || Boolean(confirmation)"
                @click="emit('update:modelValue', false)" />
        </template>
    </Dialog>

    <Dialog
        :visible="Boolean(confirmation)"
        :header="confirmation?.kind === 'clear' ? 'Clear all saved reports?' : 'Delete saved report?'"
        modal
        :draggable="false"
        :closable="!busy"
        :close-on-escape="!busy"
        :dismissable-mask="false"
        :style="{ width: 'min(480px, 94vw)' }"
        @update:visible="visible => { if (!visible) emit('cancelConfirmation'); }"
    >
        <div class="flex flex-column gap-3">
            <template v-if="confirmation?.kind === 'delete'">
                <p class="m-0">Delete report <strong>#{{ confirmation.report.id }}</strong> and all its positions?</p>
                <p class="m-0 text-600">
                    {{ confirmation.report.accountName || 'Unnamed account' }}<br>
                    {{ confirmation.report.statementPeriod || 'No statement period' }}
                </p>
            </template>
            <p v-else-if="confirmation?.kind === 'clear'" class="m-0">
                Delete all <strong>{{ confirmation.count }}</strong> saved reports and their positions?
            </p>
            <p class="m-0 text-600">This cannot be undone. Your original CSV files will remain on disk.</p>
            <Message v-if="operationError" severity="error" :closable="false">{{ operationError }}</Message>
        </div>
        <template #footer>
            <Button label="Cancel" severity="secondary" outlined :disabled="busy"
                @click="emit('cancelConfirmation')" />
            <Button :label="confirmation?.kind === 'clear' ? 'Delete all reports' : 'Delete report'"
                icon="pi pi-trash" severity="danger" :loading="busy" :disabled="busy"
                @click="emit('confirmDeletion')" />
        </template>
    </Dialog>
</template>
