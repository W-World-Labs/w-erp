import { useState } from "react";
import { Link } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  AlertCircle,
  CheckCircle,
  Clock,
  Edit,
  FileText,
  Plus,
  Scale,
  Search,
  Trash2,
  XCircle,
  type LucideIcon,
} from "lucide-react";
import { QueryStateGate } from "@/components/common/QueryStateGate";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { PaymentDialog } from "@/components/payments/PaymentDialog";
import { InvoiceReminderDialog } from "@/components/payments/InvoiceReminderDialog";
import { AccountsReceivableAgingPanel } from "@/components/payments/AccountsReceivableAgingPanel";
import { toast } from "sonner";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

interface PaymentBadge {
  label: string;
  variant: "default" | "secondary" | "destructive" | "outline";
  icon?: string;
  visible?: boolean;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
}

interface Payment {
  id: string;
  payment_no?: string;
  payment_date_label: string;
  payment_type_badge: PaymentBadge;
  payer?: string;
  payee?: string;
  payment_method_label: string;
  signed_amount_label: string;
  amount_class: string;
  invoice_badge: PaymentBadge;
  status_badge: PaymentBadge;
  can_delete: boolean;
  delete_action: DeleteAction | null;
  update_action: BackendAction | null;
}

interface PaymentOverview {
  stats: {
    total: number;
    expense: number;
    income: number;
    expense_label: string;
    income_label: string;
    pending_invoice: number;
    overdue_invoice: number;
  };
  payments: Payment[];
  payment_type_options: Array<{ value: string; label: string }>;
  payment_method_options: Array<{ value: string; label: string }>;
  invoice_status_options: Array<{ value: string; label: string }>;
  create_action: BackendAction | null;
  empty_message: string;
}

const PAYMENTS_OVERVIEW_ENDPOINT = "/api/payments/overview";

const emptyPaymentOverview: PaymentOverview = {
  stats: {
    total: 0,
    expense: 0,
    income: 0,
    expense_label: "¥0",
    income_label: "¥0",
    pending_invoice: 0,
    overdue_invoice: 0,
  },
  payments: [],
  payment_type_options: [],
  payment_method_options: [],
  invoice_status_options: [],
  create_action: null,
  empty_message: "暂无支付记录",
};

const badgeIcons: Record<string, LucideIcon> = {
  AlertCircle,
  CheckCircle,
  Clock,
  FileText,
  XCircle,
};

function PaymentBadgeView({ badge }: { badge: PaymentBadge }) {
  const Icon = badge.icon ? badgeIcons[badge.icon] : null;

  return (
    <Badge variant={badge.variant}>
      {Icon && <Icon className="mr-1 h-3 w-3" />}
      {badge.label}
    </Badge>
  );
}

function appendOverviewParam(
  params: URLSearchParams,
  key: string,
  value: string,
) {
  if (value && value !== "all") {
    params.set(key, value);
  }
}

export default function Payments() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [methodFilter, setMethodFilter] = useState<string>("all");
  const [invoiceFilter, setInvoiceFilter] = useState<string>("all");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedPayment, setSelectedPayment] = useState<Payment | undefined>();
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [paymentToDelete, setPaymentToDelete] = useState<DeleteAction | null>(
    null,
  );
  const [reminderDialogOpen, setReminderDialogOpen] = useState(false);

  const {
    data: paymentOverview = emptyPaymentOverview,
    isLoading,
    isError,
    refetch,
  } = useQuery({
    queryKey: [
      "payments-overview",
      search,
      typeFilter,
      methodFilter,
      invoiceFilter,
    ],
    queryFn: async () => {
      const params = new URLSearchParams();
      const searchValue = search.trim();
      if (searchValue) {
        params.set("search", searchValue);
      }
      appendOverviewParam(params, "payment_type", typeFilter);
      appendOverviewParam(params, "payment_method", methodFilter);
      appendOverviewParam(params, "invoice_status", invoiceFilter);
      const query = params.toString();
      return await api.get<PaymentOverview>(
        `${PAYMENTS_OVERVIEW_ENDPOINT}${query ? `?${query}` : ""}`,
      );
    },
  });
  const payments = paymentOverview.payments;
  const stats = paymentOverview.stats;

  const saveMutation = useMutation({
    mutationFn: async ({
      action,
      data,
    }: {
      action: BackendAction;
      data: Record<string, unknown>;
    }) => {
      await api.runAction(action, data);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["payments-overview"] });
      toast.success(action.success_message || "支付记录已保存");
      setDialogOpen(false);
    },
    onError: () => {
      toast.error("保存失败");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["payments-overview"] });
      toast.success(action.success_message);
    },
    onError: () => {
      toast.error("删除失败");
    },
  });

  const handleSubmit = (data: any) => {
    const action = selectedPayment
      ? selectedPayment.update_action
      : paymentOverview.create_action;
    if (!action) {
      toast.error("当前无法保存支付记录");
      return;
    }
    saveMutation.mutate({ action, data });
  };

  const handleEdit = (payment: Payment) => {
    setSelectedPayment(payment);
    setDialogOpen(true);
  };

  const handleDelete = (action: DeleteAction) => {
    setPaymentToDelete(action);
    setDeleteDialogOpen(true);
  };

  const confirmDelete = () => {
    const action = paymentToDelete;
    if (action) {
      deleteMutation.mutate(action);
      setDeleteDialogOpen(false);
      setPaymentToDelete(null);
    }
  };

  return (
    <QueryStateGate
      isLoading={isLoading}
      isError={isError}
      refetch={refetch}
      loadingFallback={
        <div className="flex h-full items-center justify-center">加载中...</div>
      }
    >
    <div className="container mx-auto space-y-6 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">支付管理</h1>
          <p className="mt-1 text-muted-foreground">
            管理支付记录、发票绑定和追票提醒
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" asChild>
            <Link to="/reconciliation">
              <Scale className="mr-2 h-4 w-4" />
              去对账中心
            </Link>
          </Button>
          <Button variant="outline" onClick={() => setReminderDialogOpen(true)}>
            <AlertCircle className="mr-2 h-4 w-4" />
            追票提醒
          </Button>
          <Button
            disabled={!paymentOverview.create_action}
            onClick={() => {
              setSelectedPayment(undefined);
              setDialogOpen(true);
            }}
          >
            <Plus className="mr-2 h-4 w-4" />
            新增支付
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card className="p-4">
          <div className="text-sm text-muted-foreground">总记录数</div>
          <div className="mt-1 text-2xl font-bold">{stats.total}</div>
        </Card>
        <Card className="p-4">
          <div className="text-sm text-muted-foreground">总支出</div>
          <div className="mt-1 text-2xl font-bold text-red-600">
            {stats.expense_label}
          </div>
        </Card>
        <Card className="p-4">
          <div className="text-sm text-muted-foreground">总收入</div>
          <div className="mt-1 text-2xl font-bold text-green-600">
            {stats.income_label}
          </div>
        </Card>
        <Card className="p-4">
          <div className="text-sm text-muted-foreground">待收发票</div>
          <div className="mt-1 text-2xl font-bold text-orange-600">
            {stats.pending_invoice}
          </div>
        </Card>
      </div>

      <AccountsReceivableAgingPanel />

      <Card className="p-6">
        <div className="mb-4 flex flex-col gap-4 md:flex-row">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground" />
            <Input
              placeholder="搜索支付编号、付款方、收款方..."
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              className="pl-10"
            />
          </div>
          <Select value={typeFilter} onValueChange={setTypeFilter}>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="类型" />
            </SelectTrigger>
            <SelectContent>
              {paymentOverview.payment_type_options.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={methodFilter} onValueChange={setMethodFilter}>
            <SelectTrigger className="w-[160px]">
              <SelectValue placeholder="支付方式" />
            </SelectTrigger>
            <SelectContent>
              {paymentOverview.payment_method_options.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={invoiceFilter} onValueChange={setInvoiceFilter}>
            <SelectTrigger className="w-[140px]">
              <SelectValue placeholder="发票状态" />
            </SelectTrigger>
            <SelectContent>
              {paymentOverview.invoice_status_options.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="rounded-lg border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>支付编号</TableHead>
                <TableHead>日期</TableHead>
                <TableHead>类型</TableHead>
                <TableHead>付款方</TableHead>
                <TableHead>收款方</TableHead>
                <TableHead>支付方式</TableHead>
                <TableHead className="text-right">金额</TableHead>
                <TableHead>发票状态</TableHead>
                <TableHead>状态</TableHead>
                <TableHead className="text-right">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {payments.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={10} className="py-12">
                    <div className="flex flex-col items-center justify-center text-center">
                      <FileText className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">
                        {paymentOverview.empty_message}
                      </h3>
                      <p className="text-muted-foreground mb-6 max-w-md">
                        登记客户回款，系统据此跟踪应收账款和现金流。
                      </p>
                      {paymentOverview.create_action && (
                        <Button
                          size="lg"
                          onClick={() => {
                            setSelectedPayment(undefined);
                            setDialogOpen(true);
                          }}
                        >
                          <Plus className="mr-2 h-4 w-4" /> 新增支付
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ) : (
                payments.map((payment) => (
                  <TableRow key={payment.id}>
                    <TableCell className="font-medium">
                      {payment.payment_no}
                    </TableCell>
                    <TableCell>{payment.payment_date_label}</TableCell>
                    <TableCell>
                      <PaymentBadgeView badge={payment.payment_type_badge} />
                    </TableCell>
                    <TableCell>{payment.payer || "-"}</TableCell>
                    <TableCell>{payment.payee || "-"}</TableCell>
                    <TableCell>{payment.payment_method_label}</TableCell>
                    <TableCell className="text-right font-medium">
                      <span className={payment.amount_class}>
                        {payment.signed_amount_label}
                      </span>
                    </TableCell>
                    <TableCell>
                      {payment.invoice_badge.visible === false ? (
                        "-"
                      ) : (
                        <PaymentBadgeView badge={payment.invoice_badge} />
                      )}
                    </TableCell>
                    <TableCell>
                      <PaymentBadgeView badge={payment.status_badge} />
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-2">
                        {payment.update_action && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleEdit(payment)}
                          >
                            <Edit className="h-4 w-4" />
                          </Button>
                        )}
                        {payment.can_delete && payment.delete_action && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDelete(payment.delete_action)}
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        )}
                      </div>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      </Card>

      <PaymentDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        payment={selectedPayment}
        onSubmit={handleSubmit}
      />

      <InvoiceReminderDialog
        open={reminderDialogOpen}
        onOpenChange={setReminderDialogOpen}
      />

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {paymentToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={confirmDelete}>
              确认删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
    </QueryStateGate>
  );
}
