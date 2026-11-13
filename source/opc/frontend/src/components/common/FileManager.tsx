import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
    Download,
    Eye,
    Trash2,
    FileText,
    FileSpreadsheet,
    Image as ImageIcon,
    File,
    MoreVertical,
    Upload
} from "lucide-react";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { FileUploader } from "./FileUploader";
import { FilePreviewDialog } from "./FilePreviewDialog";
import { cn } from "@/lib/utils";

export interface FileItem {
    id: string;
    fileName: string;
    filePath: string;
    fileType?: string;
    fileSize?: number;
    fileSizeLabel?: string;
    description?: string | null;
    uploadDate?: string;
    uploadDateLabel?: string;
}

interface FileManagerProps {
    files: FileItem[];
    bucketName: string;
    onUpload?: (file: File) => Promise<void>;
    onDelete?: (file: FileItem) => Promise<void>;
    onDownload?: (file: FileItem) => Promise<void>;
    uploading?: boolean;
    uploadProgress?: number;
    disabled?: boolean;
    showUploader?: boolean;
    maxFiles?: number;
    title?: string;
    emptyMessage?: string;
    className?: string;
    compact?: boolean;
}

export function FileManager({
    files,
    bucketName,
    onUpload,
    onDelete,
    onDownload,
    uploading = false,
    uploadProgress = 0,
    disabled = false,
    showUploader = true,
    maxFiles,
    title = "文件管理",
    emptyMessage = "暂无文件",
    className,
    compact = false,
}: FileManagerProps) {
    const [previewFile, setPreviewFile] = useState<FileItem | null>(null);
    const [deleteFile, setDeleteFile] = useState<FileItem | null>(null);
    const [selectedFile, setSelectedFile] = useState<File | null>(null);

    const handleFileSelect = useCallback((file: File) => {
        setSelectedFile(file);
    }, []);

    const handleUpload = useCallback(async () => {
        if (selectedFile && onUpload) {
            await onUpload(selectedFile);
            setSelectedFile(null);
        }
    }, [selectedFile, onUpload]);

    const handleDelete = useCallback(async () => {
        if (deleteFile && onDelete) {
            await onDelete(deleteFile);
            setDeleteFile(null);
        }
    }, [deleteFile, onDelete]);

    const getFileIcon = (fileName: string, fileType?: string) => {
        const ext = fileName.split('.').pop()?.toLowerCase();

        if (fileType?.startsWith('image/') || ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext || '')) {
            return <ImageIcon className="h-5 w-5 text-purple-500" />;
        }
        if (ext === 'pdf') {
            return <FileText className="h-5 w-5 text-red-500" />;
        }
        if (['doc', 'docx'].includes(ext || '')) {
            return <FileText className="h-5 w-5 text-blue-600" />;
        }
        if (['xls', 'xlsx'].includes(ext || '')) {
            return <FileSpreadsheet className="h-5 w-5 text-green-600" />;
        }
        return <File className="h-5 w-5 text-gray-500" />;
    };

    const canUpload = showUploader && (!maxFiles || files.length < maxFiles);

    return (
        <div className={cn("space-y-4", className)}>
            {/* 标题和统计 */}
            {!compact && (
                <div className="flex items-center justify-between">
                    <h4 className="text-sm font-medium">{title}</h4>
                    <Badge variant="secondary">{files.length} 个文件</Badge>
                </div>
            )}

            {/* 上传区域 */}
            {canUpload && onUpload && (
                <div className="space-y-2">
                    <FileUploader
                        onFileSelect={handleFileSelect}
                        onRemove={() => setSelectedFile(null)}
                        selectedFile={selectedFile}
                        disabled={disabled || uploading}
                        uploading={uploading}
                        progress={uploadProgress}
                        compact={compact}
                    />
                    {selectedFile && !uploading && (
                        <Button onClick={handleUpload} className="w-full" size="sm">
                            <Upload className="h-4 w-4 mr-2" />
                            确认上传
                        </Button>
                    )}
                </div>
            )}

            {/* 文件列表 */}
            {files.length === 0 ? (
                <div className="text-center py-6 text-sm text-muted-foreground">
                    {emptyMessage}
                </div>
            ) : (
                <div className={cn(
                    "space-y-2",
                    compact ? "max-h-48" : "max-h-72",
                    "overflow-y-auto"
                )}>
                    {files.map((file) => (
                        <Card key={file.id} className="hover:bg-muted/50 transition-colors">
                            <CardContent className="p-3">
                                <div className="flex items-center gap-3">
                                    {/* 文件图标 */}
                                    {getFileIcon(file.fileName, file.fileType)}

                                    {/* 文件信息 */}
                                    <div className="flex-1 min-w-0">
                                        <p className="text-sm font-medium truncate" title={file.fileName}>
                                            {file.fileName}
                                        </p>
                                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                            {file.fileSizeLabel && (
                                                <span>{file.fileSizeLabel}</span>
                                            )}
                                            {file.uploadDateLabel && (
                                                <span>{file.uploadDateLabel}</span>
                                            )}
                                            {file.description && (
                                                <span className="truncate">{file.description}</span>
                                            )}
                                        </div>
                                    </div>

                                    {/* 操作按钮 */}
                                    <div className="flex items-center gap-1">
                                        {/* 预览按钮 */}
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            onClick={() => setPreviewFile(file)}
                                            title="预览"
                                        >
                                            <Eye className="h-4 w-4" />
                                        </Button>

                                        {/* 更多操作 */}
                                        <DropdownMenu>
                                            <DropdownMenuTrigger asChild>
                                                <Button variant="ghost" size="sm">
                                                    <MoreVertical className="h-4 w-4" />
                                                </Button>
                                            </DropdownMenuTrigger>
                                            <DropdownMenuContent align="end">
                                                <DropdownMenuItem onClick={() => setPreviewFile(file)}>
                                                    <Eye className="h-4 w-4 mr-2" />
                                                    预览
                                                </DropdownMenuItem>
                                                {onDownload && (
                                                    <DropdownMenuItem onClick={() => onDownload(file)}>
                                                        <Download className="h-4 w-4 mr-2" />
                                                        下载
                                                    </DropdownMenuItem>
                                                )}
                                                {onDelete && !disabled && (
                                                    <DropdownMenuItem
                                                        onClick={() => setDeleteFile(file)}
                                                        className="text-destructive"
                                                    >
                                                        <Trash2 className="h-4 w-4 mr-2" />
                                                        删除
                                                    </DropdownMenuItem>
                                                )}
                                            </DropdownMenuContent>
                                        </DropdownMenu>
                                    </div>
                                </div>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            )}

            {/* 预览对话框 */}
            <FilePreviewDialog
                open={!!previewFile}
                onOpenChange={(open) => !open && setPreviewFile(null)}
                filePath={previewFile?.filePath || null}
                fileName={previewFile?.fileName || null}
                bucketName={bucketName}
            />

            {/* 删除确认对话框 */}
            <AlertDialog open={!!deleteFile} onOpenChange={(open) => !open && setDeleteFile(null)}>
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>确认删除</AlertDialogTitle>
                        <AlertDialogDescription>
                            确定要删除文件 "{deleteFile?.fileName}" 吗？此操作无法撤销。
                        </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogCancel>取消</AlertDialogCancel>
                        <AlertDialogAction onClick={handleDelete}>
                            确认删除
                        </AlertDialogAction>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>
        </div>
    );
}

// 简化版：单文件管理（用于表单中的单一文件字段）
interface SingleFileFieldProps {
    fileName?: string | null;
    filePath?: string | null;
    bucketName: string;
    onFileSelect: (file: File) => void;
    onRemove?: () => void;
    onPreview?: () => void;
    disabled?: boolean;
    uploading?: boolean;
    progress?: number;
}

export function SingleFileField({
    fileName,
    filePath,
    bucketName,
    onFileSelect,
    onRemove,
    onPreview,
    disabled = false,
    uploading = false,
    progress = 0,
}: SingleFileFieldProps) {
    const [showPreview, setShowPreview] = useState(false);

    return (
        <div className="space-y-2">
            <FileUploader
                onFileSelect={onFileSelect}
                onRemove={onRemove}
                existingFileName={fileName}
                disabled={disabled}
                uploading={uploading}
                progress={progress}
                compact={true}
            />

            {fileName && filePath && (
                <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => setShowPreview(true)}
                >
                    <Eye className="h-4 w-4 mr-2" />
                    预览当前文件
                </Button>
            )}

            <FilePreviewDialog
                open={showPreview}
                onOpenChange={setShowPreview}
                filePath={filePath || null}
                fileName={fileName || null}
                bucketName={bucketName}
            />
        </div>
    );
}
