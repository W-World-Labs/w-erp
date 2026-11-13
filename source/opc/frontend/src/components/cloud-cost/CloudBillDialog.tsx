import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import { useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";

const billSchema = z.object({
  provider_id: z.string().min(1, "请选择云服务商"),
  bill_month: z.string().optional(),
  item_name: z.string().optional(),
  bill_date: z.string().optional(),
  total_amount: z.string().min(1, "请输入总金额"),
  discount_amount: z.string().optional(),
  actual_amount: z.string().optional(),
  notes: z.string().optional(),
});

type BillFormValues = z.infer<typeof billSchema>;

const CLOUD_COST_FORM_OPTIONS_ENDPOINT = "/api/v1/cloud_cost/form_options";

interface CloudCostFormOption {
  value: string;
  label: string;
}

interface CloudCostFormOptions {
  provider_options: CloudCostFormOption[];
  bill_create_action: BackendAction | null;
}

interface CloudBillDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  initialData?: (any & { update_action?: BackendAction | null });
}

const CloudBillDialog = ({
  open,
  onOpenChange,
  onSuccess,
  initialData,
}: CloudBillDialogProps) => {
  const form = useForm<BillFormValues>({
    resolver: zodResolver(billSchema),
    defaultValues: {
      provider_id: "",
      bill_month: "",
      item_name: "",
      bill_date: "",
      total_amount: "",
      discount_amount: "",
      actual_amount: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["cloud-cost-form-options"],
    queryFn: () =>
      api.get<CloudCostFormOptions>(CLOUD_COST_FORM_OPTIONS_ENDPOINT),
    enabled: open,
  });
  const providerOptions = formOptions?.provider_options ?? [];

  useEffect(() => {
    if (!open) {
      return;
    }
    form.reset(initialData ? {
      provider_id: initialData.provider_id || "",
      bill_month: initialData.bill_month || initialData.billing_period || "",
      item_name: initialData.item_name || "",
      bill_date: initialData.bill_date || "",
      total_amount: initialData.total_amount?.toString() || "",
      discount_amount: initialData.discount_amount?.toString() || "",
      actual_amount: initialData.actual_amount?.toString() || "",
      notes: initialData.notes || "",
    } : {
      provider_id: "",
      bill_month: "",
      item_name: "",
      bill_date: "",
      total_amount: "",
      discount_amount: "",
      actual_amount: "",
      notes: "",
    });
  }, [open, initialData, form]);

  const onSubmit = async (data: BillFormValues) => {
    try {
      const payload = {
        provider_id: data.provider_id || null,
        billing_period: data.bill_month || undefined,
        total_amount: data.total_amount,
        notes: data.notes || null,
      };

      const action = initialData?.update_action ?? formOptions?.bill_create_action;
      if (!action) {
        toast.error("当前无法保存云账单");
        return;
      }

      await api.runAction(action, payload);
      toast.success(action.success_message || "云账单已保存");

      form.reset();
      onOpenChange(false);
      onSuccess?.();
    } catch (error: any) {
      toast.error(error.message || "操作失败");
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? "编辑账单" : "录入账单"}</DialogTitle>
          <DialogDescription>
            录入云服务账单信息
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="provider_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>云服务商 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择云服务商" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {providerOptions.map((option) => (
                          <SelectItem key={option.value} value={option.value}>
                            {option.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="bill_month"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>账期</FormLabel>
                    <FormControl>
                      <Input placeholder="如：2025-01" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="item_name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>账单项目</FormLabel>
                  <FormControl>
                    <Input placeholder="服务名称或账单项目" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="bill_date"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>账单日期</FormLabel>
                  <FormControl>
                    <Input type="date" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="grid grid-cols-3 gap-4">
              <FormField
                control={form.control}
                name="total_amount"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>总金额（元）*</FormLabel>
                    <FormControl>
                      <Input inputMode="decimal" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="discount_amount"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>优惠金额（元）</FormLabel>
                    <FormControl>
                      <Input inputMode="decimal" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="actual_amount"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>实付金额（元）</FormLabel>
                    <FormControl>
                      <Input inputMode="decimal" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="notes"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>备注</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="其他说明信息"
                      className="resize-none"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit">
                {initialData ? "保存" : "添加"}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
};

export default CloudBillDialog;
