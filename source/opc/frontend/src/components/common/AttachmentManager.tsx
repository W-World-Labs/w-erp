import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { api, files, type DeleteAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { Upload, FileText, Image as ImageIcon, X, Download, Trash2, File, Camera } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";

interface AttachmentManagerProps {
  entityId?: string;
  entityType: 'asset' | 'budget' | 'rent' | 'salary' | 'utility' | 'payment';
  disabled?: boolean;
}

interface Attachment {
  id: string;
  file_name: string;
  file_path: string;
  file_type: string;
  description: string | null;
  upload_date: string;
  upload_date_label?: string;
  file_icon: "FileText" | "ImageIcon" | "File";
  file_icon_class_name: string;
  delete_action: DeleteAction | null;
}

const attachmentIconMap = {
  FileText,
  ImageIcon,
  File,
};

const AttachmentIcon = ({
  name,
  className,
}: {
  name?: keyof typeof attachmentIconMap;
  className?: string;
}) => {
  const Icon = name ? attachmentIconMap[name] || File : File;
  return <Icon className={className || "h-5 w-5 text-gray-500"} />;
};

export function AttachmentManager({ entityId, entityType, disabled }: AttachmentManagerProps) {
  const [uploading, setUploading] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [description, setDescription] = useState<string>("");

  const { data: attachmentsOverview, refetch: refetchAttachments } = useQuery({
    queryKey: ["attachments-overview", entityType, entityId],
    queryFn: async () => {
      if (!entityId) return { data: [] as Attachment[] };
      return files.list(entityType, entityId);
    },
    enabled: !!entityId,
  });
  const attachments = (attachmentsOverview?.data || []) as Attachment[];

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // 验证文件类型
    const validTypes = [
      'image/jpeg', 'image/jpg', 'image/png',
      'application/pdf',
      'application/msword',
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'application/vnd.ms-excel',
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      'text/plain'
    ];

    if (!validTypes.includes(file.type)) {
      toast.error("只支持图片、PDF、Word、Excel和文本文件");
      return;
    }

    // 验证文件大小 (10MB)
    if (file.size > 10 * 1024 * 1024) {
      toast.error("文件大小不能超过 10MB");
      return;
    }

    setSelectedFile(file);
  };

  const handleUpload = async () => {
    if (!selectedFile || !entityId) {
      toast.error("请选择文件");
      return;
    }

    setUploading(true);

    try {
      await files.uploadForEntity(
        selectedFile,
        entityType,
        entityId,
        { description },
      );

      toast.success("附件上传成功");
      setSelectedFile(null);
      setDescription("");
      refetchAttachments();
    } catch (error) {
      console.error('Upload error:', error);
      toast.error("上传失败");
    } finally {
      setUploading(false);
    }
  };

  const handleDownload = async (attachment: Attachment) => {
    try {
      const blob = await files.download(attachment.file_path);

      // 创建下载链接
      const url = URL.createObjectURL(blob);
      const a = window.document.createElement('a');
      a.href = url;
      a.download = attachment.file_name;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Download error:', error);
      toast.error("下载失败");
    }
  };

  const handleDelete = async (action?: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;

    try {
      await api.runAction(action);

      toast.success(action.success_message);
      refetchAttachments();
    } catch (error) {
      console.error('Delete error:', error);
      toast.error("删除失败");
    }
  };

  if (!entityId) {
    return (
      <Alert>
        <AlertDescription>
          请先保存记录后再上传附件
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="space-y-4">
      <div className="space-y-3">
        <Label>上传附件</Label>

        <div className="space-y-2">
          <Label className="text-xs">附件说明（可选）</Label>
          <Input
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="如：发票扫描件、照片等"
            disabled={disabled || uploading}
          />
        </div>

        {selectedFile ? (
          <div className="flex items-center gap-2 p-3 border rounded-md bg-primary/5">
            <File className="h-5 w-5 text-gray-500" />
            <span className="flex-1 text-sm truncate">{selectedFile.name}</span>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => setSelectedFile(null)}
              disabled={uploading}
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => document.getElementById(`attachment-upload-${entityType}`)?.click()}
              disabled={disabled || uploading}
            >
              <Upload className="mr-2 h-4 w-4" />
              选择文件
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => document.getElementById(`attachment-camera-${entityType}`)?.click()}
              disabled={disabled || uploading}
            >
              <Camera className="mr-2 h-4 w-4" />
              拍照上传
            </Button>
            <input
              id={`attachment-upload-${entityType}`}
              type="file"
              accept="image/*,application/pdf,.doc,.docx,.xls,.xlsx,.txt"
              className="hidden"
              onChange={handleFileChange}
              disabled={disabled || uploading}
            />
            <input
              id={`attachment-camera-${entityType}`}
              type="file"
              accept="image/*"
              capture="environment"
              className="hidden"
              onChange={handleFileChange}
              disabled={disabled || uploading}
            />
          </div>
        )}

        {selectedFile && (
          <Button
            type="button"
            onClick={handleUpload}
            disabled={uploading}
            className="w-full"
          >
            {uploading ? "上传中..." : "上传附件"}
          </Button>
        )}

        <Alert>
          <FileText className="h-4 w-4" />
          <AlertDescription className="text-xs">
            支持图片、PDF、Word、Excel、文本文件，大小不超过 10MB
          </AlertDescription>
        </Alert>
      </div>

      {attachments.length > 0 && (
        <>
          <Separator />
          <div className="space-y-2">
            <Label>已上传附件 ({attachments.length})</Label>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {attachments.map((att) => (
                <Card key={att.id}>
                  <CardContent className="p-3">
                    <div className="flex items-center gap-3">
                      <AttachmentIcon name={att.file_icon} className={att.file_icon_class_name} />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate">
                          {att.file_name}
                        </p>
                        {att.description && (
                          <p className="text-xs text-muted-foreground truncate">
                            {att.description}
                          </p>
                        )}
                        <p className="text-xs text-muted-foreground">
                          {att.upload_date_label}
                        </p>
                      </div>
                      <div className="flex gap-1">
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDownload(att)}
                          disabled={disabled}
                        >
                          <Download className="h-4 w-4" />
                        </Button>
                        {att.delete_action && (
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDelete(att.delete_action)}
                            disabled={disabled}
                          >
                            <Trash2 className="h-4 w-4 text-destructive" />
                          </Button>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
