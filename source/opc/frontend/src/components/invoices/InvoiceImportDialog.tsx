import { useState } from "react";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import {
  Upload,
  FileText,
  Download,
  AlertCircle,
  CheckCircle2,
  Loader2,
} from "lucide-react";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface InvoiceImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: () => void;
  previewAction?: BackendAction | null;
  commitAction?: BackendAction | null;
}

type ImportFormat = "csv" | "xml";

interface ImportPayload {
  filename: string;
  file_base64: string;
  import_format: ImportFormat;
}

interface DisplayBadge {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  icon_key?: string;
  class_name?: string;
}

interface InvoiceImportRow {
  row_number: number;
  invoice_number: string;
  title: string;
  supplier: string;
  amount_label: string;
  tax_amount_label: string;
  invoice_date: string;
  error: string;
  status_badge: DisplayBadge;
}

interface InvoiceImportStats {
  total_label: string;
  valid_label: string;
  invalid_label: string;
  duplicate_label: string;
}

interface InvoiceImportPreview {
  rows: InvoiceImportRow[];
  stats: InvoiceImportStats;
  can_import: boolean;
  import_button_label: string;
  preview_message: string;
  progress_value: number;
  progress_label: string;
}

interface InvoiceImportCommit extends InvoiceImportPreview {
  success_message: string;
  should_close: boolean;
}

const statusIcons = {
  AlertCircle,
  CheckCircle2,
};

export function InvoiceImportDialog({
  open,
  onOpenChange,
  onSuccess,
  previewAction,
  commitAction,
}: InvoiceImportDialogProps) {
  const [importing, setImporting] = useState(false);
  const [progressValue, setProgressValue] = useState(0);
  const [progressLabel, setProgressLabel] = useState("0%");
  const [preview, setPreview] = useState<InvoiceImportPreview | null>(null);
  const [activeFile, setActiveFile] = useState<ImportPayload | null>(null);
  const [importFormat, setImportFormat] = useState<ImportFormat>("csv");

  const downloadTemplate = () => {
    const template = [
      "invoice_number,title,vendor_name,amount,tax_amount,invoice_date,description,invoice_type",
      "INV-2024-001,办公设备采购,XX科技有限公司,10000,1300,2024-01-15,备注说明,input",
    ].join("\n");
    const blob = new Blob([`\uFEFF${template}`], {
      type: "text/csv;charset=utf-8",
    });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "发票导入模板.csv";
    link.click();
    URL.revokeObjectURL(url);
    toast.success("模板下载成功");
  };

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setImporting(true);
    setProgressValue(20);
    setProgressLabel("上传中");

    try {
      if (!previewAction) {
        throw new Error("后端未返回发票导入预览动作");
      }
      const payload = {
        filename: file.name,
        file_base64: await readFileAsDataUrl(file),
        import_format: importFormat,
      };
      const result = await api.runAction<InvoiceImportPreview>(
        previewAction,
        payload,
      );
      setActiveFile(payload);
      setPreview(result);
      setProgressValue(result.progress_value);
      setProgressLabel(result.progress_label);
      toast.success(result.preview_message);
    } catch (error) {
      console.error("文件解析失败:", error);
      toast.error("文件解析失败，请检查文件格式");
      setPreview(null);
      setActiveFile(null);
    } finally {
      setImporting(false);
    }
  };

  const handleImport = async () => {
    if (!activeFile || !preview?.can_import) {
      toast.error("没有有效的发票数据可导入");
      return;
    }

    setImporting(true);
    setProgressValue(50);
    setProgressLabel("导入中");

    try {
      if (!commitAction) {
        throw new Error("后端未返回发票导入提交动作");
      }
      const result = await api.runAction<InvoiceImportCommit>(
        commitAction,
        activeFile,
      );
      setPreview(result);
      setProgressValue(result.progress_value);
      setProgressLabel(result.progress_label);
      toast.success(result.success_message);
      if (result.should_close) {
        onSuccess();
        onOpenChange(false);
        setPreview(null);
        setActiveFile(null);
      }
    } catch (error) {
      console.error("批量导入失败:", error);
      toast.error("导入失败，请重试");
    } finally {
      setImporting(false);
    }
  };

  const handleClear = () => {
    setPreview(null);
    setActiveFile(null);
    setProgressValue(0);
    setProgressLabel("0%");
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-5xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>批量导入发票</DialogTitle>
          <DialogDescription>
            支持CSV和XML格式批量导入，解析校验由后端统一完成
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          <Tabs
            value={importFormat}
            onValueChange={(v) => setImportFormat(v as ImportFormat)}
          >
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="csv">CSV 格式</TabsTrigger>
              <TabsTrigger value="xml">XML 格式（电子发票）</TabsTrigger>
            </TabsList>

            <TabsContent value="csv" className="space-y-4 mt-4">
              <Alert>
                <FileText className="h-4 w-4" />
                <AlertDescription>
                  请上传包含发票数据的CSV文件。建议先下载模板填写。
                </AlertDescription>
              </Alert>

              <div className="flex gap-2">
                <Button variant="outline" onClick={downloadTemplate}>
                  <Download className="mr-2 h-4 w-4" />
                  下载导入模板
                </Button>

                <label className="flex-1">
                  <Button
                    variant="outline"
                    className="w-full"
                    disabled={importing}
                    asChild
                  >
                    <span>
                      <Upload className="mr-2 h-4 w-4" />
                      {importing ? "处理中..." : "选择文件上传"}
                    </span>
                  </Button>
                  <input
                    type="file"
                    accept=".csv"
                    onChange={handleFileUpload}
                    className="hidden"
                    disabled={importing}
                  />
                </label>
              </div>
            </TabsContent>

            <TabsContent value="xml" className="space-y-4 mt-4">
              <Alert>
                <FileText className="h-4 w-4" />
                <AlertDescription>
                  支持税务局标准电子发票XML格式。
                </AlertDescription>
              </Alert>

              <label>
                <Button
                  variant="outline"
                  className="w-full"
                  disabled={importing}
                  asChild
                >
                  <span>
                    <Upload className="mr-2 h-4 w-4" />
                    {importing ? "处理中..." : "选择XML文件"}
                  </span>
                </Button>
                <input
                  type="file"
                  accept=".xml"
                  onChange={handleFileUpload}
                  className="hidden"
                  disabled={importing}
                />
              </label>
            </TabsContent>
          </Tabs>

          {importing && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span>处理中...</span>
                <span>{progressLabel}</span>
              </div>
              <Progress value={progressValue} />
            </div>
          )}

          {preview && (
            <>
              <div className="grid grid-cols-4 gap-4">
                <ImportStatCard value={preview.stats.total_label} label="总数" />
                <ImportStatCard
                  value={preview.stats.valid_label}
                  label="有效"
                  className="text-green-600"
                />
                <ImportStatCard
                  value={preview.stats.invalid_label}
                  label="无效"
                  className="text-red-600"
                />
                <ImportStatCard
                  value={preview.stats.duplicate_label}
                  label="重复"
                  className="text-yellow-600"
                />
              </div>

              <div className="border rounded-lg max-h-96 overflow-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>状态</TableHead>
                      <TableHead>发票号</TableHead>
                      <TableHead>标题</TableHead>
                      <TableHead>供应商</TableHead>
                      <TableHead>金额</TableHead>
                      <TableHead>开票日期</TableHead>
                      <TableHead>错误信息</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {preview.rows.map((row) => (
                      <TableRow key={`${row.row_number}-${row.invoice_number}`}>
                        <TableCell>
                          <ImportStatusBadge badge={row.status_badge} />
                        </TableCell>
                        <TableCell className="font-mono text-sm">
                          {row.invoice_number}
                        </TableCell>
                        <TableCell>{row.title}</TableCell>
                        <TableCell>{row.supplier}</TableCell>
                        <TableCell>{row.amount_label}</TableCell>
                        <TableCell>{row.invoice_date}</TableCell>
                        <TableCell className="text-red-600 text-sm">
                          {row.error}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>

              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={handleClear}>
                  清除
                </Button>
                <Button
                  onClick={handleImport}
                  disabled={importing || !preview.can_import}
                >
                  {importing ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      导入中...
                    </>
                  ) : (
                    preview.import_button_label
                  )}
                </Button>
              </div>
            </>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}

function ImportStatusBadge({ badge }: { badge: DisplayBadge }) {
  const Icon = badge.icon_key
    ? statusIcons[badge.icon_key as keyof typeof statusIcons]
    : null;
  return (
    <Badge variant={badge.variant || "outline"} className={badge.class_name || ""}>
      {Icon && <Icon className="h-3 w-3 mr-1" />}
      {badge.label}
    </Badge>
  );
}

function ImportStatCard({
  value,
  label,
  className = "",
}: {
  value: string;
  label: string;
  className?: string;
}) {
  return (
    <div className="rounded-lg border bg-card text-card-foreground shadow-sm p-4">
      <div className={`text-2xl font-bold ${className}`}>{value}</div>
      <div className="text-sm text-muted-foreground">{label}</div>
    </div>
  );
}

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(new Error("文件读取失败"));
    reader.readAsDataURL(file);
  });
}
