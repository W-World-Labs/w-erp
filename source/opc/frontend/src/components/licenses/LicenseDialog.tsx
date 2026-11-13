import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useQuery } from "@tanstack/react-query";
import { api, files, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { Upload, File, X, FileText, Image as ImageIcon, Sparkles, Loader2 } from "lucide-react";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useDocumentParser } from "@/hooks/useDocumentParser";

interface LicenseDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  license?: any;
  action?: BackendAction | null;
  onSuccess: () => void;
}

interface LicenseFormOption {
  value: string;
  label: string;
}

interface LicenseFormOptions {
  license_type_options: LicenseFormOption[];
  status_options: LicenseFormOption[];
  default_values: {
    license_type: string;
    status: string;
  };
}

export function LicenseDialog({ open, onOpenChange, license, action: createAction, onSuccess }: LicenseDialogProps) {
  const { register, handleSubmit, reset, setValue, watch } = useForm();
  const [uploading, setUploading] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [currentFilePath, setCurrentFilePath] = useState<string | null>(null);
  const [currentFileName, setCurrentFileName] = useState<string | null>(null);
  const { parseFile, isParsing, error: parseError, enabled: parserEnabled } = useDocumentParser('license');
  const { data: formOptions } = useQuery({
    queryKey: ["license-form-options"],
    queryFn: () => api.get<LicenseFormOptions>("/api/v1/licenses/form_options"),
    enabled: open,
  });
  const licenseTypeOptions = formOptions?.license_type_options ?? [];
  const statusOptions = formOptions?.status_options ?? [];
  const selectedLicenseType = watch("license_type") || "";
  const selectedStatus = watch("status") || "";

  const fillableFields = [
    'license_type',
    'license_name',
    'license_number',
    'issuing_authority',
    'issue_date',
    'expiry_date',
    'notes',
  ];

  const handleAutoParse = async () => {
    if (!selectedFile) return;
    const fields = await parseFile(selectedFile);
    if (fields) {
      let filled = 0;
      for (const [key, value] of Object.entries(fields)) {
        if (value && fillableFields.includes(key)) {
          setValue(key, value);
          filled++;
        }
      }
      toast.success(filled > 0
        ? `AI 识别完成，已填入 ${filled} 个字段，请核对后提交`
        : '未能从文件中识别到信息，请尝试其他文件');
    }
  };

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    reset(license ? {
      ...license,
      license_type: license.license_type || defaults.license_type,
      status: license.status || defaults.status,
      notes: license.notes || "",
    } : {
      license_type: defaults.license_type,
      license_name: "",
      license_number: "",
      issuing_authority: "",
      issue_date: "",
      expiry_date: "",
      status: defaults.status,
      notes: "",
    });
    setCurrentFilePath(license?.file_path || null);
    setCurrentFileName(license?.file_name || null);
    setSelectedFile(null);
  }, [open, license, formOptions?.default_values, reset]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // 验证文件类型
    const validTypes = ['image/jpeg', 'image/jpg', 'image/png', 'application/pdf'];
    if (!validTypes.includes(file.type)) {
      toast.error("只支持 JPG、PNG 和 PDF 格式文件");
      return;
    }

    // 验证文件大小 (10MB)
    if (file.size > 10 * 1024 * 1024) {
      toast.error("文件大小不能超过 10MB");
      return;
    }

    setSelectedFile(file);
  };

  const uploadFile = async (file: File): Promise<{ path: string; name: string }> => {
    const uploadResult = await files.upload(file);
    return { path: uploadResult.id, name: uploadResult.original_name };
  };

  const handleRemoveFile = () => {
    setSelectedFile(null);
  };

  const handleDeleteCurrentFile = async () => {
    if (!currentFilePath) return;

    // File cleanup handled by backend;

    setCurrentFilePath(null);
    setCurrentFileName(null);
    toast.success("文件已删除");
  };

  const onSubmit = async (data: any) => {
    // Auth handled by backend
    setUploading(true);

    try {
      let filePath = currentFilePath;
      let fileName = currentFileName;

      // 如果有新文件，先上传
      if (selectedFile) {
        const uploadResult = await uploadFile(selectedFile);
        filePath = uploadResult.path;
        fileName = uploadResult.name;

        // 如果是更新且有旧文件，删除旧文件
        if (license && currentFilePath) {
          // File cleanup handled by backend;
        }
      }

      const licenseData = {
        ...data,
        file_path: filePath,
        file_name: fileName,
      };

      const action = license?.update_action ?? createAction;
      if (!action) {
        toast.error("当前无法保存证照");
        return;
      }

      await api.runAction(action, licenseData);
      toast.success(action.success_message || "证照已保存");

      onSuccess();
      onOpenChange(false);
    } catch (error) {
      console.error('Submit error:', error);
      toast.error("操作失败");
    } finally {
      setUploading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{license ? "编辑证照" : "添加证照"}</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label>证照类型 *</Label>
              <Select onValueChange={(value) => setValue("license_type", value)} value={selectedLicenseType}>
                <SelectTrigger>
                  <SelectValue placeholder="选择证照类型" />
                </SelectTrigger>
                <SelectContent>
                  {licenseTypeOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <Label>证照名称 *</Label>
              <Input {...register("license_name", { required: true })} placeholder="如：工商营业执照" />
            </div>

            <div className="space-y-2">
              <Label>证照编号 *</Label>
              <Input {...register("license_number", { required: true })} placeholder="证照编号" />
            </div>

            <div className="space-y-2">
              <Label>发证机关</Label>
              <Input {...register("issuing_authority")} placeholder="发证机关" />
            </div>

            <div className="space-y-2">
              <Label>发证日期 *</Label>
              <Input type="date" {...register("issue_date", { required: true })} />
            </div>

            <div className="space-y-2">
              <Label>到期日期</Label>
              <Input type="date" {...register("expiry_date")} />
            </div>

            <div className="space-y-2">
              <Label>状态</Label>
              <Select onValueChange={(value) => setValue("status", value)} value={selectedStatus}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {statusOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          <div className="space-y-2">
            <Label>备注</Label>
            <Textarea {...register("notes")} placeholder="备注信息" rows={3} />
          </div>

          <div className="space-y-2">
            <Label>上传文件</Label>

            {/* 当前文件显示 */}
            {currentFilePath && !selectedFile && (
              <div className="flex items-center gap-2 p-3 border rounded-md bg-muted/50">
                {currentFileName?.endsWith('.pdf') ? (
                  <FileText className="h-5 w-5 text-red-500" />
                ) : (
                  <ImageIcon className="h-5 w-5 text-blue-500" />
                )}
                <span className="flex-1 text-sm truncate">{currentFileName}</span>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={handleDeleteCurrentFile}
                  disabled={uploading}
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>
            )}

            {/* 选中的新文件显示 */}
            {selectedFile && (
              <div className="space-y-2">
                <div className="flex items-center gap-2 p-3 border rounded-md bg-primary/5">
                  {selectedFile.type === 'application/pdf' ? (
                    <FileText className="h-5 w-5 text-red-500" />
                  ) : (
                    <ImageIcon className="h-5 w-5 text-blue-500" />
                  )}
                  <span className="flex-1 text-sm truncate">{selectedFile.name}</span>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={handleRemoveFile}
                    disabled={uploading || isParsing}
                  >
                    <X className="h-4 w-4" />
                  </Button>
                </div>
                {parserEnabled && (
                  <>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      className="w-full"
                      onClick={handleAutoParse}
                      disabled={isParsing || uploading}
                    >
                      {isParsing ? (
                        <><Loader2 className="mr-2 h-4 w-4 animate-spin" />识别中...</>
                      ) : (
                        <><Sparkles className="mr-2 h-4 w-4" />✨ AI 智能识别</>
                      )}
                    </Button>
                    {parseError && (
                      <p className="text-xs text-destructive">{parseError}</p>
                    )}
                  </>
                )}
              </div>
            )}

            {/* 上传按钮 */}
            {!selectedFile && (
              <div>
                <Button
                  type="button"
                  variant="outline"
                  className="w-full"
                  onClick={() => document.getElementById('license-file-upload')?.click()}
                  disabled={uploading}
                >
                  <Upload className="mr-2 h-4 w-4" />
                  {currentFilePath ? "更换文件" : "选择文件"}
                </Button>
                <input
                  id="license-file-upload"
                  type="file"
                  accept="image/jpeg,image/jpg,image/png,application/pdf"
                  className="hidden"
                  onChange={handleFileChange}
                  disabled={uploading}
                />
              </div>
            )}

            <Alert>
              <File className="h-4 w-4" />
              <AlertDescription className="text-xs">
                支持 JPG、PNG、PDF 格式，文件大小不超过 10MB
              </AlertDescription>
            </Alert>
          </div>

          <div className="flex justify-end gap-2">
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)} disabled={uploading}>
              取消
            </Button>
            <Button type="submit" disabled={uploading}>
              {uploading ? "处理中..." : license ? "更新" : "添加"}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
