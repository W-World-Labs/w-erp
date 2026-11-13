import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { AlertCircle, Calendar } from "lucide-react";
import { Card } from "@/components/ui/card";

interface InvoiceReminderDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface UrgencyBadge {
  label: string;
  variant: "default" | "secondary" | "destructive" | "outline";
}

interface PendingInvoice {
  id: string;
  payment_no: string;
  payment_date_label: string;
  amount_label: string;
  payer: string;
  payee: string;
  invoice_deadline_label: string;
  payment_method_label: string;
  urgency_badge: UrgencyBadge;
}

interface PaymentsOverview {
  invoice_reminder_stats: {
    total: number;
    overdue: number;
    urgent: number;
    total_amount_label: string;
  };
  pending_invoice_payments: PendingInvoice[];
}

const emptyPaymentsOverview: PaymentsOverview = {
  invoice_reminder_stats: {
    total: 0,
    overdue: 0,
    urgent: 0,
    total_amount_label: "¥0",
  },
  pending_invoice_payments: [],
};

export function InvoiceReminderDialog({
  open,
  onOpenChange,
}: InvoiceReminderDialogProps) {
  const { data: overview = emptyPaymentsOverview, isLoading } = useQuery({
    queryKey: ["payments-invoice-reminders"],
    queryFn: () => api.get<PaymentsOverview>("/api/payments/overview"),
    enabled: open,
  });

  const pendingInvoices = overview.pending_invoice_payments;
  const stats = overview.invoice_reminder_stats;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-6xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertCircle className="w-5 h-5 text-orange-600" />
            追票提醒
          </DialogTitle>
        </DialogHeader>

        <div className="grid gap-4 md:grid-cols-4 mb-4">
          <Card className="p-4">
            <div className="text-sm text-muted-foreground">待追发票</div>
            <div className="text-2xl font-bold mt-1">{stats.total}</div>
          </Card>
          <Card className="p-4">
            <div className="text-sm text-muted-foreground">已逾期</div>
            <div className="text-2xl font-bold mt-1 text-red-600">
              {stats.overdue}
            </div>
          </Card>
          <Card className="p-4">
            <div className="text-sm text-muted-foreground">紧急(3天内)</div>
            <div className="text-2xl font-bold mt-1 text-orange-600">
              {stats.urgent}
            </div>
          </Card>
          <Card className="p-4">
            <div className="text-sm text-muted-foreground">涉及金额</div>
            <div className="text-2xl font-bold mt-1">
              {stats.total_amount_label}
            </div>
          </Card>
        </div>

        {isLoading ? (
          <div className="text-center py-8 text-muted-foreground">
            加载中...
          </div>
        ) : pendingInvoices.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <AlertCircle className="w-12 h-12 mx-auto mb-2 text-green-600" />
            <p>太棒了！没有待追发票</p>
          </div>
        ) : (
          <div className="border rounded-lg">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>支付编号</TableHead>
                  <TableHead>支付日期</TableHead>
                  <TableHead>付款方</TableHead>
                  <TableHead>收款方</TableHead>
                  <TableHead>支付方式</TableHead>
                  <TableHead className="text-right">金额</TableHead>
                  <TableHead>截止日期</TableHead>
                  <TableHead>紧急程度</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {pendingInvoices.map((payment) => (
                  <TableRow key={payment.id}>
                    <TableCell className="font-medium">
                      {payment.payment_no}
                    </TableCell>
                    <TableCell>{payment.payment_date_label}</TableCell>
                    <TableCell>{payment.payer || "-"}</TableCell>
                    <TableCell>{payment.payee || "-"}</TableCell>
                    <TableCell>{payment.payment_method_label}</TableCell>
                    <TableCell className="text-right font-medium">
                      {payment.amount_label}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-1">
                        <Calendar className="w-4 h-4" />
                        {payment.invoice_deadline_label}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={payment.urgency_badge?.variant || "outline"}
                      >
                        {payment.urgency_badge?.label}
                      </Badge>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
