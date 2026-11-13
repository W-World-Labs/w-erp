import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { api } from "@/integrations/api/client";
import { toast } from "sonner";
import { Upload, FileSpreadsheet, AlertCircle, CheckCircle2 } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useQuery } from "@tanstack/react-query";
import {
  BackendImportOptions,
  BackendImportResult,
  fileToBase64,
  IMPORT_OPTIONS_ENDPOINT,
  isSupportedBackendImportFile,
} from "@/lib/backendImport";

interface ImportDataDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export const ImportDataDialog = ({ open, onOpenChange }: ImportDataDialogProps) => {
  const [selectedTable, setSelectedTable] = useState<string>("");
  const [file, setFile] = useState<File | null>(null);
  const [importing, setImporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [result, setResult] = useState<BackendImportResult | null>(null);
  const { data: importOptions = { table_options: [] } } =
    useQuery<BackendImportOptions>({
      queryKey: ["backend-import-options"],
      queryFn: () => api.get<BackendImportOptions>(IMPORT_OPTIONS_ENDPOINT),
      enabled: open,
    });

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (!selectedFile) {
      return;
    }

    if (!isSupportedBackendImportFile(selectedFile)) {
      toast.error("请上传 CSV 文件");
      return;
    }

    setFile(selectedFile);
    setResult(null);
  };

  const handleImport = async () => {
    if (!file || !selectedTable) {
      toast.error("请选择数据表和文件");
      return;
    }

    setImporting(true);
    setProgress(20);
    setResult(null);

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

      if (importResult.failed === 0) {
        toast.success(`成功导入 ${importResult.success} 条数据`);
      } else {
        toast.warning(`成功 ${importResult.success} 条，失败 ${importResult.failed} 条`);
      }
    } catch (error) {
      console.error("导入错误:", error);
      toast.error("导入失败: " + (error as Error).message);
    } finally {
      setImporting(false);
    }
  };

  const handleClose = () => {
    setFile(null);
    setSelectedTable("");
    setProgress(0);
    setResult(null);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            批量导入数据
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="table">选择数据表</Label>
            <Select value={selectedTable} onValueChange={setSelectedTable} disabled={importing}>
              <SelectTrigger>
                <SelectValue placeholder="选择要导入的数据表" />
              </SelectTrigger>
              <SelectContent>
                {importOptions.table_options.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="file">上传文件</Label>
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                className="w-full"
                onClick={() => document.getElementById("file-upload")?.click()}
                disabled={importing}
              >
                <FileSpreadsheet className="mr-2 h-4 w-4" />
                {file ? file.name : "选择 CSV 文件"}
              </Button>
              <input
                id="file-upload"
                type="file"
                accept=".csv,text/csv"
                className="hidden"
                onChange={handleFileChange}
                disabled={importing}
              />
            </div>
            <p className="text-xs text-muted-foreground">
              文件会上传到后端，由后端解析、校验并写入数据。
            </p>
          </div>

          {importing && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span>导入进度</span>
                <span>{progress}%</span>
              </div>
              <Progress value={progress} />
            </div>
          )}

          {result && (
            <Alert className={result.failed === 0 ? "border-green-500" : "border-yellow-500"}>
              <div className="flex items-start gap-2">
                {result.failed === 0 ? (
                  <CheckCircle2 className="h-4 w-4 text-green-500 mt-0.5" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-yellow-500 mt-0.5" />
                )}
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
              </div>
            </Alert>
          )}

          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription className="text-xs">
              <p className="font-medium mb-1">导入提示：</p>
              <ul className="list-disc list-inside space-y-1">
                <li>第一行为字段名，字段映射和必填校验由后端执行</li>
                <li>日期格式建议使用: YYYY-MM-DD</li>
                <li>数值字段不要包含货币符号</li>
              </ul>
            </AlertDescription>
          </Alert>
        </div>

        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={handleClose} disabled={importing}>
            取消
          </Button>
          <Button onClick={handleImport} disabled={!file || !selectedTable || importing}>
            {importing ? "导入中..." : "开始导入"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
};
