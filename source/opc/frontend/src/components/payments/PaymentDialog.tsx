import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Calendar } from "@/components/ui/calendar";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { CalendarIcon } from "lucide-react";
import { format } from "date-fns";
import { cn } from "@/lib/utils";
import { Switch } from "@/components/ui/switch";
import { api } from "@/integrations/api/client";
import { AttachmentManager } from "@/components/common/AttachmentManager";

const paymentSchema = z.object({
  payment_no: z.string().min(1, "请输入支付编号"),
  payment_date: z.date().optional(),
  amount: z.string().min(1, "请输入金额"),
  payment_method: z.string().min(1, "请选择支付方式"),
  payment_type: z.string().min(1, "请选择类型"),
  payer: z.string().min(1, "请输入付款方"),
  payee: z.string().min(1, "请输入收款方"),
  category: z.string().optional(),
  invoice_required: z.boolean(),
  invoice_deadline: z.date().optional().nullable(),
  status: z.string(),
  notes: z.string().optional(),
});

type PaymentFormValues = z.infer<typeof paymentSchema>;

interface PaymentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  payment?: any;
  onSubmit: (data: any) => void;
}

interface PaymentFormOption {
  value: string;
  label: string;
}

interface PaymentFormOptions {
  payment_type_options: PaymentFormOption[];
  payment_method_options: PaymentFormOption[];
  category_options: PaymentFormOption[];
  status_options: PaymentFormOption[];
  default_values: {
    payment_type: string;
    payment_method: string;
    category: string;
    status: string;
    invoice_required: boolean;
  };
}

export function PaymentDialog({ open, onOpenChange, payment, onSubmit }: PaymentDialogProps) {
  const form = useForm<PaymentFormValues>({
    resolver: zodResolver(paymentSchema),
    defaultValues: {
      payment_no: "",
      payment_date: undefined,
      amount: "",
      payment_method: "",
      payment_type: "",
      payer: "",
      payee: "",
      category: "",
      invoice_required: false,
      invoice_deadline: null,
      status: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["payment-form-options"],
    queryFn: () => api.get<PaymentFormOptions>("/api/payments/form_options"),
    enabled: open,
  });
  const paymentTypeOptions = formOptions?.payment_type_options ?? [];
  const paymentMethodOptions = formOptions?.payment_method_options ?? [];
  const categoryOptions = formOptions?.category_options ?? [];
  const statusOptions = formOptions?.status_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    if (payment) {
      form.reset({
        payment_no: payment.payment_no,
        payment_date: payment.payment_date ? new Date(payment.payment_date) : undefined,
        amount: payment.amount?.toString() || "",
        payment_method: payment.payment_method || defaults.payment_method,
        payment_type: payment.payment_type || defaults.payment_type,
        payer: payment.payer || "",
        payee: payment.payee || "",
        category: payment.category || "",
        invoice_required: payment.invoice_required ?? defaults.invoice_required,
        invoice_deadline: payment.invoice_deadline ? new Date(payment.invoice_deadline) : null,
        status: payment.status || defaults.status,
        notes: payment.notes || "",
      });
    } else {
      form.reset({
        payment_no: "",
        payment_date: undefined,
        amount: "",
        payment_method: defaults.payment_method,
        payment_type: defaults.payment_type,
        payer: "",
        payee: "",
        category: defaults.category,
        invoice_required: defaults.invoice_required,
        invoice_deadline: null,
        status: defaults.status,
        notes: "",
      });
    }
  }, [open, payment, formOptions?.default_values, form]);

  const handleSubmit = (data: PaymentFormValues) => {
    onSubmit(data);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{payment ? "编辑支付记录" : "新增支付记录"}</DialogTitle>
        </DialogHeader>
        <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>支付编号 *</Label>
              <Input {...form.register("payment_no")} placeholder="PAY202501001" />
              {form.formState.errors.payment_no && (
                <p className="text-sm text-destructive">{form.formState.errors.payment_no.message}</p>
              )}
            </div>
            <div>
              <Label>支付日期</Label>
              <Popover>
                <PopoverTrigger asChild>
                  <Button
                    variant="outline"
                    className={cn(
                      "w-full justify-start text-left font-normal",
                      !form.watch("payment_date") && "text-muted-foreground"
                    )}
                  >
                    <CalendarIcon className="mr-2 h-4 w-4" />
                    {form.watch("payment_date") ? format(form.watch("payment_date")!, "PPP") : "后端自动登记"}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-auto p-0">
                  <Calendar
                    mode="single"
                    selected={form.watch("payment_date")}
                    onSelect={(date) => form.setValue("payment_date", date)}
                  />
                </PopoverContent>
              </Popover>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>类型 *</Label>
              <Select value={form.watch("payment_type")} onValueChange={(value) => form.setValue("payment_type", value)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {paymentTypeOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>金额 *</Label>
              <Input {...form.register("amount")} inputMode="decimal" placeholder="0.00" />
              {form.formState.errors.amount && (
                <p className="text-sm text-destructive">{form.formState.errors.amount.message}</p>
              )}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>支付方式 *</Label>
              <Select value={form.watch("payment_method")} onValueChange={(value) => form.setValue("payment_method", value)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {paymentMethodOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>状态</Label>
              <Select value={form.watch("status")} onValueChange={(value) => form.setValue("status", value)}>
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

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>付款方 *</Label>
              <Input {...form.register("payer")} placeholder="付款方名称" />
            </div>
            <div>
              <Label>收款方 *</Label>
              <Input {...form.register("payee")} placeholder="收款方名称" />
            </div>
          </div>

          <div>
            <Label>费用类别</Label>
            <Select value={form.watch("category") || ""} onValueChange={(value) => form.setValue("category", value)}>
              <SelectTrigger>
                <SelectValue placeholder="选择费用类别" />
              </SelectTrigger>
              <SelectContent>
                {categoryOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-4 border-t pt-4">
            <div className="flex items-center justify-between">
              <Label>需要发票</Label>
              <Switch
                checked={form.watch("invoice_required")}
                onCheckedChange={(checked) => {
                  form.setValue("invoice_required", checked);
                }}
              />
            </div>

            {form.watch("invoice_required") && (
              <div>
                <Label>发票截止日期</Label>
                <Popover>
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      className={cn(
                        "w-full justify-start text-left font-normal",
                        !form.watch("invoice_deadline") && "text-muted-foreground"
                      )}
                    >
                      <CalendarIcon className="mr-2 h-4 w-4" />
                      {form.watch("invoice_deadline") ? format(form.watch("invoice_deadline")!, "PPP") : "选择截止日期"}
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-0">
                    <Calendar
                      mode="single"
                      selected={form.watch("invoice_deadline") || undefined}
                      onSelect={(date) => form.setValue("invoice_deadline", date || null)}
                    />
                  </PopoverContent>
                </Popover>
              </div>
            )}
          </div>

          <div>
            <Label>备注</Label>
            <Textarea {...form.register("notes")} placeholder="输入备注信息" rows={3} />
          </div>

          {payment?.id && (
            <div>
              <Label>附件</Label>
              <AttachmentManager entityId={payment.id} entityType="payment" />
            </div>
          )}

          <div className="flex justify-end gap-2 pt-4">
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit">保存</Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
