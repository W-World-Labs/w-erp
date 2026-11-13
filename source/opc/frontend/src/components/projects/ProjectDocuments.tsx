import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { api, files, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { Upload, FileText, Image as ImageIcon, X, Download, Trash2, Camera } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";

interface ProjectDocumentsProps {
  projectId?: string;
  disabled?: boolean;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

interface Document {
  id: string;
  document_type: string;
  file_name: string;
  file_path: string;
  description: string | null;
  upload_date: string;
  upload_date_label?: string;
  document_type_label: string;
  file_icon: "FileText" | "ImageIcon";
  file_icon_class_name: string;
  delete_action: DeleteAction | null;
}

interface DocumentTypeOption {
  value: string;
  label: string;
}

interface DocumentsOverview {
  documents: Document[];
  count: number;
  document_type_options: DocumentTypeOption[];
  upload_action: BackendAction | null;
  empty_message: string;
}

const documentIconMap = {
  FileText,
  ImageIcon,
};

const DocumentIcon = ({
  name,
  className,
}: {
  name?: keyof typeof documentIconMap;
  className?: string;
}) => {
  const Icon = name ? documentIconMap[name] || FileText : FileText;
  return <Icon className={className || "h-5 w-5 text-gray-500"} />;
};

export function ProjectDocuments({ projectId, disabled }: ProjectDocumentsProps) {
  const [uploading, setUploading] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [documentType, setDocumentType] = useState<string>("");
  const [description, setDescription] = useState<string>("");

  const { data: documentsOverview, refetch: refetchDocuments } = useQuery({
    queryKey: ["project-documents-overview", projectId],
    queryFn: async () => {
      if (!projectId) {
        return {
          documents: [],
          count: 0,
          document_type_options: [],
          upload_action: null,
          empty_message: "暂无项目文档",
        };
      }
      return api.get<DocumentsOverview>(
        `/api/project_documents/overview?project_id=${encodeURIComponent(projectId)}`,
      );
    },
    enabled: !!projectId,
  });
  const documents = documentsOverview?.documents ?? [];
  const documentTypes = documentsOverview?.document_type_options ?? [];
  const uploadAction = documentsOverview?.upload_action ?? null;

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
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
    ];
    
    if (!validTypes.includes(file.type)) {
      toast.error("只支持图片、PDF、Word和Excel文档");
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
    if (!selectedFile || !documentType || !projectId) {
      toast.error("请选择文件和文档类型");
      return;
    }

    setUploading(true);

    try {
      const uploadResult = await files.uploadForEntity(
        selectedFile,
        "projects",
        projectId,
      );

      const action = uploadAction;
      if (!action) {
        toast.error("当前无法上传项目文档");
        return;
      }

      await api.runAction(action, {
          ...action.default_values,
          document_type: documentType,
          file_name: uploadResult.original_name,
          file_path: uploadResult.id,
          description: description || null,
        });

      toast.success(action.success_message || "文档上传成功");
      setSelectedFile(null);
      setDocumentType("");
      setDescription("");
      refetchDocuments();
    } catch (error) {
      console.error('Upload error:', error);
      toast.error("上传失败");
    } finally {
      setUploading(false);
    }
  };

  const handleDownload = async (document: Document) => {
    try {
      const data = await files.download(document.file_path);

      // 创建下载链接
      const url = URL.createObjectURL(data);
      const a = window.document.createElement('a');
      a.href = url;
      a.download = document.file_name;
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
      refetchDocuments();
    } catch (error) {
      console.error('Delete error:', error);
      toast.error("删除失败");
    }
  };

  if (!projectId) {
    return (
      <Alert>
        <AlertDescription>
          请先保存项目基本信息后再上传文档
        </AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="space-y-4">
      <div className="space-y-3">
        <Label>上传项目文档</Label>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div className="space-y-2">
            <Label className="text-xs">文档类型 *</Label>
            <Select 
              value={documentType} 
              onValueChange={setDocumentType}
              disabled={disabled || uploading}
            >
              <SelectTrigger>
                <SelectValue placeholder="选择文档类型" />
              </SelectTrigger>
              <SelectContent>
                {documentTypes.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label className="text-xs">文档说明（可选）</Label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="如：初版设计方案"
              disabled={disabled || uploading}
            />
          </div>
        </div>

        {selectedFile ? (
          <div className="flex items-center gap-2 p-3 border rounded-md bg-primary/5">
            <FileText className="h-5 w-5 text-gray-500" />
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
              onClick={() => document.getElementById('project-doc-upload')?.click()}
              disabled={disabled || uploading}
            >
              <Upload className="mr-2 h-4 w-4" />
              选择文件
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => document.getElementById('project-doc-camera')?.click()}
              disabled={disabled || uploading}
            >
              <Camera className="mr-2 h-4 w-4" />
              拍照上传
            </Button>
            <input
              id="project-doc-upload"
              type="file"
              accept="image/jpeg,image/jpg,image/png,application/pdf,.doc,.docx,.xls,.xlsx"
              className="hidden"
              onChange={handleFileChange}
              disabled={disabled || uploading}
            />
            <input
              id="project-doc-camera"
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
            disabled={!documentType || uploading}
            className="w-full"
          >
            {uploading ? "上传中..." : "上传文档"}
          </Button>
        )}

        <Alert>
          <FileText className="h-4 w-4" />
          <AlertDescription className="text-xs">
            支持图片、PDF、Word、Excel文档，大小不超过 10MB
          </AlertDescription>
        </Alert>
      </div>

      {documents.length > 0 && (
        <>
          <Separator />
          <div className="space-y-2">
            <Label>已上传文档 ({documents.length})</Label>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {documents.map((doc) => (
                <Card key={doc.id}>
                  <CardContent className="p-3">
                    <div className="flex items-center gap-3">
                      <DocumentIcon name={doc.file_icon} className={doc.file_icon_class_name} />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="text-sm font-medium truncate">
                            {doc.file_name}
                          </span>
                          <span className="text-xs bg-primary/10 text-primary px-2 py-0.5 rounded whitespace-nowrap">
                            {doc.document_type_label}
                          </span>
                        </div>
                        {doc.description && (
                          <p className="text-xs text-muted-foreground truncate">
                            {doc.description}
                          </p>
                        )}
                        <p className="text-xs text-muted-foreground">
                          {doc.upload_date_label}
                        </p>
                      </div>
                      <div className="flex gap-1">
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDownload(doc)}
                          disabled={disabled}
                        >
                          <Download className="h-4 w-4" />
                        </Button>
                        {doc.delete_action && (
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDelete(doc.delete_action)}
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
