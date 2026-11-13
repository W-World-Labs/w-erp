import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { files } from "@/integrations/api/client";
import {
    Download,
    ExternalLink,
    ZoomIn,
    ZoomOut,
    RotateCw,
    FileText,
    FileSpreadsheet,
    File,
    AlertCircle,
    X,
    Maximize2,
    Minimize2
} from "lucide-react";
import { cn } from "@/lib/utils";

interface FilePreviewDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    filePath: string | null;
    fileName: string | null;
    bucketName?: string;
}

type PreviewType = 'image' | 'pdf' | 'office' | 'unsupported';

export function FilePreviewDialog({
    open,
    onOpenChange,
    filePath,
    fileName,
    bucketName = 'contracts',
}: FilePreviewDialogProps) {
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [previewUrl, setPreviewUrl] = useState<string | null>(null);
    const [previewType, setPreviewType] = useState<PreviewType>('unsupported');
    const [zoom, setZoom] = useState(100);
    const [rotation, setRotation] = useState(0);
    const [isFullscreen, setIsFullscreen] = useState(false);

    useEffect(() => {
        if (open && filePath) {
            loadPreview();
        }
        return () => {
            if (previewUrl) {
                URL.revokeObjectURL(previewUrl);
            }
        };
    }, [open, filePath]);

    const getPreviewType = (name: string): PreviewType => {
        const ext = name.split('.').pop()?.toLowerCase();
        if (['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg'].includes(ext || '')) {
            return 'image';
        }
        if (ext === 'pdf') {
            return 'pdf';
        }
        if (['doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx'].includes(ext || '')) {
            return 'office';
        }
        return 'unsupported';
    };

    const loadPreview = async () => {
        if (!filePath || !fileName) return;

        setLoading(true);
        setError(null);
        setZoom(100);
        setRotation(0);

        try {
            const type = getPreviewType(fileName);
            setPreviewType(type);

            if (type === 'unsupported') {
                setLoading(false);
                return;
            }

            const data = await files.download(filePath);

            // 创建预览 URL
            const url = URL.createObjectURL(data);
            setPreviewUrl(url);
        } catch (err: any) {
            console.error('Preview load error:', err);
            setError(err.message || '无法加载文件预览');
        } finally {
            setLoading(false);
        }
    };

    const handleDownload = async () => {
        if (!filePath || !fileName) return;

        try {
            const data = await files.download(filePath);

            const url = URL.createObjectURL(data);
            const a = document.createElement('a');
            a.href = url;
            a.download = fileName;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (err: any) {
            console.error('Download error:', err);
        }
    };

    const handleZoomIn = () => setZoom((prev) => Math.min(prev + 25, 200));
    const handleZoomOut = () => setZoom((prev) => Math.max(prev - 25, 25));
    const handleRotate = () => setRotation((prev) => (prev + 90) % 360);

    const getFileIcon = () => {
        const ext = fileName?.split('.').pop()?.toLowerCase();
        switch (ext) {
            case 'pdf':
                return <FileText className="h-16 w-16 text-red-500" />;
            case 'doc':
            case 'docx':
                return <FileText className="h-16 w-16 text-blue-600" />;
            case 'xls':
            case 'xlsx':
                return <FileSpreadsheet className="h-16 w-16 text-green-600" />;
            case 'ppt':
            case 'pptx':
                return <FileText className="h-16 w-16 text-orange-500" />;
            default:
                return <File className="h-16 w-16 text-gray-500" />;
        }
    };

    const renderPreview = () => {
        if (loading) {
            return (
                <div className="flex items-center justify-center h-96">
                    <div className="space-y-4 text-center">
                        <Skeleton className="h-48 w-72 mx-auto" />
                        <Skeleton className="h-4 w-32 mx-auto" />
                    </div>
                </div>
            );
        }

        if (error) {
            return (
                <div className="flex flex-col items-center justify-center h-96 text-destructive">
                    <AlertCircle className="h-16 w-16 mb-4" />
                    <p className="text-lg font-medium">加载失败</p>
                    <p className="text-sm text-muted-foreground mt-2">{error}</p>
                    <Button variant="outline" className="mt-4" onClick={loadPreview}>
                        重试
                    </Button>
                </div>
            );
        }

        switch (previewType) {
            case 'image':
                return (
                    <div className="relative flex items-center justify-center overflow-auto bg-muted/20 rounded-lg min-h-[400px]">
                        <img
                            src={previewUrl || ''}
                            alt={fileName || 'Preview'}
                            style={{
                                transform: `scale(${zoom / 100}) rotate(${rotation}deg)`,
                                transition: 'transform 0.2s ease',
                                maxWidth: '100%',
                                maxHeight: '70vh',
                            }}
                            className="object-contain"
                        />
                    </div>
                );

            case 'pdf':
                return (
                    <div className="relative bg-muted/20 rounded-lg overflow-hidden">
                        <iframe
                            src={previewUrl || ''}
                            className="w-full border-0"
                            style={{ height: isFullscreen ? '80vh' : '500px' }}
                            title={fileName || 'PDF Preview'}
                        />
                    </div>
                );

            case 'office':
                // Office 文档使用 Microsoft Office Online Viewer 或提示下载
                return (
                    <div className="flex flex-col items-center justify-center h-96 text-center">
                        {getFileIcon()}
                        <p className="text-lg font-medium mt-4">{fileName}</p>
                        <p className="text-sm text-muted-foreground mt-2 max-w-md">
                            Office 文档暂不支持在线预览，请下载后使用相应程序打开
                        </p>
                        <div className="flex gap-2 mt-6">
                            <Button onClick={handleDownload}>
                                <Download className="h-4 w-4 mr-2" />
                                下载文件
                            </Button>
                        </div>
                    </div>
                );

            default:
                return (
                    <div className="flex flex-col items-center justify-center h-96 text-center">
                        {getFileIcon()}
                        <p className="text-lg font-medium mt-4">{fileName}</p>
                        <p className="text-sm text-muted-foreground mt-2">
                            此文件类型不支持在线预览
                        </p>
                        <Button onClick={handleDownload} className="mt-6">
                            <Download className="h-4 w-4 mr-2" />
                            下载文件
                        </Button>
                    </div>
                );
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className={cn(
                "max-w-4xl",
                isFullscreen && "max-w-[95vw] max-h-[95vh]"
            )}>
                <DialogHeader className="flex flex-row items-center justify-between">
                    <DialogTitle className="truncate flex-1 pr-4">
                        {fileName || '文件预览'}
                    </DialogTitle>
                </DialogHeader>

                {/* 工具栏 */}
                <div className="flex items-center justify-between border-b pb-3">
                    <div className="flex items-center gap-1">
                        {previewType === 'image' && (
                            <>
                                <Button variant="ghost" size="sm" onClick={handleZoomOut} title="缩小">
                                    <ZoomOut className="h-4 w-4" />
                                </Button>
                                <span className="text-sm text-muted-foreground min-w-[50px] text-center">
                                    {zoom}%
                                </span>
                                <Button variant="ghost" size="sm" onClick={handleZoomIn} title="放大">
                                    <ZoomIn className="h-4 w-4" />
                                </Button>
                                <div className="w-px h-4 bg-border mx-2" />
                                <Button variant="ghost" size="sm" onClick={handleRotate} title="旋转">
                                    <RotateCw className="h-4 w-4" />
                                </Button>
                            </>
                        )}
                    </div>

                    <div className="flex items-center gap-1">
                        <Button variant="ghost" size="sm" onClick={() => setIsFullscreen(!isFullscreen)}>
                            {isFullscreen ? (
                                <Minimize2 className="h-4 w-4" />
                            ) : (
                                <Maximize2 className="h-4 w-4" />
                            )}
                        </Button>
                        <Button variant="ghost" size="sm" onClick={handleDownload}>
                            <Download className="h-4 w-4" />
                        </Button>
                        {previewUrl && previewType === 'pdf' && (
                            <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => window.open(previewUrl, '_blank')}
                                title="在新标签页打开"
                            >
                                <ExternalLink className="h-4 w-4" />
                            </Button>
                        )}
                    </div>
                </div>

                {/* 预览区域 */}
                <div className="mt-2">
                    {renderPreview()}
                </div>
            </DialogContent>
        </Dialog>
    );
}
