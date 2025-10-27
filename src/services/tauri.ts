import { invoke } from '@tauri-apps/api/core';
import type { PortfolioSummary } from '../types/portfolio.ts';
import type { RebalancePlan, TargetAllocation } from '../types/rebalance.ts';

export class TauriService {
    private static isTauriAvailable(): boolean {
        return typeof window !== undefined &&
            window.__TAURI_INTERNALS__ !== undefined;
    }

    private static ensureTauriAvailable(): void {
        if (!this.isTauriAvailable()) {
            throw new Error('Tauri APIs are not available. This application must be run in the Tauri desktop environment.');
        }
    }

    static async openFileDialog(): Promise<string | null> {
        this.ensureTauriAvailable();
        try {
            return await invoke<string | null>('open_file_dialog');
        } catch (error) {
            throw new Error(`Failed to open file dialog: ${error}`);
        }
    }

    static async processCSVFile(filePath: string): Promise<PortfolioSummary> {
        this.ensureTauriAvailable();
        try {
            return await invoke<PortfolioSummary>('process_csv_report', { filePath });
        } catch (error) {
            throw new Error(`Failed to process CSV file: ${error}`);
        }
    }

    static async calcRebalance(filePath: string, targets: TargetAllocation[]): Promise<RebalancePlan> {
        this.ensureTauriAvailable();
        return await invoke<RebalancePlan>('suggest_rebalance', {
            filePath,
            targets,
        });
    }
}

declare global {
    interface Window {
        __TAURI_INTERNALS__?: any;
    }
}
