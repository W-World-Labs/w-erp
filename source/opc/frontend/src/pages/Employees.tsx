import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  Brain,
  Briefcase,
  Calendar,
  Mail,
  Phone,
  Plus,
  Users,
} from "lucide-react";
import { EmployeeDialog } from "@/components/employees/EmployeeDialog";
import { toast } from "sonner";
import { DependencyAlert } from "@/components/common/DependencyAlert";
import { AIAuditDialog } from "@/components/applications/AIAuditDialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";

interface EmployeeBadge {
  label: string;
  variant: "default" | "secondary" | "outline" | "destructive";
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
}

interface EmployeeOverviewItem {
  id: string;
  full_name?: string;
  display_name: string;
  employee_number?: string;
  avatar_url?: string;
  status_badge: EmployeeBadge;
  employment_type_badge: EmployeeBadge;
  department?: string;
  position?: string;
  email?: string;
  phone?: string;
  hire_date_label: string;
  base_salary?: number;
  base_salary_label: string;
  notes?: string;
  can_delete: boolean;
  update_action: BackendAction | null;
  delete_action: DeleteAction | null;
}

interface EmployeesOverview {
  employees: EmployeeOverviewItem[];
  empty_message: string;
  create_action: BackendAction | null;
}

const emptyOverview: EmployeesOverview = {
  employees: [],
  empty_message: "还没有员工记录",
  create_action: null,
};

function EmployeeBadgeView({ badge }: { badge: EmployeeBadge }) {
  return <Badge variant={badge.variant}>{badge.label}</Badge>;
}

export default function Employees() {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingEmployee, setEditingEmployee] = useState<any>(null);
  const [aiAuditOpen, setAiAuditOpen] = useState(false);
  const [aiAuditEmployee, setAiAuditEmployee] =
    useState<EmployeeOverviewItem | null>(null);
  const [employeeToDelete, setEmployeeToDelete] =
    useState<DeleteAction | null>(null);

  const { data: overview = emptyOverview, refetch } = useQuery({
    queryKey: ["employees-overview"],
    queryFn: () => api.get<EmployeesOverview>("/api/employees/overview"),
  });
  const employees = overview.employees;

  const handleEdit = (employee: EmployeeOverviewItem) => {
    setEditingEmployee(employee);
    setDialogOpen(true);
  };

  const handleDelete = async (action: DeleteAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message);
      refetch();
    } catch (err: any) {
      if (err?.status === 409) {
        toast.error("该员工存在关联的薪资或社保记录，无法删除");
      } else {
        toast.error("删除失败");
      }
    } finally {
      setEmployeeToDelete(null);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">员工管理</h1>
          <p className="mt-2 text-muted-foreground">
            管理公司员工信息，追踪入职离职
          </p>
        </div>
        <Button
          disabled={!overview.create_action}
          onClick={() => {
            setEditingEmployee(null);
            setDialogOpen(true);
          }}
        >
          <Plus className="mr-2 h-4 w-4" />
          添加员工
        </Button>
      </div>

      <DependencyAlert dependency="company-info" />

      <div className="grid gap-4">
        {employees.map((employee) => (
          <Card key={employee.id} className="p-6">
            <div className="flex items-start gap-6">
              <Avatar className="h-16 w-16">
                <AvatarImage src={employee.avatar_url} />
                <AvatarFallback>
                  <Users className="h-8 w-8" />
                </AvatarFallback>
              </Avatar>

              <div className="flex-1 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <h3 className="text-xl font-semibold">
                      {employee.display_name}
                    </h3>
                    {employee.employee_number && (
                      <span className="text-sm text-muted-foreground">
                        ({employee.employee_number})
                      </span>
                    )}
                    <EmployeeBadgeView badge={employee.status_badge} />
                    <EmployeeBadgeView badge={employee.employment_type_badge} />
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setAiAuditEmployee(employee);
                        setAiAuditOpen(true);
                      }}
                    >
                      <Brain className="mr-1 h-4 w-4" />
                      AI审核
                    </Button>
                    {employee.update_action && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleEdit(employee)}
                      >
                        编辑
                      </Button>
                    )}
                    {employee.can_delete && employee.delete_action && (
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => setEmployeeToDelete(employee.delete_action)}
                      >
                        删除
                      </Button>
                    )}
                  </div>
                </div>

                <div className="grid grid-cols-2 gap-4 text-sm md:grid-cols-4">
                  {employee.department && (
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <Briefcase className="h-4 w-4" />
                      <span>{employee.department}</span>
                    </div>
                  )}
                  {employee.position && (
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <span>职位：{employee.position}</span>
                    </div>
                  )}
                  {employee.email && (
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <Mail className="h-4 w-4" />
                      <span>{employee.email}</span>
                    </div>
                  )}
                  {employee.phone && (
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <Phone className="h-4 w-4" />
                      <span>{employee.phone}</span>
                    </div>
                  )}
                  <div className="flex items-center gap-2 text-muted-foreground">
                    <Calendar className="h-4 w-4" />
                    <span>入职：{employee.hire_date_label}</span>
                  </div>
                  {employee.base_salary ? (
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <span>基本工资：{employee.base_salary_label}</span>
                    </div>
                  ) : null}
                </div>

                {employee.notes && (
                  <p className="text-sm text-muted-foreground">
                    {employee.notes}
                  </p>
                )}
              </div>
            </div>
          </Card>
        ))}
      </div>

      {employees.length === 0 && (
        <Card className="p-12 text-center">
          <Users className="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
          <h3 className="mb-2 text-lg font-semibold">
            {overview.empty_message}
          </h3>
          <p className="mb-4 text-muted-foreground">
            开始添加公司员工信息，建立完整的人事档案
          </p>
          <Button
            disabled={!overview.create_action}
            onClick={() => {
              setEditingEmployee(null);
              setDialogOpen(true);
            }}
          >
            <Plus className="mr-2 h-4 w-4" />
            添加第一位员工
          </Button>
        </Card>
      )}

      <EmployeeDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        employee={editingEmployee}
        action={
          editingEmployee ? editingEmployee.update_action : overview.create_action
        }
        onSuccess={() => {
          refetch();
          setDialogOpen(false);
          setEditingEmployee(null);
        }}
      />

      <AIAuditDialog
        open={aiAuditOpen}
        onOpenChange={setAiAuditOpen}
        auditType="employee"
        targetId={aiAuditEmployee?.id}
        title={`员工评估：${aiAuditEmployee?.display_name || ""}`}
      />

      <AlertDialog
        open={!!employeeToDelete}
        onOpenChange={() => setEmployeeToDelete(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {employeeToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const action = employeeToDelete;
                if (action) handleDelete(action);
              }}
            >
              确认删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
