import { useState } from "react";
import { toast } from "sonner";
import { Download, FileText, Loader2, Eye } from "lucide-react";
import { files } from "@/integrations/api/client";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { FilePreviewDialog } from "@/components/common/FilePreviewDialog";

type BadgeVariant = "secondary" | "default" | "destructive" | "outline";

interface InvoiceBadge {
  label: string;
  variant: BadgeVariant;
}

interface Invoice {
  id: string;
  invoice_number: string;
  invoice_type: string | null;
  amount: number;
  amount_label?: string;
  tax_amount: number | null;
  tax_amount_label?: string;
  total_with_tax_label?: string;
  invoice_date: string;
  invoice_date_label?: string;
  due_date: string | null;
  due_date_label?: string;
  status: string;
  status_badge?: InvoiceBadge;
  supplier: string | null;
  description: string | null;
  file_path: string | null;
  file_name?: string | null;
  file_label?: string | null;
  has_file?: boolean;
  created_by: string | null;
  created_at: string;
  created_at_label?: string;
  updated_at: string;
}

interface InvoiceDetailDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  invoice: Invoice | null;
}

export function InvoiceDetailDialog({
  open,
  onOpenChange,
  invoice,
}: InvoiceDetailDialogProps) {
  const [downloading, setDownloading] = useState(false);
  const [showPreview, setShowPreview] = useState(false);

  if (!invoice) return null;

  const handleDownload = async () => {
    if (!invoice.file_path) {
      toast.error("该发票没有附件");
      return;
    }

    setDownloading(true);
    try {
      const data = await files.download(invoice.file_path);

      // 创建下载链接
      const url = window.URL.createObjectURL(data);
      const a = document.createElement("a");
      a.href = url;
      a.download = invoice.file_name || invoice.file_label || "invoice";
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);

      toast.success("文件下载成功");
    } catch (error) {
      console.error("下载文件失败:", error);
      toast.error("下载失败，请重试");
    } finally {
      setDownloading(false);
    }
  };

  const InfoRow = ({
    label,
    value,
  }: {
    label: string;
    value: string | number | null | undefined;
  }) => (
    <div className="flex justify-between py-2">
      <span className="text-sm text-muted-foreground">{label}</span>
      <span className="text-sm font-medium">{value || "-"}</span>
    </div>
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>发票详情</DialogTitle>
          <DialogDescription>
            发票号：{invoice.invoice_number}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* 基本信息 */}
          <div>
            <h3 className="font-semibold mb-3">基本信息</h3>
            <div className="space-y-1">
              <InfoRow label="发票描述" value={invoice.description} />
              <InfoRow label="供应商" value={invoice.supplier} />
              <InfoRow label="发票类型" value={invoice.invoice_type} />
              <div className="flex justify-between py-2">
                <span className="text-sm text-muted-foreground">状态</span>
                {invoice.status_badge ? (
                  <Badge variant={invoice.status_badge.variant}>
                    {invoice.status_badge.label}
                  </Badge>
                ) : (
                  <span className="text-sm font-medium">-</span>
                )}
              </div>
            </div>
          </div>

          <Separator />

          {/* 金额信息 */}
          <div>
            <h3 className="font-semibold mb-3">金额信息</h3>
            <div className="space-y-1">
              <InfoRow label="金额" value={invoice.amount_label} />
              <InfoRow label="税额" value={invoice.tax_amount_label} />
              {invoice.total_with_tax_label && (
                <InfoRow label="合计" value={invoice.total_with_tax_label} />
              )}
            </div>
          </div>

          <Separator />

          {/* 日期信息 */}
          <div>
            <h3 className="font-semibold mb-3">日期信息</h3>
            <div className="space-y-1">
              <InfoRow label="开票日期" value={invoice.invoice_date_label} />
              <InfoRow label="到期日期" value={invoice.due_date_label} />
              <InfoRow label="创建时间" value={invoice.created_at_label} />
            </div>
          </div>

          {/* 描述 */}
          {invoice.description && (
            <>
              <Separator />
              <div>
                <h3 className="font-semibold mb-3">描述</h3>
                <p className="text-sm text-muted-foreground whitespace-pre-wrap">
                  {invoice.description}
                </p>
              </div>
            </>
          )}

          {/* 附件 */}
          {invoice.file_path && (
            <>
              <Separator />
              <div>
                <h3 className="font-semibold mb-3">附件</h3>
                <div className="flex items-center justify-between p-3 border rounded-md">
                  <div className="flex items-center gap-2">
                    <FileText className="h-5 w-5 text-muted-foreground" />
                    <span className="text-sm">
                      {invoice.file_label || invoice.file_name || "附件"}
                    </span>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => setShowPreview(true)}
                    >
                      <Eye className="h-4 w-4 mr-2" />
                      预览
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={handleDownload}
                      disabled={downloading}
                    >
                      {downloading ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        <>
                          <Download className="h-4 w-4 mr-2" />
                          下载
                        </>
                      )}
                    </Button>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>

        {/* 文件预览对话框 */}
        <FilePreviewDialog
          open={showPreview}
          onOpenChange={setShowPreview}
          filePath={invoice.file_path}
          fileName={invoice.file_name || invoice.file_label || "附件"}
          bucketName="invoices"
        />
      </DialogContent>
    </Dialog>
  );
}
