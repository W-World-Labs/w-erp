import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import {
  Dialog,
  DialogContent,
  DialogDescription,
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
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";

const milestoneSchema = z.object({
  milestone_date: z.string().min(1, "请选择日期"),
  title: z.string().min(1, "请输入标题").max(100, "标题不能超过100个字符"),
  description: z.string().optional(),
  milestone_type: z.string().min(1, "请选择类型"),
  importance: z.string().min(1, "请选择重要程度"),
  notes: z.string().optional(),
});

type MilestoneFormData = z.infer<typeof milestoneSchema>;

interface MilestoneFormOption {
  value: string;
  label: string;
}

interface MilestoneFormOptions {
  milestone_type_options: MilestoneFormOption[];
  importance_options: MilestoneFormOption[];
  default_values: {
    milestone_type: string;
    importance: string;
  };
}

interface MilestoneDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  milestone?: any;
  action?: BackendAction | null;
}

export const MilestoneDialog = ({
  open,
  onOpenChange,
  milestone,
  action: createAction,
}: MilestoneDialogProps) => {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const queryClient = useQueryClient();

  const form = useForm<MilestoneFormData>({
    resolver: zodResolver(milestoneSchema),
    defaultValues: {
      milestone_date: "",
      title: "",
      description: "",
      milestone_type: "",
      importance: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["milestone-form-options"],
    queryFn: () => api.get<MilestoneFormOptions>("/api/v1/milestones/form_options"),
    enabled: open,
  });
  const milestoneTypeOptions = formOptions?.milestone_type_options ?? [];
  const importanceOptions = formOptions?.importance_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset({
      milestone_date: milestone?.milestone_date || "",
      title: milestone?.title || "",
      description: milestone?.description || "",
      milestone_type: milestone?.milestone_type || defaults.milestone_type,
      importance: milestone?.importance || defaults.importance,
      notes: milestone?.notes || "",
    });
  }, [open, milestone, formOptions?.default_values, form]);

  const onSubmit = async (data: MilestoneFormData) => {
    setIsSubmitting(true);
    try {
      const action = milestone?.update_action ?? createAction;
      if (!action) {
        toast.error("当前无法保存里程碑");
        return;
      }
      await api.runAction(action, {
        milestone_date: data.milestone_date,
        title: data.title,
        description: data.description,
        milestone_type: data.milestone_type,
        importance: data.importance,
        notes: data.notes,
      });
      queryClient.invalidateQueries({ queryKey: ["company-info-overview"] });
      queryClient.invalidateQueries({ queryKey: ["timeline-milestones"] });
      toast.success(action.success_message || "里程碑已保存");
      onOpenChange(false);
      if (!milestone) form.reset();
    } catch (error) {
      console.error("保存里程碑失败:", error);
      toast.error("保存里程碑失败");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{milestone ? "编辑里程碑" : "添加里程碑"}</DialogTitle>
          <DialogDescription>
            标记企业发展过程中的重要时刻和事件
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="milestone_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>日期 *</FormLabel>
                    <FormControl>
                      <Input type="date" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="milestone_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>类型 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择类型" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {milestoneTypeOptions.map((option) => (
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

            <FormField
              control={form.control}
              name="importance"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>重要程度 *</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="选择重要程度" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {importanceOptions.map((option) => (
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
              name="title"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>标题 *</FormLabel>
                  <FormControl>
                    <Input placeholder="输入里程碑标题" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>描述</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="描述这个里程碑的详细信息"
                      className="resize-none"
                      rows={3}
                      {...field}
                    />
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
                      placeholder="添加额外的备注信息"
                      className="resize-none"
                      rows={2}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="flex justify-end gap-2 pt-4">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
              >
                取消
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? "保存中..." : milestone ? "更新" : "创建"}
              </Button>
            </div>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
};
