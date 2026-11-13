import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Separator } from "@/components/ui/separator";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { useQuery } from "@tanstack/react-query";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { AttachmentManager } from "@/components/common/AttachmentManager";
import { api } from "@/integrations/api/client";

const rentSchema = z.object({
  location_name: z.string().min(1, "场地名称不能为空"),
  address: z.string().min(1, "地址不能为空"),
  area: z.string().min(1, "面积不能为空"),
  monthly_rent: z.string().min(1, "月租金不能为空"),
  contract_start_date: z.string().optional(),
  contract_end_date: z.string().min(1, "请选择合同结束日期"),
  payment_cycle: z.string().min(1, "请选择缴费周期"),
  next_payment_date: z.string().min(1, "请选择下次缴费日期"),
  notes: z.string().optional(),
});

type RentFormValues = z.infer<typeof rentSchema>;

interface RentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: any) => void;
  initialData?: any;
}

interface RentFormOption {
  value: string;
  label: string;
}

interface RentFormOptions {
  payment_cycle_options: RentFormOption[];
  default_values: {
    payment_cycle: string;
  };
}

export function RentDialog({ open, onOpenChange, onSubmit, initialData }: RentDialogProps) {
  const form = useForm<RentFormValues>({
    resolver: zodResolver(rentSchema),
    defaultValues: {
      location_name: "",
      address: "",
      area: "",
      monthly_rent: "",
      contract_start_date: "",
      contract_end_date: "",
      payment_cycle: "",
      next_payment_date: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["rent-form-options"],
    queryFn: () => api.get<RentFormOptions>("/api/v1/rent/form_options"),
    enabled: open,
  });
  const paymentCycleOptions = formOptions?.payment_cycle_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      location_name: initialData.location_name || "",
      address: initialData.address || "",
      area: initialData.area?.toString() || "",
      monthly_rent: initialData.monthly_rent?.toString() || "",
      contract_start_date: initialData.contract_start_date || "",
      contract_end_date: initialData.contract_end_date || "",
      payment_cycle: initialData.payment_cycle || defaults.payment_cycle,
      next_payment_date: initialData.next_payment_date || "",
      notes: initialData.notes || "",
    } : {
      location_name: "",
      address: "",
      area: "",
      monthly_rent: "",
      contract_start_date: "",
      contract_end_date: "",
      payment_cycle: defaults.payment_cycle,
      next_payment_date: "",
      notes: "",
    });
  }, [open, initialData, formOptions?.default_values, form]);

  const handleSubmit = (values: RentFormValues) => {
    onSubmit(values);
    
    form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? '编辑场地信息' : '添加场地'}</DialogTitle>
        </DialogHeader>
        
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="location_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>场地名称 *</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入场地名称" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="address"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>地址 *</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入详细地址" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="area"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>面积（㎡）*</FormLabel>
                    <FormControl>
                      <Input {...field} inputMode="decimal" placeholder="输入面积" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="monthly_rent"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>月租金（元）*</FormLabel>
                    <FormControl>
                      <Input {...field} inputMode="decimal" placeholder="输入月租金" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="contract_start_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>合同开始日期</FormLabel>
                    <FormControl>
                      <Input {...field} type="date" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="contract_end_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>合同结束日期 *</FormLabel>
                    <FormControl>
                      <Input {...field} type="date" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="payment_cycle"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>缴费周期 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择缴费周期" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {paymentCycleOptions.map((option) => (
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
                name="next_payment_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>下次缴费日期 *</FormLabel>
                    <FormControl>
                      <Input {...field} type="date" />
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
                    <Textarea {...field} placeholder="输入备注信息" rows={3} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Separator />

            <div className="space-y-2">
              <h3 className="font-semibold text-sm">附件</h3>
              <AttachmentManager 
                entityId={initialData?.id} 
                entityType="rent"
              />
            </div>

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit">
                {initialData ? '更新' : '确认添加'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
