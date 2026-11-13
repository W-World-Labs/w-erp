import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { 
  Upload, 
  FileText, 
  Image as ImageIcon, 
  X, 
  File, 
  FileSpreadsheet,
  FileType,
  CheckCircle,
  AlertCircle,
  Loader2
} from "lucide-react";

interface FileUploaderProps {
  onFileSelect: (file: File) => void;
  onRemove?: () => void;
  selectedFile?: File | null;
  existingFileName?: string | null;
  accept?: string;
  maxSize?: number; // in MB
  disabled?: boolean;
  uploading?: boolean;
  progress?: number;
  error?: string | null;
  className?: string;
  compact?: boolean;
}

const DEFAULT_ACCEPT = "image/*,application/pdf,.doc,.docx,.xls,.xlsx,.txt";
const DEFAULT_MAX_SIZE = 10; // 10MB

export function FileUploader({
  onFileSelect,
  onRemove,
  selectedFile,
  existingFileName,
  accept = DEFAULT_ACCEPT,
  maxSize = DEFAULT_MAX_SIZE,
  disabled = false,
  uploading = false,
  progress = 0,
  error = null,
  className,
  compact = false,
}: FileUploaderProps) {
  const [isDragging, setIsDragging] = useState(false);

  const validateFile = useCallback((file: File): string | null => {
    // 验证文件大小
    if (file.size > maxSize * 1024 * 1024) {
      return `文件大小不能超过 ${maxSize}MB`;
    }

    // 验证文件类型
    const acceptedTypes = accept.split(",").map(t => t.trim());
    const fileType = file.type;
    const fileExt = `.${file.name.split('.').pop()?.toLowerCase()}`;

    const isAccepted = acceptedTypes.some(acceptType => {
      if (acceptType.startsWith('.')) {
        return fileExt === acceptType;
      }
      if (acceptType.endsWith('/*')) {
        return fileType.startsWith(acceptType.replace('/*', '/'));
      }
      return fileType === acceptType;
    });

    if (!isAccepted) {
      return "不支持的文件格式";
    }

    return null;
  }, [accept, maxSize]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (!disabled && !uploading) {
      setIsDragging(true);
    }
  }, [disabled, uploading]);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    if (disabled || uploading) return;

    const file = e.dataTransfer.files[0];
    if (file) {
      const validationError = validateFile(file);
      if (validationError) {
        // 使用外部错误处理或内部状态
        console.error(validationError);
        return;
      }
      onFileSelect(file);
    }
  }, [disabled, uploading, onFileSelect, validateFile]);

  const handleFileChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      const validationError = validateFile(file);
      if (validationError) {
        alert(validationError);
        return;
      }
      onFileSelect(file);
    }
    // 重置input以允许选择相同文件
    e.target.value = '';
  }, [onFileSelect, validateFile]);

  const getFileIcon = (fileName: string) => {
    const ext = fileName.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'pdf':
        return <FileText className="h-8 w-8 text-red-500" />;
      case 'doc':
      case 'docx':
        return <FileType className="h-8 w-8 text-blue-600" />;
      case 'xls':
      case 'xlsx':
        return <FileSpreadsheet className="h-8 w-8 text-green-600" />;
      case 'jpg':
      case 'jpeg':
      case 'png':
      case 'gif':
      case 'webp':
        return <ImageIcon className="h-8 w-8 text-purple-500" />;
      default:
        return <File className="h-8 w-8 text-gray-500" />;
    }
  };

  const currentFileName = selectedFile?.name || existingFileName;

  // 紧凑模式 - 显示选中的文件或上传按钮
  if (compact) {
    return (
      <div className={cn("space-y-2", className)}>
        <div className="flex items-center gap-2">
          <input
            type="file"
            accept={accept}
            onChange={handleFileChange}
            className="hidden"
            id="file-upload-compact"
            disabled={disabled || uploading}
          />
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => document.getElementById('file-upload-compact')?.click()}
            disabled={disabled || uploading}
          >
            <Upload className="h-4 w-4 mr-2" />
            {currentFileName ? '更换文件' : '选择文件'}
          </Button>
          
          {currentFileName && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <FileText className="h-4 w-4" />
              <span className="truncate max-w-[200px]">{currentFileName}</span>
              {onRemove && !uploading && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-6 w-6 p-0"
                  onClick={onRemove}
                >
                  <X className="h-3 w-3" />
                </Button>
              )}
            </div>
          )}
        </div>
        
        {uploading && (
          <Progress value={progress} className="h-1" />
        )}
        
        {error && (
          <p className="text-xs text-destructive flex items-center gap-1">
            <AlertCircle className="h-3 w-3" />
            {error}
          </p>
        )}
      </div>
    );
  }

  // 完整拖拽上传区域
  return (
    <div className={cn("space-y-3", className)}>
      {/* 拖拽区域 */}
      <div
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        className={cn(
          "relative border-2 border-dashed rounded-lg transition-all duration-200",
          "flex flex-col items-center justify-center p-6",
          isDragging && "border-primary bg-primary/5 scale-[1.02]",
          !isDragging && !disabled && "border-muted-foreground/25 hover:border-primary/50",
          disabled && "opacity-50 cursor-not-allowed bg-muted/30",
          error && "border-destructive/50"
        )}
      >
        <input
          type="file"
          accept={accept}
          onChange={handleFileChange}
          className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
          disabled={disabled || uploading}
        />

        {uploading ? (
          <div className="text-center space-y-3 w-full">
            <Loader2 className="h-10 w-10 animate-spin text-primary mx-auto" />
            <div className="space-y-1">
              <p className="text-sm font-medium">正在上传...</p>
              <Progress value={progress} className="h-2" />
              <p className="text-xs text-muted-foreground">{progress}%</p>
            </div>
          </div>
        ) : currentFileName ? (
          <div className="text-center space-y-3">
            <div className="flex items-center justify-center">
              {getFileIcon(currentFileName)}
            </div>
            <div className="space-y-1">
              <p className="text-sm font-medium truncate max-w-[250px]">
                {currentFileName}
              </p>
              <div className="flex items-center justify-center gap-1 text-xs text-green-600">
                <CheckCircle className="h-3 w-3" />
                已选择
              </div>
            </div>
            {onRemove && (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onRemove();
                }}
              >
                <X className="h-4 w-4 mr-1" />
                移除
              </Button>
            )}
          </div>
        ) : (
          <div className="text-center space-y-3">
            <div className={cn(
              "h-14 w-14 rounded-full flex items-center justify-center mx-auto",
              "bg-primary/10 text-primary transition-transform",
              isDragging && "scale-110"
            )}>
              <Upload className="h-6 w-6" />
            </div>
            <div className="space-y-1">
              <p className="text-sm font-medium">
                {isDragging ? "释放文件以上传" : "拖拽文件到此处，或点击选择"}
              </p>
              <p className="text-xs text-muted-foreground">
                支持 PDF、Word、Excel、图片等格式，最大 {maxSize}MB
              </p>
            </div>
          </div>
        )}
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="flex items-center gap-2 text-sm text-destructive">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      )}
    </div>
  );
}
