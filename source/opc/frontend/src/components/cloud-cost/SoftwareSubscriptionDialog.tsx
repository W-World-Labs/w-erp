import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { useEffect } from "react";
import { api, softwareSubscriptions } from "@/integrations/api/client";
import { toast } from "sonner";
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
import { Checkbox } from "@/components/ui/checkbox";

const softwareSchema = z.object({
  software_name: z.string().min(1, "请输入软件名称"),
  vendor: z.string().min(1, "请输入供应商"),
  subscription_type: z.string().min(1, "请选择订阅类型"),
  plan_name: z.string().optional(),
  license_count: z.string().min(1, "请输入许可数量"),
  total_price: z.string().min(1, "请输入总价格"),
  billing_cycle: z.string().min(1, "请选择计费周期"),
  start_date: z.string().optional(),
  next_billing_date: z.string().min(1, "请选择下次计费日期"),
  auto_renewal: z.boolean(),
  usage_department: z.string().optional(),
  account_email: z.string().email("请输入有效的邮箱").optional().or(z.literal("")),
  notes: z.string().optional(),
});

type SoftwareFormValues = z.infer<typeof softwareSchema>;

const CLOUD_COST_FORM_OPTIONS_ENDPOINT = "/api/v1/cloud_cost/form_options";

interface CloudCostFormOption {
  value: string;
  label: string;
}

interface CloudCostFormOptions {
  software_subscription_type_options: CloudCostFormOption[];
  billing_cycle_options: CloudCostFormOption[];
  default_values: {
    software_subscription_type: string;
    billing_cycle: string;
    license_count: number;
    auto_renewal: boolean;
  };
}

interface SoftwareSubscriptionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  initialData?: any;
}

const SoftwareSubscriptionDialog = ({
  open,
  onOpenChange,
  onSuccess,
  initialData,
}: SoftwareSubscriptionDialogProps) => {
  const form = useForm<SoftwareFormValues>({
    resolver: zodResolver(softwareSchema),
    defaultValues: {
      software_name: "",
      vendor: "",
      subscription_type: "",
      plan_name: "",
      license_count: "",
      total_price: "",
      billing_cycle: "",
      start_date: "",
      next_billing_date: "",
      auto_renewal: false,
      usage_department: "",
      account_email: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["cloud-cost-form-options"],
    queryFn: () =>
      api.get<CloudCostFormOptions>(CLOUD_COST_FORM_OPTIONS_ENDPOINT),
    enabled: open,
  });
  const softwareSubscriptionTypeOptions =
    formOptions?.software_subscription_type_options ?? [];
  const billingCycleOptions = formOptions?.billing_cycle_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      software_name: initialData.software_name || "",
      vendor: initialData.vendor || "",
      subscription_type: initialData.subscription_type || defaults.software_subscription_type,
      plan_name: initialData.plan_name || "",
      license_count: initialData.license_count?.toString() || String(defaults.license_count),
      total_price: initialData.total_price?.toString() || "",
      billing_cycle: initialData.billing_cycle || defaults.billing_cycle,
      start_date: initialData.start_date || "",
      next_billing_date: initialData.next_billing_date || "",
      auto_renewal: initialData.auto_renewal ?? defaults.auto_renewal,
      usage_department: initialData.usage_department || "",
      account_email: initialData.account_email || "",
      notes: initialData.notes || "",
    } : {
      software_name: "",
      vendor: "",
      subscription_type: defaults.software_subscription_type,
      plan_name: "",
      license_count: String(defaults.license_count),
      total_price: "",
      billing_cycle: defaults.billing_cycle,
      start_date: "",
      next_billing_date: "",
      auto_renewal: defaults.auto_renewal,
      usage_department: "",
      account_email: "",
      notes: "",
    });
  }, [open, initialData, formOptions?.default_values, form]);

  const onSubmit = async (data: SoftwareFormValues) => {
    try {
      const payload = {
        software_name: data.software_name,
        vendor: data.vendor,
        subscription_type: data.subscription_type,
        plan_name: data.plan_name || null,
        license_count: data.license_count,
        total_price: data.total_price,
        billing_cycle: data.billing_cycle,
        start_date: data.start_date || undefined,
        next_billing_date: data.next_billing_date,
        auto_renewal: data.auto_renewal,
        usage_department: data.usage_department || null,
        account_email: data.account_email || null,
        notes: data.notes || null,
      };

      if (initialData?.id) {
        await softwareSubscriptions.update(initialData.id, payload);
        toast.success("软件订阅信息已更新");
      } else {
        await softwareSubscriptions.create(payload);
        toast.success("软件订阅已添加");
      }

      form.reset();
      onOpenChange(false);
      onSuccess?.();
    } catch (error: any) {
      toast.error(error.message || "操作失败");
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? "编辑软件订阅" : "添加软件订阅"}</DialogTitle>
          <DialogDescription>
            管理SaaS工具和软件许可订阅
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="software_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>软件名称 *</FormLabel>
                    <FormControl>
                      <Input placeholder="如：GitHub Enterprise" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="vendor"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>供应商 *</FormLabel>
                    <FormControl>
                      <Input placeholder="如：GitHub" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="subscription_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>订阅类型 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {softwareSubscriptionTypeOptions.map((option) => (
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
                name="plan_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>套餐名称</FormLabel>
                    <FormControl>
                      <Input placeholder="如：Team Plan" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-3 gap-4">
              <FormField
                control={form.control}
                name="license_count"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>许可数量 *</FormLabel>
                    <FormControl>
                      <Input inputMode="numeric" placeholder="1" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="total_price"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>总价格（元）*</FormLabel>
                    <FormControl>
                      <Input inputMode="decimal" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="billing_cycle"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>计费周期 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {billingCycleOptions.map((option) => (
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
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="start_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>开始日期</FormLabel>
                    <FormControl>
                      <Input type="date" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="next_billing_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>下次计费日期 *</FormLabel>
                    <FormControl>
                      <Input type="date" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="usage_department"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>使用部门</FormLabel>
                    <FormControl>
                      <Input placeholder="如：研发部" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="account_email"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>账户邮箱</FormLabel>
                    <FormControl>
                      <Input type="email" placeholder="email@example.com" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <FormField
              control={form.control}
              name="auto_renewal"
              render={({ field }) => (
                <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
                  <FormControl>
                    <Checkbox
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  </FormControl>
                  <div className="space-y-1 leading-none">
                    <FormLabel>
                      自动续费
                    </FormLabel>
                  </div>
                </FormItem>
              )}
            />

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

export default SoftwareSubscriptionDialog;
