import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { useEffect } from "react";
import { api, domainSubscriptions } from "@/integrations/api/client";
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

const domainSchema = z.object({
  domain_name: z.string().min(1, "请输入域名"),
  registrar: z.string().min(1, "请输入注册商"),
  registration_date: z.string().min(1, "请选择注册日期"),
  expiry_date: z.string().min(1, "请选择到期日期"),
  initial_purchase_cost: z.string().min(1, "请输入首次购买成本"),
  annual_fee: z.string().min(1, "请输入年度续费成本"),
  auto_renewal: z.boolean(),
  dns_provider: z.string().optional(),
  contact_email: z.string().email("请输入有效的邮箱").optional().or(z.literal("")),
  notes: z.string().optional(),
});

type DomainFormValues = z.infer<typeof domainSchema>;

const CLOUD_COST_FORM_OPTIONS_ENDPOINT = "/api/v1/cloud_cost/form_options";

interface CloudCostFormOption {
  value: string;
  label: string;
}

interface CloudCostFormOptions {
  default_values: Record<string, unknown>;
}

interface DomainSubscriptionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
  initialData?: any;
}

const DomainSubscriptionDialog = ({
  open,
  onOpenChange,
  onSuccess,
  initialData,
}: DomainSubscriptionDialogProps) => {
  const form = useForm<DomainFormValues>({
    resolver: zodResolver(domainSchema),
    defaultValues: {
      domain_name: "",
      registrar: "",
      registration_date: "",
      expiry_date: "",
      initial_purchase_cost: "",
      annual_fee: "",
      auto_renewal: false,
      dns_provider: "",
      contact_email: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["cloud-cost-form-options"],
    queryFn: () =>
      api.get<CloudCostFormOptions>(CLOUD_COST_FORM_OPTIONS_ENDPOINT),
    enabled: open,
  });

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      domain_name: initialData.domain_name || "",
      registrar: initialData.registrar || "",
      registration_date: initialData.registration_date || "",
      expiry_date: initialData.expiry_date || "",
      initial_purchase_cost: initialData.initial_purchase_cost?.toString() || "",
      annual_fee: initialData.annual_fee?.toString() || "",
      auto_renewal: initialData.auto_renewal ?? false,
      dns_provider: initialData.dns_provider || "",
      contact_email: initialData.contact_email || "",
      notes: initialData.notes || "",
    } : {
      domain_name: "",
      registrar: "",
      registration_date: "",
      expiry_date: "",
      initial_purchase_cost: "",
      annual_fee: "",
      auto_renewal: false,
      dns_provider: "",
      contact_email: "",
      notes: "",
    });
  }, [open, initialData, formOptions?.default_values, form]);

  const onSubmit = async (data: DomainFormValues) => {
    try {
      const payload = {
        domain_name: data.domain_name,
        registrar: data.registrar,
        registration_date: data.registration_date,
        expiry_date: data.expiry_date,
        initial_purchase_cost: data.initial_purchase_cost,
        annual_fee: data.annual_fee,
        auto_renewal: data.auto_renewal,
        dns_provider: data.dns_provider || null,
        contact_email: data.contact_email || null,
        notes: data.notes || null,
      };

      if (initialData?.id) {
        await domainSubscriptions.update(initialData.id, payload);
        toast.success("域名订阅信息已更新");
      } else {
        await domainSubscriptions.create(payload);
        toast.success("域名订阅已添加");
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
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? "编辑域名" : "添加域名"}</DialogTitle>
          <DialogDescription>
            管理域名订阅信息
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="domain_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>域名 *</FormLabel>
                    <FormControl>
                      <Input placeholder="example.com" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="registrar"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>注册商 *</FormLabel>
                    <FormControl>
                      <Input placeholder="如：阿里云" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="registration_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>注册日期 *</FormLabel>
                    <FormControl>
                      <Input type="date" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="expiry_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>到期日期 *</FormLabel>
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
                name="initial_purchase_cost"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>首次购买成本（元）*</FormLabel>
                    <FormControl>
                      <Input inputMode="decimal" placeholder="0.00" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="annual_fee"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>年度续费成本（元）*</FormLabel>
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
              name="dns_provider"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>DNS服务商</FormLabel>
                  <FormControl>
                    <Input placeholder="如：Cloudflare" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

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
              name="contact_email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>联系邮箱</FormLabel>
                  <FormControl>
                    <Input type="email" placeholder="email@example.com" {...field} />
                  </FormControl>
                  <FormMessage />
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

export default DomainSubscriptionDialog;
