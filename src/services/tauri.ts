import { invoke } from '@tauri-apps/api/core';
import type { PortfolioSummary } from '../types/portfolio.ts';
import type { RebalancePlan, TargetAllocation } from '../types/rebalance.ts';

interface CommandErrorPayload {
    code?: string;
    message?: string;
    details?: string;
}

export class TauriCommandError extends Error {
    code: string;
    details?: string;
    raw: unknown;

    constructor(payload: CommandErrorPayload, fallbackMessage: string, raw: unknown) {
        const message = payload.message || fallbackMessage;
        super(message);
        this.name = 'TauriCommandError';
        this.code = payload.code ?? 'unknown_error';
        this.details = payload.details;
        this.raw = raw;
    }
}

export const isTauriCommandError = (error: unknown): error is TauriCommandError =>
    error instanceof TauriCommandError;

const normalizeErrorPayload = (error: unknown): CommandErrorPayload => {
    if (!error) {
        return {};
    }

    if (typeof error === 'string') {
        try {
            const parsed = JSON.parse(error);
            if (parsed && typeof parsed === 'object') {
                return parsed as CommandErrorPayload;
            }
        } catch {
            return { message: error };
        }
    }

    if (typeof error === 'object') {
        const candidate = error as Record<string, unknown>;
        return {
            code: typeof candidate.code === 'string' ? candidate.code : undefined,
            message: typeof candidate.message === 'string' ? candidate.message : undefined,
            details: typeof candidate.details === 'string' ? candidate.details : undefined,
        };
    }

    return { message: String(error) };
};

export class TauriService {
    static isTauriAvailable(): boolean {
        return typeof window !== 'undefined' &&
            window.__TAURI_INTERNALS__ !== undefined;
    }

    private static ensureTauriAvailable(): void {
        if (!this.isTauriAvailable()) {
            throw new Error('Tauri APIs are not available. This application must be run in the Tauri desktop environment.');
        }
    }

    private static mapError(error: unknown, fallbackMessage: string): TauriCommandError {
        const payload = normalizeErrorPayload(error);
        return new TauriCommandError(payload, fallbackMessage, error);
    }

    static async openFileDialog(): Promise<string | null> {
        this.ensureTauriAvailable();
        try {
            return await invoke<string | null>('open_file_dialog');
        } catch (error) {
            throw this.mapError(error, 'Failed to open file dialog.');
        }
    }

    static async processCSVFile(filePath: string): Promise<PortfolioSummary> {
        this.ensureTauriAvailable();
        try {
            return await invoke<PortfolioSummary>('process_csv_report', { filePath });
        } catch (error) {
            throw this.mapError(error, 'Failed to process CSV file.');
        }
    }

    static async calcRebalance(filePath: string, targets: TargetAllocation[]): Promise<RebalancePlan> {
        this.ensureTauriAvailable();
        try {
            return await invoke<RebalancePlan>('suggest_rebalance', {
                filePath,
                targets,
            });
        } catch (error) {
            throw this.mapError(error, 'Failed to calculate rebalancing plan.');
        }
    }
}

declare global {
    interface Window {
        __TAURI_INTERNALS__?: any;
    }
}
