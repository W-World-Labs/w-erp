import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
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

const budgetSchema = z.object({
  budget_type: z.string().min(1, "请选择预算类型"),
  period: z.string().min(1, "请选择预算期"),
  category: z.string().min(1, "请选择预算类别"),
  budget_amount: z.string().min(1, "预算金额不能为空"),
  notes: z.string().optional(),
});

type BudgetFormValues = z.infer<typeof budgetSchema>;

interface BudgetFormOption {
  value: string;
  label: string;
}

interface BudgetFormOptions {
  budget_type_options: BudgetFormOption[];
  category_options: BudgetFormOption[];
  default_values: {
    budget_type: string;
    category: string;
  };
}

interface BudgetDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: any) => void;
  initialData?: any;
  budgetType: 'monthly' | 'quarterly';
  defaultPeriod: string;
}

export function BudgetDialog({ open, onOpenChange, onSubmit, initialData, budgetType, defaultPeriod }: BudgetDialogProps) {
  const form = useForm<BudgetFormValues>({
    resolver: zodResolver(budgetSchema),
    defaultValues: {
      budget_type: "",
      period: "",
      category: "",
      budget_amount: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["budget-form-options"],
    queryFn: () => api.get<BudgetFormOptions>("/api/budgets/form_options"),
    enabled: open,
  });
  const budgetTypeOptions = formOptions?.budget_type_options ?? [];
  const categoryOptions = formOptions?.category_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      budget_type: initialData.budget_type || defaults.budget_type,
      period: initialData.period || defaultPeriod,
      category: initialData.category || defaults.category,
      budget_amount: initialData.budget_amount?.toString() || "",
      notes: initialData.notes || "",
    } : {
      budget_type: budgetType || defaults.budget_type,
      period: defaultPeriod,
      category: defaults.category,
      budget_amount: "",
      notes: "",
    });
  }, [open, initialData, budgetType, defaultPeriod, formOptions?.default_values, form]);

  const handleSubmit = (values: BudgetFormValues) => {
    onSubmit(values);
    form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>{initialData ? '编辑预算' : '新建预算'}</DialogTitle>
        </DialogHeader>
        
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="budget_type"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>预算类型 *</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value} disabled={!!initialData}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="选择预算类型" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {budgetTypeOptions.map((option) => (
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
                  <FormLabel>预算期 *</FormLabel>
                  <FormControl>
                    <Input 
                      {...field} 
                      type={budgetType === 'monthly' ? 'month' : 'text'}
                      placeholder={budgetType === 'monthly' ? '2024-01' : '2024-Q1'}
                      disabled={!!initialData}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="category"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>预算类别 *</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value} disabled={!!initialData}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="选择类别" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {categoryOptions.map((option) => (
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
              name="budget_amount"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>预算金额 *</FormLabel>
                  <FormControl>
                    <Input {...field} type="number" step="0.01" placeholder="输入预算金额" />
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
                entityType="budget"
              />
            </div>

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit">
                {initialData ? '更新' : '创建'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
