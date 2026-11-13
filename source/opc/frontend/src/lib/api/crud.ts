import { api } from '@/integrations/api/client';

import { getEndpoint } from './contracts';
import { ApiError, type ApiResponse } from './errors';

export async function fetchData<T>(
    tableName: string,
    options?: {
        select?: string;
        filters?: Record<string, unknown>;
        order?: { column: string; ascending?: boolean };
        limit?: number;
        single?: boolean;
    }
): Promise<ApiResponse<T>> {
    try {
        const endpoint = getEndpoint(tableName);
        const params = new URLSearchParams();
        if (options?.filters) {
            for (const [key, value] of Object.entries(options.filters)) {
                if (value !== undefined && value !== null) {
                    params.set(key, String(value));
                }
            }
        }
        if (options?.order) {
            params.set('order_by', options.order.column);
            params.set('order_dir', options.order.ascending ? 'asc' : 'desc');
        }
        if (options?.limit) {
            params.set('limit', String(options.limit));
        }

        const url = params.toString() ? `${endpoint}?${params}` : endpoint;
        const result = await api.get<{ data: T; count?: number }>(url);
        return { data: result.data as T, error: null, success: true };
    } catch (err) {
        console.warn(`[API] fetchData(${tableName}) error:`, err);
        return {
            data: null,
            error: new ApiError(err instanceof Error ? err.message : '获取数据失败', 'FETCH_ERROR'),
            success: false,
        };
    }
}

export async function insertData<T>(
    tableName: string,
    data: Record<string, unknown> | Record<string, unknown>[],
    _options?: { returning?: boolean }
): Promise<ApiResponse<T>> {
    try {
        const endpoint = getEndpoint(tableName);
        const insertPayload = Array.isArray(data) ? data[0] : data;
        const result = await api.post<T>(endpoint, insertPayload);
        return { data: result as T, error: null, success: true };
    } catch (err) {
        console.warn(`[API] insertData(${tableName}) error:`, err);
        return {
            data: null,
            error: new ApiError(err instanceof Error ? err.message : '创建数据失败', 'INSERT_ERROR'),
            success: false,
        };
    }
}

export async function updateData<T>(
    tableName: string,
    id: string,
    data: Record<string, unknown>,
    _options?: { returning?: boolean }
): Promise<ApiResponse<T>> {
    try {
        const endpoint = `${getEndpoint(tableName)}/${id}`;
        const result = await api.put<T>(endpoint, data);
        return { data: result as T, error: null, success: true };
    } catch (err) {
        console.warn(`[API] updateData(${tableName}, ${id}) error:`, err);
        return {
            data: null,
            error: new ApiError(err instanceof Error ? err.message : '更新数据失败', 'UPDATE_ERROR'),
            success: false,
        };
    }
}

export async function deleteData(
    tableName: string,
    id: string
): Promise<ApiResponse<null>> {
    try {
        const endpoint = `${getEndpoint(tableName)}/${id}`;
        await api.delete(endpoint);
        return { data: null, error: null, success: true };
    } catch (err) {
        console.warn(`[API] deleteData(${tableName}, ${id}) error:`, err);
        return {
            data: null,
            error: new ApiError(err instanceof Error ? err.message : '删除数据失败', 'DELETE_ERROR'),
            success: false,
        };
    }
}
