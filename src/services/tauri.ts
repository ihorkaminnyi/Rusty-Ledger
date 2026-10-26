import { invoke } from "@tauri-apps/api/core";
import type { StoredPortfolioSummary, StoredReportListItem } from "../types/portfolio.ts";
import type {
  RebalancePlan,
  RebalanceStrategy,
  TargetAllocation,
} from "../types/rebalance.ts";
import type { NonNegativeNumber } from "../types/numeric.ts";

interface CommandErrorPayload {
  code?: string;
  message?: string;
  details?: string;
}

export class TauriCommandError extends Error {
  code: string;
  details?: string;
  raw: unknown;

  constructor(
    payload: CommandErrorPayload,
    fallbackMessage: string,
    raw: unknown,
  ) {
    const message = payload.message || fallbackMessage;
    super(message);
    this.name = "TauriCommandError";
    this.code = payload.code ?? "unknown_error";
    this.details = payload.details;
    this.raw = raw;
  }
}

export const isTauriCommandError = (
  error: unknown,
): error is TauriCommandError => error instanceof TauriCommandError;

const normalizeErrorPayload = (error: unknown): CommandErrorPayload => {
  if (!error) {
    return {};
  }

  if (typeof error === "string") {
    try {
      const parsed = JSON.parse(error);
      if (parsed && typeof parsed === "object") {
        return parsed as CommandErrorPayload;
      }
    } catch {
      return { message: error };
    }
  }

  if (typeof error === "object") {
    const candidate = error as Record<string, unknown>;
    return {
      code: typeof candidate.code === "string" ? candidate.code : undefined,
      message:
        typeof candidate.message === "string" ? candidate.message : undefined,
      details:
        typeof candidate.details === "string" ? candidate.details : undefined,
    };
  }

  return { message: String(error) };
};

export class TauriService {
  static isTauriAvailable(): boolean {
    return (
      typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined
    );
  }

  private static ensureTauriAvailable(): void {
    if (!this.isTauriAvailable()) {
      throw new Error(
        "Tauri APIs are not available. This application must be run in the Tauri desktop environment.",
      );
    }
  }

  private static mapError(
    error: unknown,
    fallbackMessage: string,
  ): TauriCommandError {
    const payload = normalizeErrorPayload(error);
    return new TauriCommandError(payload, fallbackMessage, error);
  }

  static async getAppVersion(): Promise<string> {
    if (!this.isTauriAvailable()) {
      return "0.1.0";
    }

    try {
      const { getVersion } = await import("@tauri-apps/api/app");
      return await getVersion();
    } catch (error) {
      throw this.mapError(error, "Failed to get app version.");
    }
  }

  static async openFileDialog(): Promise<string | null> {
    this.ensureTauriAvailable();
    try {
      return await invoke<string | null>("open_file_dialog");
    } catch (error) {
      throw this.mapError(error, "Failed to open file dialog.");
    }
  }

  static async getLatestPortfolioSummary(): Promise<StoredPortfolioSummary | null> {
    this.ensureTauriAvailable();
    try {
      return await invoke<StoredPortfolioSummary | null>("get_latest_portfolio_summary");
    } catch (error) {
      throw this.mapError(error, "Failed to get latest portfolio summary.");
    }
  }

  static async processCSVFile(filePath: string): Promise<StoredPortfolioSummary> {
    this.ensureTauriAvailable();
    try {
      return await invoke<StoredPortfolioSummary>("process_csv_report", { filePath });
    } catch (error) {
      throw this.mapError(error, "Failed to process CSV file.");
    }
  }

  static async listPortfolioSummaries(): Promise<StoredReportListItem[]> {
    this.ensureTauriAvailable();
    try {
      return await invoke<StoredReportListItem[]>("list_portfolio_summaries");
    } catch (error) {
      throw this.mapError(error, "Failed to load saved reports.");
    }
  }

  static async getPortfolioSummary(id: number): Promise<StoredPortfolioSummary | null> {
    this.ensureTauriAvailable();
    try {
      return await invoke<StoredPortfolioSummary | null>("get_portfolio_summary", { id });
    } catch (error) {
      throw this.mapError(error, "Failed to open the saved report.");
    }
  }

  static async deletePortfolioSummary(id: number): Promise<boolean> {
    this.ensureTauriAvailable();
    try {
      return await invoke<boolean>("delete_portfolio_summary", { id });
    } catch (error) {
      throw this.mapError(error, "Failed to delete the saved report.");
    }
  }

  static async clearPortfolioSummaries(): Promise<number> {
    this.ensureTauriAvailable();
    try {
      return await invoke<number>("clear_portfolio_summaries");
    } catch (error) {
      throw this.mapError(error, "Failed to clear saved reports.");
    }
  }

  static async calcRebalanceForSummary(
    id: number,
    targets: TargetAllocation[],
    depositAmount: NonNegativeNumber,
    rebalanceStrategy: RebalanceStrategy,
  ): Promise<RebalancePlan> {
    this.ensureTauriAvailable();
    try {
      return await invoke<RebalancePlan>("suggest_rebalance_for_summary", {
        id, targets, depositAmount, rebalanceStrategy,
      });
    } catch (error) {
      throw this.mapError(error, "Failed to calculate rebalancing for the saved report.");
    }
  }

  static async calcRebalance(
    filePath: string,
    targets: TargetAllocation[],
    depositAmount: NonNegativeNumber,
    rebalanceStrategy: RebalanceStrategy,
  ): Promise<RebalancePlan> {
    this.ensureTauriAvailable();
    try {
      return await invoke<RebalancePlan>("suggest_rebalance", {
        filePath,
        targets,
        depositAmount,
        rebalanceStrategy,
      });
    } catch (error) {
      throw this.mapError(error, "Failed to calculate rebalancing plan.");
    }
  }

  static async openExternalLink(url: string): Promise<void> {
    if (!this.isTauriAvailable()) {
      window.open(url, "_blank", "noopener");
      return;
    }

    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
    } catch (error) {
      throw this.mapError(error, "Failed to open external link.");
    }
  }
}

declare global {
  interface Window {
    __TAURI_INTERNALS__?: any;
  }
}
