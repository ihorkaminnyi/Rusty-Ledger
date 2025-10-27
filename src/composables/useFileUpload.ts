import { TauriService } from '../services/tauri';
import { useAppStore } from '../stores/appStore.ts';

export function useFileUpload() {
    const appStore = useAppStore();

    const processCSVFile = async (filePath: string) => {
        appStore.setLoading(true);
        appStore.setError(null);
        try {
            const portfolio = await TauriService.processCSVFile(filePath);
            appStore.setPortfolio(portfolio);
            appStore.setReportFilePath(filePath);
            appStore.setRebalance(null);
        } catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            appStore.setError(errorMessage);
            throw error;
        } finally {
            appStore.setLoading(false);
        }
    };

    const selectFile = async () => {
        const filePath = await TauriService.openFileDialog();
        if (filePath) {
            await processCSVFile(filePath);
        }
    };

    return {
        processCSVFile,
        selectFile,
    };
}
