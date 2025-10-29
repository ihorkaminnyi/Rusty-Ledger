export type AppErrorScope =
    | 'global'
    | 'upload'
    | 'rebalancing'
    | 'tauri'
    | 'portfolio'
    | (string & {});

export interface AppError {
    id: string;
    scope: AppErrorScope;
    message: string;
    code?: string;
    details?: string;
    createdAt: number;
}

export interface AppErrorInput {
    scope?: AppErrorScope;
    message: string;
    code?: string;
    details?: string;
    append?: boolean;
}
