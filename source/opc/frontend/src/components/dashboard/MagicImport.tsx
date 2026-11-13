import { useState, useCallback } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { api } from "@/integrations/api/client";
import { toast } from "sonner";
import { useQuery } from "@tanstack/react-query";
import {
  Upload,
  FileSpreadsheet,
  CheckCircle2,
  Sparkles,
  FileText,
  Image,
  Mail,
  Loader2,
  Wand2,
  AlertCircle
} from "lucide-react";
import { Progress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  BackendImportOptions,
  BackendImportResult,
  fileToBase64,
  IMPORT_OPTIONS_ENDPOINT,
  isSupportedBackendImportFile,
} from "@/lib/backendImport";

interface MagicImportProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
}

export const MagicImport = ({ open, onOpenChange, onSuccess }: MagicImportProps) => {
  const [isDragging, setIsDragging] = useState(false);
  const [file, setFile] = useState<File | null>(null);
  const [selectedTable, setSelectedTable] = useState("");
  const [importing, setImporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [result, setResult] = useState<BackendImportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { data: importOptions = { table_options: [] } } =
    useQuery<BackendImportOptions>({
      queryKey: ["backend-import-options"],
      queryFn: () => api.get<BackendImportOptions>(IMPORT_OPTIONS_ENDPOINT),
      enabled: open,
    });

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const droppedFile = e.dataTransfer.files[0];
    if (droppedFile) {
      handleFile(droppedFile);
    }
  }, []);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (selectedFile) {
      handleFile(selectedFile);
    }
  };

  const handleFile = (selectedFile: File) => {
    if (!isSupportedBackendImportFile(selectedFile)) {
      const message = "请上传 CSV 文件";
      setError(message);
      toast.error(message);
      return;
    }

    setFile(selectedFile);
    setError(null);
    setResult(null);
  };

  const handleImport = async () => {
    if (!file || !selectedTable) {
      toast.error("请选择数据类型和文件");
      return;
    }

    setImporting(true);
    setProgress(20);
    setError(null);

    try {
      const selectedOption = importOptions.table_options.find(
        (option) => option.value === selectedTable,
      );
      if (!selectedOption?.import_action) {
        throw new Error("后端未返回导入动作");
      }
      const fileBase64 = await fileToBase64(file);
      setProgress(60);
      const importResult = await api.runAction<BackendImportResult>(
        selectedOption.import_action,
        {
          filename: file.name,
          file_base64: fileBase64,
        },
      );
      setProgress(100);
      setResult(importResult);

      if (importResult.success > 0) {
        if (importResult.failed > 0) {
          toast.warning(`成功导入 ${importResult.success} 条，失败 ${importResult.failed} 条`);
        } else {
          toast.success(`成功导入 ${importResult.success} 条数据`);
        }
        onSuccess?.();
      } else {
        toast.error("导入失败，请检查数据格式");
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : "导入失败";
      console.error("导入错误:", err);
      setError(message);
      toast.error(message);
    } finally {
      setImporting(false);
    }
  };

  const handleClose = () => {
    setFile(null);
    setSelectedTable("");
    setProgress(0);
    setResult(null);
    setError(null);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[600px] overflow-hidden">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <div className="p-2 rounded-lg bg-gradient-to-br from-primary/20 to-purple-500/20">
              <Wand2 className="h-5 w-5 text-primary" />
            </div>
            魔法导入
          </DialogTitle>
          <DialogDescription>
            选择文件后交给后端识别字段、校验并导入
          </DialogDescription>
        </DialogHeader>

        <div className="py-4 space-y-4">
          <Select value={selectedTable} onValueChange={setSelectedTable} disabled={importing}>
            <SelectTrigger>
              <SelectValue placeholder="选择导入类型" />
            </SelectTrigger>
            <SelectContent>
              {importOptions.table_options.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <div
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            className={cn(
              "relative border-2 border-dashed rounded-xl p-8 text-center transition-all duration-300",
              isDragging
                ? "border-primary bg-primary/5 scale-[1.02]"
                : "border-muted-foreground/20 hover:border-primary/50",
              importing && "pointer-events-none opacity-50"
            )}
          >
            <div className="space-y-4">
              <div className="mx-auto w-16 h-16 rounded-full bg-gradient-to-br from-primary/10 to-purple-500/10 flex items-center justify-center">
                {importing ? (
                  <Loader2 className="h-8 w-8 text-primary animate-spin" />
                ) : (
                  <Upload className={cn(
                    "h-8 w-8 text-primary transition-transform duration-300",
                    isDragging && "scale-110"
                  )} />
                )}
              </div>
              <div>
                <p className="text-lg font-medium">{file ? file.name : "拖拽文件到这里"}</p>
                <p className="text-sm text-muted-foreground mt-1">
                  或点击选择文件 · 支持 CSV
                </p>
              </div>
              <Button
                variant="outline"
                onClick={() => document.getElementById("magic-file-upload")?.click()}
                className="gap-2"
                disabled={importing}
              >
                <FileSpreadsheet className="h-4 w-4" />
                选择文件
              </Button>
              <input
                id="magic-file-upload"
                type="file"
                accept=".csv,text/csv"
                className="hidden"
                onChange={handleFileSelect}
              />
            </div>

            <div className="mt-6 pt-6 border-t border-muted grid grid-cols-3 gap-4 text-xs text-muted-foreground">
              <div className="flex flex-col items-center gap-2">
                <FileText className="h-5 w-5" />
                <span>发票/付款</span>
              </div>
              <div className="flex flex-col items-center gap-2">
                <Image className="h-5 w-5" />
                <span>资产/合同</span>
              </div>
              <div className="flex flex-col items-center gap-2">
                <Mail className="h-5 w-5" />
                <span>员工/工资</span>
              </div>
            </div>
          </div>

          {importing && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="flex items-center gap-2">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  后端导入中...
                </span>
                <span>{progress}%</span>
              </div>
              <Progress value={progress} className="h-2" />
            </div>
          )}

          {error && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {result && (
            <Alert className={result.failed === 0 ? "border-green-500" : "border-yellow-500"}>
              <CheckCircle2 className="h-4 w-4 text-green-500" />
              <AlertDescription>
                <div className="space-y-1">
                  <p>总行数: {result.total}</p>
                  <p>成功导入: {result.success} 条</p>
                  {result.failed > 0 && <p>导入失败: {result.failed} 条</p>}
                  {result.errors.slice(0, 3).map((error) => (
                    <p key={`${error.row}-${error.error}`} className="text-xs">
                      第 {error.row} 行：{error.error}
                    </p>
                  ))}
                </div>
              </AlertDescription>
            </Alert>
          )}

          <div className="flex justify-end gap-2 pt-2">
            <Button variant="outline" onClick={handleClose} disabled={importing}>
              取消
            </Button>
            <Button
              onClick={handleImport}
              disabled={importing || !file || !selectedTable}
              className="gap-2"
            >
              {importing ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Sparkles className="h-4 w-4" />
              )}
              确认导入
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
