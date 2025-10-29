import { onBeforeUnmount, onMounted, ref } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { TauriService, isTauriCommandError } from '../services/tauri';
import { useAppStore } from '../stores/appStore.ts';

type FileWithPath = File & { path?: string };

const getFilePathFromDropEvent = (event: DragEvent): string | null => {
    const files = event.dataTransfer?.files;
    const firstFile = files?.item(0) as FileWithPath | null;
    return firstFile?.path ?? null;
};

const clearDataTransfer = (event: DragEvent) => {
    event.dataTransfer?.clearData();
};

export function useFileUpload() {
    const appStore = useAppStore();
    const isDragging = ref(false);
    const tauriUnsubscribers: UnlistenFn[] = [];
    const isTauriEnv = TauriService.isTauriAvailable();

    const cleanupTauriListeners = () => {
        tauriUnsubscribers.forEach((unsubscribe) => {
            try {
                unsubscribe();
            } catch (error) {
                console.error('Failed to unsubscribe from Tauri event:', error);
            }
        });
        tauriUnsubscribers.length = 0;
    };

    const pushUploadError = (error: unknown) => {
        if (isTauriCommandError(error)) {
            appStore.pushError({
                scope: 'upload',
                message: error.message,
                details: error.details,
                code: error.code,
            });
            return;
        }

        const fallbackMessage = error instanceof Error ? error.message : String(error);
        appStore.pushError({
            scope: 'upload',
            message: fallbackMessage,
            details: error instanceof Error ? error.stack : undefined,
        });
    };

    const processFilePath = async (filePath: string) => {
        appStore.setLoading(true);
        appStore.clearErrorsByScope('upload');
        try {
            const portfolio = await TauriService.processCSVFile(filePath);
            appStore.setPortfolio(portfolio);
            appStore.setReportFilePath(filePath);
            appStore.setRebalance(null);
        } catch (error) {
            pushUploadError(error);
            throw error;
        } finally {
            appStore.setLoading(false);
        }
    };

    const selectFile = async () => {
        try {
            const filePath = await TauriService.openFileDialog();
            if (filePath) {
                await processFilePath(filePath);
            }
        } catch (error) {
            pushUploadError(error);
        }
    };

    const handleDragEnter = () => {
        isDragging.value = true;
    };
    const handleDragOver = () => {
        isDragging.value = true;
    };
    const handleDragLeave = () => {
        isDragging.value = false;
    };

    const handleDrop = async (event: DragEvent) => {
        isDragging.value = false;

        if (appStore.isLoading) {
            clearDataTransfer(event);
            return;
        }

        if (isTauriEnv && tauriUnsubscribers.length > 0) {
            clearDataTransfer(event);
            return;
        }

        const filePath = getFilePathFromDropEvent(event);
        if (!filePath) {
            console.error('Dropped file is missing a file path. Drag-and-drop is only supported in the desktop app.');
            return;
        }

        try {
            await processFilePath(filePath);
        } catch (error) {
            console.error('Failed to process dropped file:', error);
            pushUploadError(error);
        } finally {
            clearDataTransfer(event);
        }
    };

    const registerTauriDragDrop = async () => {
        if (!isTauriEnv) {
            return;
        }

        try {
            const appWindow = getCurrentWindow();
            const unlisten = await appWindow.onDragDropEvent(async ({ payload }) => {
                switch (payload.type) {
                    case 'enter':
                    case 'over':
                        isDragging.value = true;
                        return;
                    case 'leave':
                        isDragging.value = false;
                        return;
                    case 'drop': {
                        isDragging.value = false;
                        if (appStore.isLoading) {
                            return;
                        }

                        const filePath = payload.paths?.[0];
                        if (!filePath) {
                            return;
                        }

                        try {
                            await processFilePath(filePath);
                        } catch (error) {
                            console.error('Failed to process dropped file:', error);
                            pushUploadError(error);
                        }
                        return;
                    }
                    default:
                        return;
                }
            });
            tauriUnsubscribers.push(unlisten);
        } catch (error) {
            console.error('Failed to register Tauri drag-n-drop listeners:', error);
            cleanupTauriListeners();
        }
    };

    onMounted(() => {
        if (!isTauriEnv) {
            return;
        }

        void registerTauriDragDrop();
    });

    onBeforeUnmount(() => {
        cleanupTauriListeners();
    });

    return {
        isDragging,
        selectFile,
        handleDragEnter,
        handleDragOver,
        handleDragLeave,
        handleDrop,
    };
}
