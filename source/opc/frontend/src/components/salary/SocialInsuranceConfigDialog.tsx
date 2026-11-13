import { useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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

const configSchema = z.object({
  city: z.string().min(1, "城市不能为空"),
  social_security_base_min: z.string().min(1, "社保基数下限不能为空"),
  social_security_base_max: z.string().min(1, "社保基数上限不能为空"),
  housing_fund_base_min: z.string().min(1, "公积金基数下限不能为空"),
  housing_fund_base_max: z.string().min(1, "公积金基数上限不能为空"),
  pension_personal_rate: z.string().min(1, "养老保险个人比例不能为空"),
  medical_personal_rate: z.string().min(1, "医疗保险个人比例不能为空"),
  unemployment_personal_rate: z.string().min(1, "失业保险个人比例不能为空"),
  pension_company_rate: z.string().min(1, "养老保险公司比例不能为空"),
  medical_company_rate: z.string().min(1, "医疗保险公司比例不能为空"),
  unemployment_company_rate: z.string().min(1, "失业保险公司比例不能为空"),
  work_injury_company_rate: z.string().min(1, "工伤保险公司比例不能为空"),
  maternity_company_rate: z.string().min(1, "生育保险公司比例不能为空"),
  housing_fund_personal_rate: z.string().min(1, "公积金个人比例不能为空"),
  housing_fund_company_rate: z.string().min(1, "公积金公司比例不能为空"),
  notes: z.string().optional(),
});

type ConfigFormValues = z.infer<typeof configSchema>;

interface SocialInsuranceConfigDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: any) => void;
  initialData?: any;
}

const defaultConfigValues: ConfigFormValues = {
  city: "杭州",
  social_security_base_min: "4462",
  social_security_base_max: "24060",
  housing_fund_base_min: "2490",
  housing_fund_base_max: "36675",
  pension_personal_rate: "8",
  medical_personal_rate: "2",
  unemployment_personal_rate: "0.5",
  pension_company_rate: "14",
  medical_company_rate: "9.5",
  unemployment_company_rate: "0.5",
  work_injury_company_rate: "0.2",
  maternity_company_rate: "0",
  housing_fund_personal_rate: "12",
  housing_fund_company_rate: "12",
  notes: "杭州2025社保公积金标准（预估）",
};

const backendInputFields: Record<string, string> = {
  social_security_base_min: "social_security_base_min_input",
  social_security_base_max: "social_security_base_max_input",
  housing_fund_base_min: "housing_fund_base_min_input",
  housing_fund_base_max: "housing_fund_base_max_input",
  pension_personal_rate: "pension_personal_rate_input",
  medical_personal_rate: "medical_personal_rate_input",
  unemployment_personal_rate: "unemployment_personal_rate_input",
  pension_company_rate: "pension_company_rate_input",
  medical_company_rate: "medical_company_rate_input",
  unemployment_company_rate: "unemployment_company_rate_input",
  work_injury_company_rate: "work_injury_company_rate_input",
  maternity_company_rate: "maternity_company_rate_input",
  housing_fund_personal_rate: "housing_fund_personal_rate_input",
  housing_fund_company_rate: "housing_fund_company_rate_input",
};

const backendInput = (
  initialData: any,
  field: keyof ConfigFormValues,
) => initialData[backendInputFields[field]] || "";

const configFormDefaults = (initialData?: any): ConfigFormValues => {
  if (!initialData) {
    return defaultConfigValues;
  }

  return {
    city: initialData.city || "",
    social_security_base_min: backendInput(initialData, "social_security_base_min"),
    social_security_base_max: backendInput(initialData, "social_security_base_max"),
    housing_fund_base_min: backendInput(initialData, "housing_fund_base_min"),
    housing_fund_base_max: backendInput(initialData, "housing_fund_base_max"),
    pension_personal_rate: backendInput(initialData, "pension_personal_rate"),
    medical_personal_rate: backendInput(initialData, "medical_personal_rate"),
    unemployment_personal_rate: backendInput(
      initialData,
      "unemployment_personal_rate",
    ),
    pension_company_rate: backendInput(initialData, "pension_company_rate"),
    medical_company_rate: backendInput(initialData, "medical_company_rate"),
    unemployment_company_rate: backendInput(
      initialData,
      "unemployment_company_rate",
    ),
    work_injury_company_rate: backendInput(initialData, "work_injury_company_rate"),
    maternity_company_rate: backendInput(initialData, "maternity_company_rate"),
    housing_fund_personal_rate: backendInput(
      initialData,
      "housing_fund_personal_rate",
    ),
    housing_fund_company_rate: backendInput(
      initialData,
      "housing_fund_company_rate",
    ),
    notes: initialData.notes || "",
  };
};

export function SocialInsuranceConfigDialog({
  open,
  onOpenChange,
  onSubmit,
  initialData,
}: SocialInsuranceConfigDialogProps) {
  const form = useForm<ConfigFormValues>({
    resolver: zodResolver(configSchema),
    defaultValues: configFormDefaults(initialData),
  });

  useEffect(() => {
    if (open) {
      form.reset(configFormDefaults(initialData));
    }
  }, [form, initialData, open]);

  const handleSubmit = (values: ConfigFormValues) => {
    onSubmit({
      city: values.city,
      social_security_base_min: values.social_security_base_min,
      social_security_base_max: values.social_security_base_max,
      housing_fund_base_min: values.housing_fund_base_min,
      housing_fund_base_max: values.housing_fund_base_max,
      pension_personal_rate: values.pension_personal_rate,
      medical_personal_rate: values.medical_personal_rate,
      unemployment_personal_rate: values.unemployment_personal_rate,
      pension_company_rate: values.pension_company_rate,
      medical_company_rate: values.medical_company_rate,
      unemployment_company_rate: values.unemployment_company_rate,
      work_injury_company_rate: values.work_injury_company_rate,
      maternity_company_rate: values.maternity_company_rate,
      housing_fund_personal_rate: values.housing_fund_personal_rate,
      housing_fund_company_rate: values.housing_fund_company_rate,
      notes: values.notes,
    });
    form.reset();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>社保公积金配置</DialogTitle>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="city"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>城市 *</FormLabel>
                  <FormControl>
                    <Input {...field} placeholder="输入城市名称" />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Separator />
            <h3 className="font-semibold text-sm">缴费基数设置</h3>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="social_security_base_min"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>社保基数下限 *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="输入社保基数下限" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="social_security_base_max"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>社保基数上限 *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="输入社保基数上限" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="housing_fund_base_min"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>公积金基数下限 *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="输入公积金基数下限" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="housing_fund_base_max"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>公积金基数上限 *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="输入公积金基数上限" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <Separator />
            <h3 className="font-semibold text-sm">社保比例（个人）</h3>

            <div className="grid grid-cols-3 gap-4">
              <FormField
                control={form.control}
                name="pension_personal_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>养老保险 (%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="8" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="medical_personal_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>医疗保险 (%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="2" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="unemployment_personal_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>失业保险 (%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="0.2" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <Separator />
            <h3 className="font-semibold text-sm">社保比例（公司）</h3>

            <div className="grid grid-cols-3 gap-4">
              <FormField
                control={form.control}
                name="pension_company_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>养老保险 (%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="16" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="medical_company_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>医疗保险 (%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="9.8" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="unemployment_company_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>失业保险 (%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="0.8" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="work_injury_company_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>工伤保险 (%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="0.2" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="maternity_company_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>生育保险 (%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="0.8" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>

            <Separator />
            <h3 className="font-semibold text-sm">公积金比例</h3>

            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="housing_fund_personal_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>公积金（个人）(%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="12" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="housing_fund_company_rate"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>公积金（公司）(%) *</FormLabel>
                    <FormControl>
                      <Input {...field} type="number" step="0.01" placeholder="12" />
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

            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                取消
              </Button>
              <Button type="submit">
                保存配置
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
