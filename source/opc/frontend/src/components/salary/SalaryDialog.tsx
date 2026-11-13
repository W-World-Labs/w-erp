import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Separator } from "@/components/ui/separator";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { AttachmentManager } from "@/components/common/AttachmentManager";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "@/components/ui/use-toast";

const salarySchema = z.object({
  employee_name: z.string().min(1, "员工姓名不能为空"),
  department: z.string().optional(),
  position: z.string().optional(),
  period: z.string().optional(),
  base_salary: z.string().min(1, "基本工资不能为空"),
  bonus: z.string().optional(),
  deduction: z.string().optional(),
  social_security: z.string().optional(),
  housing_fund: z.string().optional(),
  social_security_company: z.string().optional(),
  housing_fund_company: z.string().optional(),
  notes: z.string().optional(),
});

type SalaryFormValues = z.infer<typeof salarySchema>;

interface SalaryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: any) => void;
  initialData?: any;
  insurancePreviewAction?: BackendAction | null;
}

interface SalaryInsurancePreview {
  social_security_input: string;
  housing_fund_input: string;
  social_security_company_input: string;
  housing_fund_company_input: string;
}

export function SalaryDialog({
  open,
  onOpenChange,
  onSubmit,
  initialData,
  insurancePreviewAction,
  employees = [],
}: SalaryDialogProps & { employees?: any[] }) {
  const form = useForm<SalaryFormValues>({
    resolver: zodResolver(salarySchema),
    defaultValues: initialData
      ? {
          employee_name: initialData.employee_name,
          department: initialData.department || "",
          position: initialData.position || "",
          period: initialData.period,
          base_salary: initialData.base_salary?.toString() || "",
          bonus: initialData.bonus?.toString() || "",
          deduction: initialData.deduction?.toString() || "",
          social_security: initialData.social_security?.toString() || "",
          housing_fund: initialData.housing_fund?.toString() || "",
          social_security_company:
            initialData.social_security_company?.toString() || "",
          housing_fund_company:
            initialData.housing_fund_company?.toString() || "",
          notes: initialData.notes || "",
        }
      : {
          employee_name: "",
          department: "",
          position: "",
          period: "",
          base_salary: "",
          bonus: "0",
          deduction: "0",
          social_security: "0",
          housing_fund: "0",
          social_security_company: "0",
          housing_fund_company: "0",
          notes: "",
        },
  });

  const calculateInsurance = async () => {
    try {
      if (!insurancePreviewAction) {
        throw new Error("后端未返回社保公积金计算动作");
      }
      const preview = await api.runAction<SalaryInsurancePreview>(
        insurancePreviewAction,
        {
          employee_id: form.getValues("employee_name") || undefined,
          base_salary: form.getValues("base_salary") || "0",
        },
      );

      form.setValue("social_security", preview.social_security_input);
      form.setValue("housing_fund", preview.housing_fund_input);
      form.setValue(
        "social_security_company",
        preview.social_security_company_input,
      );
      form.setValue(
        "housing_fund_company",
        preview.housing_fund_company_input,
      );
    } catch (error) {
      toast({
        title: "自动计算失败",
        description:
          error instanceof Error ? error.message : "请先配置社保公积金规则",
        variant: "destructive",
      });
    }
  };

  const handleSubmit = (values: SalaryFormValues) => {
    onSubmit({
      ...values,
      employee_id: values.employee_name,
    });

    form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? "编辑工资记录" : "录入工资"}</DialogTitle>
        </DialogHeader>

        <Form {...form}>
          <form
            onSubmit={form.handleSubmit(handleSubmit)}
            className="space-y-4"
          >
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="employee_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>员工姓名 *</FormLabel>
                    <FormControl>
                      {initialData ? (
                        <Input {...field} disabled />
                      ) : (
                        <Select
                          onValueChange={(val) => {
                            field.onChange(val);
                            const emp = employees?.find((e) => e.id === val);
                            if (emp) {
                              if (emp.base_salary)
                                form.setValue(
                                  "base_salary",
                                  emp.base_salary.toString(),
                                );
                              if (emp.department)
                                form.setValue("department", emp.department);
                              if (emp.position)
                                form.setValue("position", emp.position);
                            }
                          }}
                          defaultValue={field.value}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="选择员工" />
                          </SelectTrigger>
                          <SelectContent>
                            {employees?.map((emp: any) => (
                              <SelectItem key={emp.id} value={emp.id}>
                                {emp.name}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      )}
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="period"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>工资期</FormLabel>
                    <FormControl>
                      <Input {...field} type="month" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="department"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>部门</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入部门" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="position"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>职位</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入职位" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="base_salary"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>基本工资 *</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        type="number"
                        step="0.01"
                        placeholder="输入基本工资"
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="bonus"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>奖金</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        type="number"
                        step="0.01"
                        placeholder="输入奖金"
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="deduction"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>其他扣款</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        type="number"
                        step="0.01"
                        placeholder="输入其他扣款"
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <Separator />
            <div className="flex items-center justify-between">
              <h3 className="font-semibold text-sm">社保公积金</h3>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={calculateInsurance}
              >
                自动计算
              </Button>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="social_security"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>社保（个人）</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        type="number"
                        step="0.01"
                        placeholder="输入社保个人缴纳"
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="social_security_company"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>社保（公司）</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        type="number"
                        step="0.01"
                        placeholder="输入社保公司缴纳"
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="housing_fund"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>公积金（个人）</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        type="number"
                        step="0.01"
                        placeholder="输入公积金个人缴纳"
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="housing_fund_company"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>公积金（公司）</FormLabel>
                    <FormControl>
                      <Input
                        {...field}
                        type="number"
                        step="0.01"
                        placeholder="输入公积金公司缴纳"
                      />
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
                entityType="salary"
              />
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
              >
                取消
              </Button>
              <Button type="submit">{initialData ? "更新" : "确认录入"}</Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
