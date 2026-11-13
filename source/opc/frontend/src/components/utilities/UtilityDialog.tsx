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

const utilitySchema = z.object({
  location_name: z.string().min(1, "场地名称不能为空"),
  utility_type: z.string().min(1, "请选择账单类型"),
  period: z.string().optional(),
  usage: z.string().min(1, "用量不能为空"),
  unit_price: z.string().min(1, "单价不能为空"),
  due_date: z.string().min(1, "请选择缴费截止日期"),
  notes: z.string().optional(),
});

type UtilityFormValues = z.infer<typeof utilitySchema>;

interface UtilityDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: any) => void;
  initialData?: any;
}

interface UtilityFormOption {
  value: string;
  label: string;
}

interface UtilityFormOptions {
  utility_type_options: UtilityFormOption[];
  default_values: {
    utility_type: string;
  };
}

export function UtilityDialog({ open, onOpenChange, onSubmit, initialData }: UtilityDialogProps) {
  const form = useForm<UtilityFormValues>({
    resolver: zodResolver(utilitySchema),
    defaultValues: {
      location_name: "",
      utility_type: "",
      period: "",
      usage: "",
      unit_price: "",
      due_date: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["rent-form-options"],
    queryFn: () => api.get<UtilityFormOptions>("/api/v1/rent/form_options"),
    enabled: open,
  });
  const utilityTypeOptions = formOptions?.utility_type_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      location_name: initialData.location_name || "",
      utility_type: initialData.utility_type || defaults.utility_type,
      period: initialData.period || "",
      usage: initialData.usage?.toString() || "",
      unit_price: initialData.unit_price?.toString() || "",
      due_date: initialData.due_date || "",
      notes: initialData.notes || "",
    } : {
      location_name: "",
      utility_type: defaults.utility_type,
      period: "",
      usage: "",
      unit_price: "",
      due_date: "",
      notes: "",
    });
  }, [open, initialData, formOptions?.default_values, form]);

  const handleSubmit = (values: UtilityFormValues) => {
    onSubmit(values);
    
    form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? '编辑账单' : '录入账单'}</DialogTitle>
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
                name="utility_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>账单类型 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择账单类型" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {utilityTypeOptions.map((option) => (
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
                name="period"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>账期</FormLabel>
                    <FormControl>
                      <Input {...field} type="month" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="usage"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>用量 *</FormLabel>
                    <FormControl>
                      <Input {...field} inputMode="decimal" placeholder="输入用量" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="unit_price"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>单价（元）*</FormLabel>
                    <FormControl>
                      <Input {...field} inputMode="decimal" placeholder="输入单价" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="due_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>缴费截止日期 *</FormLabel>
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
                entityType="utility"
              />
            </div>

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit">
                {initialData ? '更新' : '确认录入'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
