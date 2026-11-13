import { useState, useRef } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { files } from "@/integrations/api/client";
import { useToast } from "@/hooks/use-toast";
import { Upload, FileText } from "lucide-react";
import { createMaterialVersion } from "@/lib/materialVersions";

interface MaterialUploadDialogProps {
  materialId: string;
  materialName: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const MaterialUploadDialog = ({
  materialId,
  materialName,
  open,
  onOpenChange
}: MaterialUploadDialogProps) => {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [description, setDescription] = useState("");
  const [changeNotes, setChangeNotes] = useState("");

  const uploadMutation = useMutation({
    mutationFn: async () => {
      if (!selectedFile) throw new Error('请选择文件');

      // 上传文件到后端
      const uploadResult = await files.upload(selectedFile);
      const fileSizeBytes = selectedFile.size;

      const versionResult = await createMaterialVersion({
        materialId,
        filePath: uploadResult.id,
        fileName: uploadResult.original_name,
        fileSizeBytes,
        description,
        changeNotes
      });

      return {
        fileName: selectedFile.name,
        fileSize: versionResult.version.file_size_label,
      };
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['application_materials'] });
      queryClient.invalidateQueries({ queryKey: ['material-versions'] });

      toast({
        title: "上传成功",
        description: `${data.fileName} (${data.fileSize}) 已上传并创建版本记录`
      });

      // 重置表单
      setSelectedFile(null);
      setDescription("");
      setChangeNotes("");
      onOpenChange(false);
    },
    onError: (error: Error) => {
      toast({
        title: "上传失败",
        description: error.message,
        variant: "destructive"
      });
    }
  });

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // 验证文件类型
    const validTypes = [
      'application/pdf',
      'application/msword',
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'application/vnd.ms-excel',
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      'image/jpeg',
      'image/png',
      'image/jpg'
    ];

    if (!validTypes.includes(file.type)) {
      toast({
        title: "文件类型不支持",
        description: "仅支持 PDF、Word、Excel 和图片格式",
        variant: "destructive"
      });
      return;
    }

    // 验证文件大小 (20MB)
    if (file.size > 20 * 1024 * 1024) {
      toast({
        title: "文件过大",
        description: "文件大小不能超过 20MB",
        variant: "destructive"
      });
      return;
    }

    setSelectedFile(file);
  };

  const handleSubmit = () => {
    if (!selectedFile) {
      toast({
        title: "请选择文件",
        variant: "destructive"
      });
      return;
    }

    uploadMutation.mutate();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>上传材料</DialogTitle>
          <DialogDescription>
            为 "{materialName}" 上传文件，系统将自动创建版本记录
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* 文件选择 */}
          <div className="space-y-2">
            <Label>选择文件 *</Label>
            <div className="border-2 border-dashed rounded-lg p-6 text-center">
              <input
                ref={fileInputRef}
                type="file"
                onChange={handleFileChange}
                className="hidden"
                accept=".pdf,.doc,.docx,.xls,.xlsx,.jpg,.jpeg,.png"
              />
              {selectedFile ? (
                <div className="flex items-center justify-center gap-2">
                  <FileText className="h-8 w-8 text-primary" />
                  <div className="text-left">
                    <div className="font-medium">{selectedFile.name}</div>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setSelectedFile(null)}
                  >
                    更换
                  </Button>
                </div>
              ) : (
                <>
                  <Upload className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                  <p className="text-sm text-muted-foreground mb-4">
                    支持 PDF、Word、Excel、图片格式，最大 20MB
                  </p>
                  <Button onClick={() => fileInputRef.current?.click()}>
                    选择文件
                  </Button>
                </>
              )}
            </div>
          </div>

          {/* 描述 */}
          <div className="space-y-2">
            <Label htmlFor="description">文件描述</Label>
            <Textarea
              id="description"
              placeholder="选填：简要描述该材料的内容..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
            />
          </div>

          {/* 变更说明 */}
          <div className="space-y-2">
            <Label htmlFor="changeNotes">版本变更说明</Label>
            <Textarea
              id="changeNotes"
              placeholder="选填：说明本次上传的变更内容，例如：更新了财务数据、修正了错别字等..."
              value={changeNotes}
              onChange={(e) => setChangeNotes(e.target.value)}
              rows={2}
            />
            <p className="text-xs text-muted-foreground">
              提供变更说明有助于后续版本对比和追溯
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={!selectedFile || uploadMutation.isPending}
          >
            {uploadMutation.isPending ? '上传中...' : '上传'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
