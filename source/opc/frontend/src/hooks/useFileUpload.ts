import { useState, useCallback } from "react";
import { api, files } from "@/integrations/api/client";
import { toast } from "sonner";

interface UseFileUploadOptions {
    bucketName: string;
    onSuccess?: (filePath: string, fileName: string) => void;
    onError?: (error: Error) => void;
}

interface UseFileUploadReturn {
    upload: (file: File) => Promise<{ filePath: string; fileName: string } | null>;
    download: (filePath: string, fileName: string) => Promise<void>;
    remove: (filePath: string) => Promise<boolean>;
    getPublicUrl: (filePath: string) => string | null;
    uploading: boolean;
    progress: number;
    error: string | null;
}

export function useFileUpload({
    bucketName,
    onSuccess,
    onError,
}: UseFileUploadOptions): UseFileUploadReturn {
    const [uploading, setUploading] = useState(false);
    const [progress, setProgress] = useState(0);
    const [error, setError] = useState<string | null>(null);

    const upload = useCallback(async (file: File) => {
        setUploading(true);
        setProgress(0);
        setError(null);

        try {
            // 模拟进度（REST API 客户端目前不支持上传进度）
            const progressInterval = setInterval(() => {
                setProgress((prev) => {
                    if (prev >= 90) {
                        clearInterval(progressInterval);
                        return prev;
                    }
                    return prev + 10;
                });
            }, 100);

            const uploadResult = await files.upload(file);

            clearInterval(progressInterval);

            setProgress(100);

            const result = {
                filePath: uploadResult.id,
                fileName: uploadResult.original_name,
            };
            onSuccess?.(uploadResult.id, uploadResult.original_name);

            toast.success("文件上传成功");
            return result;
        } catch (err: any) {
            const errorMessage = err.message || "文件上传失败";
            setError(errorMessage);
            onError?.(err);
            toast.error(errorMessage);
            return null;
        } finally {
            setUploading(false);
        }
    }, [bucketName, onSuccess, onError]);

    const download = useCallback(async (filePath: string, fileName: string) => {
        try {
            // filePath is actually the file ID from backend
            const response = await fetch(`/api/files/${encodeURIComponent(filePath)}`, {
                credentials: 'include',
            });
            if (!response.ok) {
                throw new Error(`Download failed: ${response.status}`);
            }
            const data = await response.blob();

            // 创建下载链接
            const url = URL.createObjectURL(data);
            const a = document.createElement('a');
            a.href = url;
            a.download = fileName;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);

            toast.success(`已下载: ${fileName}`);
        } catch (err: any) {
            console.error('Download error:', err);
            toast.error("下载失败: " + (err.message || "未知错误"));
        }
    }, [bucketName]);

    const remove = useCallback(async (filePath: string): Promise<boolean> => {
        try {
            // filePath is actually the file ID from backend
            await api.delete(`/api/files/${encodeURIComponent(filePath)}`);

            toast.success("文件已删除");
            return true;
        } catch (err: any) {
            console.error('Remove error:', err);
            toast.error("删除失败: " + (err.message || "未知错误"));
            return false;
        }
    }, [bucketName]);

    const getPublicUrl = useCallback((filePath: string): string | null => {
        try {
            return `/api/files/${encodeURIComponent(filePath)}`;
        } catch (err) {
            console.error('Get public URL error:', err);
            return null;
        }
    }, [bucketName]);

    return {
        upload,
        download,
        remove,
        getPublicUrl,
        uploading,
        progress,
        error,
    };
}

// 便捷的预定义上传钩子
export function useContractFileUpload(
    onSuccess?: (filePath: string, fileName: string) => void
) {
    return useFileUpload({
        bucketName: 'contracts',
        onSuccess,
    });
}

export function useInvoiceFileUpload(
    onSuccess?: (filePath: string, fileName: string) => void
) {
    return useFileUpload({
        bucketName: 'invoices',
        onSuccess,
    });
}

export function useAttachmentFileUpload(
    entityType: string,
    onSuccess?: (filePath: string, fileName: string) => void
) {
    return useFileUpload({
        bucketName: 'attachments',
        onSuccess,
    });
}
