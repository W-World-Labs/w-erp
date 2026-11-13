import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Separator } from "@/components/ui/separator";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { Loader2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { ProjectDocuments } from "./ProjectDocuments";
import { api } from "@/integrations/api/client";

const projectSchema = z.object({
  name: z.string().min(1, "项目名称不能为空"),
  project_type: z.string().min(1, "请选择项目类型"),
  leader_name: z.string().min(1, "请输入负责人"),
  budget: z.string().min(1, "项目预算不能为空"),
  start_date: z.string().optional(),
  end_date: z.string().min(1, "请选择结束日期"),
  goals: z.string().optional(),
  milestones: z.string().optional(),
  team_members: z.string().optional(),
  notes: z.string().optional(),
});

type ProjectFormValues = z.infer<typeof projectSchema>;

interface ProjectDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (values: any) => void;
  initialData?: any;
  isSubmitting?: boolean;
  projectId?: string;
  onProjectCreated?: (projectId: string) => void;
}

interface ProjectFormOption {
  value: string;
  label: string;
}

interface ProjectFormOptions {
  project_type_options: ProjectFormOption[];
  default_values: {
    project_type: string;
    team_members: number;
  };
}

export function ProjectDialog({ open, onOpenChange, onSubmit, initialData, isSubmitting, projectId, onProjectCreated }: ProjectDialogProps) {
  const [currentProjectId, setCurrentProjectId] = useState<string | undefined>(projectId || initialData?.id);
  
  const form = useForm<ProjectFormValues>({
    resolver: zodResolver(projectSchema),
    defaultValues: {
      name: "",
      project_type: "",
      leader_name: "",
      budget: "",
      start_date: "",
      end_date: "",
      goals: "",
      milestones: "",
      team_members: "",
      notes: "",
    },
  });
  const { data: formOptions } = useQuery({
    queryKey: ["project-form-options"],
    queryFn: () => api.get<ProjectFormOptions>("/api/v1/projects/form_options"),
    enabled: open,
  });
  const projectTypeOptions = formOptions?.project_type_options ?? [];

  useEffect(() => {
    if (!open || !formOptions?.default_values) {
      return;
    }
    const defaults = formOptions.default_values;
    form.reset(initialData ? {
      name: initialData.name || "",
      project_type: initialData.project_type || defaults.project_type,
      leader_name: initialData.leader_name || "",
      budget: initialData.budget?.toString() || "",
      start_date: initialData.start_date || "",
      end_date: initialData.end_date || "",
      goals: initialData.goals || "",
      milestones: initialData.milestones || "",
      team_members: initialData.team_members?.toString() || String(defaults.team_members),
      notes: initialData.notes || "",
    } : {
      name: "",
      project_type: defaults.project_type,
      leader_name: "",
      budget: "",
      start_date: "",
      end_date: "",
      goals: "",
      milestones: "",
      team_members: String(defaults.team_members),
      notes: "",
    });
    setCurrentProjectId(projectId || initialData?.id);
  }, [open, initialData, projectId, formOptions?.default_values, form]);

  const handleSubmit = (values: ProjectFormValues) => {
    onSubmit(values);
    
    if (!initialData) {
      form.reset();
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{initialData ? '编辑项目' : '新建项目'}</DialogTitle>
        </DialogHeader>
        
        <Form {...form}>
          <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem className="col-span-2">
                    <FormLabel>项目名称 *</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入项目名称" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="project_type"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>项目类型 *</FormLabel>
                    <Select onValueChange={field.onChange} value={field.value}>
                      <FormControl>
                        <SelectTrigger>
                          <SelectValue placeholder="选择项目类型" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        {projectTypeOptions.map((option) => (
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
                name="leader_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>项目负责人 *</FormLabel>
                    <FormControl>
                      <Input {...field} placeholder="输入负责人姓名" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="budget"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>项目预算（元）*</FormLabel>
                    <FormControl>
                      <Input {...field} inputMode="decimal" placeholder="输入项目预算" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="team_members"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>团队成员数</FormLabel>
                    <FormControl>
                      <Input {...field} inputMode="numeric" placeholder="输入团队成员数" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="start_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>开始日期</FormLabel>
                    <FormControl>
                      <Input {...field} type="date" />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="end_date"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>结束日期 *</FormLabel>
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
              name="goals"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>项目目标</FormLabel>
                  <FormControl>
                    <Textarea {...field} placeholder="描述项目的主要目标和预期成果" rows={3} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="milestones"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>关键里程碑</FormLabel>
                  <FormControl>
                    <Textarea {...field} placeholder="列出项目的关键里程碑和时间节点" rows={3} />
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
                    <Textarea {...field} placeholder="输入备注信息" rows={2} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Separator />

            <div className="space-y-4">
              <h3 className="font-semibold">项目文档</h3>
              <ProjectDocuments 
                projectId={currentProjectId} 
                disabled={isSubmitting}
              />
            </div>

            <DialogFooter>
              <Button 
                type="button" 
                variant="outline" 
                onClick={() => {
                  onOpenChange(false);
                  if (!initialData && currentProjectId) {
                    // 如果是新建项目且有文档上传，关闭时清理状态
                    setCurrentProjectId(undefined);
                  }
                }}
                disabled={isSubmitting}
              >
                {currentProjectId && !initialData ? "完成" : "取消"}
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    处理中...
                  </>
                ) : (
                  initialData ? '更新' : '创建'
                )}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
