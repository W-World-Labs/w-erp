import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Shield,
  Landmark,
  Plus,
  Trash2,
  Pencil,
  AlertTriangle,
} from "lucide-react";

type DisplayBadge = {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  class_name?: string;
};

type DeleteAction = {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
};

function BackendBadge({ badge }: { badge?: DisplayBadge | null }) {
  if (!badge) return null;
  return (
    <Badge variant={badge.variant || "outline"} className={badge.class_name || ""}>
      {badge.label}
    </Badge>
  );
}

function ComplianceManagement() {
  const queryClient = useQueryClient();
  const [insuranceDialogOpen, setInsuranceDialogOpen] = useState(false);
  const [editingInsurance, setEditingInsurance] = useState<any>(null);
  const [pensionDialogOpen, setPensionDialogOpen] = useState(false);
  const [editingPension, setEditingPension] = useState<any>(null);

  const { data: overview, isLoading } = useQuery({
    queryKey: ["compliance-overview"],
    queryFn: () => api.get<any>(API_ENDPOINTS.complianceOverview),
  });

  const insurances = overview?.insurance_policies || [];
  const pensions = overview?.pension_plans || [];
  const complianceStats = overview?.stats || {
    active_insurance_count: 0,
    expired_insurance_count: 0,
    active_pension_count: 0,
    monthly_compliance_cost: 0,
    monthly_compliance_cost_label: "¥0",
  };
  const insuranceTypeOptions = overview?.insurance_type_options || [];
  const pensionTypeOptions = overview?.pension_type_options || [];
  const countryOptions = overview?.country_options || [];

  const saveIns = useMutation({
    mutationFn: async ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, unknown>;
    }) => {
      await api.runAction(action, values);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["compliance-overview"] });
      toast.success(action.success_message || "保险单已保存");
      setInsuranceDialogOpen(false);
      setEditingInsurance(null);
    },
  });
  const deleteIns = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["compliance-overview"] });
      toast.success(action.success_message);
    },
  });
  const savePen = useMutation({
    mutationFn: async ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, unknown>;
    }) => {
      await api.runAction(action, values);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["compliance-overview"] });
      toast.success(action.success_message || "年金计划已保存");
      setPensionDialogOpen(false);
      setEditingPension(null);
    },
  });
  const deletePen = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["compliance-overview"] });
      toast.success(action.success_message);
    },
  });

  const handleInsSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const v: Record<string, any> = {};
    fd.forEach((val, k) => {
      if (val) v[k] = val;
    });
    if (v.auto_renew) v.auto_renew = v.auto_renew === "1" ? 1 : 0;
    const action = editingInsurance
      ? editingInsurance.update_action
      : overview?.insurance_create_action;
    if (!action) {
      toast.error("当前无法保存保险单");
      return;
    }
    saveIns.mutate({ action, values: v });
  };

  const handlePenSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const fd = new FormData(e.currentTarget);
    const v: Record<string, any> = {};
    fd.forEach((val, k) => {
      if (val) v[k] = val;
    });
    const action = editingPension
      ? editingPension.update_action
      : overview?.pension_create_action;
    if (!action) {
      toast.error("当前无法保存年金计划");
      return;
    }
    savePen.mutate({ action, values: v });
  };

  const handleDeleteInsurance = (action?: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deleteIns.mutate(action);
  };

  const handleDeletePension = (action?: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deletePen.mutate(action);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">合规管理</h1>
          <p className="text-muted-foreground">
            保险单管理、年金/社保计划管理、到期提醒
          </p>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <Shield className="h-5 w-5 text-blue-500" />
              <div>
                <div className="text-2xl font-bold">
                  {complianceStats.active_insurance_count}
                </div>
                <p className="text-xs text-muted-foreground">有效保单</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-red-500" />
              <div>
                <div className="text-2xl font-bold">
                  {complianceStats.expired_insurance_count}
                </div>
                <p className="text-xs text-muted-foreground">已过期保单</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <Landmark className="h-5 w-5 text-green-500" />
              <div>
                <div className="text-2xl font-bold">
                  {complianceStats.active_pension_count}
                </div>
                <p className="text-xs text-muted-foreground">活跃年金计划</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <Landmark className="h-5 w-5 text-purple-500" />
              <div>
                <div className="text-2xl font-bold">
                  {complianceStats.monthly_compliance_cost_label}
                </div>
                <p className="text-xs text-muted-foreground">月度合规成本</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <Tabs defaultValue="insurance">
        <TabsList>
          <TabsTrigger value="insurance">保险管理</TabsTrigger>
          <TabsTrigger value="pension">年金/社保</TabsTrigger>
        </TabsList>

        {/* Insurance Tab */}
        <TabsContent value="insurance" className="space-y-4">
          <div className="flex justify-end">
            <Dialog
              open={insuranceDialogOpen}
              onOpenChange={(o) => {
                setInsuranceDialogOpen(o);
                if (!o) setEditingInsurance(null);
              }}
            >
              <DialogTrigger asChild>
                <Button disabled={!overview?.insurance_create_action}>
                  <Plus className="h-4 w-4 mr-2" /> 新增保险单
                </Button>
              </DialogTrigger>
              <DialogContent className="max-w-lg">
                <DialogHeader>
                  <DialogTitle>
                    {editingInsurance ? "编辑保险单" : "新增保险单"}
                  </DialogTitle>
                </DialogHeader>
                <form onSubmit={handleInsSubmit} className="space-y-4">
                  <div>
                    <Label>保单名称 *</Label>
                    <Input
                      name="policy_name"
                      required
                      defaultValue={editingInsurance?.policy_name}
                    />
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>保险类型 *</Label>
                      <Select
                        name="insurance_type"
                        defaultValue={
                          editingInsurance?.insurance_type || "health"
                        }
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {insuranceTypeOptions.map((option: any) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <div>
                      <Label>保单号</Label>
                      <Input
                        name="policy_number"
                        defaultValue={editingInsurance?.policy_number}
                      />
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>保险公司</Label>
                      <Input
                        name="provider"
                        defaultValue={editingInsurance?.provider}
                      />
                    </div>
                    <div>
                      <Label>国家/地区</Label>
                      <Input
                        name="country"
                        defaultValue={editingInsurance?.country}
                      />
                    </div>
                  </div>
                  <div className="grid grid-cols-3 gap-4">
                    <div>
                      <Label>保费</Label>
                      <Input
                        name="premium_amount"
                        type="number"
                        step="0.01"
                        defaultValue={editingInsurance?.premium_amount}
                      />
                    </div>
                    <div>
                      <Label>保额</Label>
                      <Input
                        name="coverage_amount"
                        type="number"
                        step="0.01"
                        defaultValue={editingInsurance?.coverage_amount}
                      />
                    </div>
                    <div>
                      <Label>免赔额</Label>
                      <Input
                        name="deductible"
                        type="number"
                        step="0.01"
                        defaultValue={editingInsurance?.deductible}
                      />
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>开始日期 *</Label>
                      <Input
                        name="start_date"
                        type="date"
                        required
                        defaultValue={editingInsurance?.start_date}
                      />
                    </div>
                    <div>
                      <Label>结束日期 *</Label>
                      <Input
                        name="end_date"
                        type="date"
                        required
                        defaultValue={editingInsurance?.end_date}
                      />
                    </div>
                  </div>
                  <div>
                    <Label>备注</Label>
                    <Textarea
                      name="notes"
                      defaultValue={editingInsurance?.notes}
                    />
                  </div>
                  <Button type="submit" className="w-full">
                    {editingInsurance ? "保存" : "创建"}
                  </Button>
                </form>
              </DialogContent>
            </Dialog>
          </div>
          <Card>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>保单名称</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>保险公司</TableHead>
                    <TableHead>保费</TableHead>
                    <TableHead>保额</TableHead>
                    <TableHead>有效期</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={8} className="text-center py-8">
                        加载中...
                      </TableCell>
                    </TableRow>
                  ) : insurances.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={8}
                        className="text-center py-8 text-muted-foreground"
                      >
                        暂无保险单
                      </TableCell>
                    </TableRow>
                  ) : (
                    insurances.map((i: any) => (
                      <TableRow key={i.id}>
                        <TableCell className="font-medium">
                          {i.policy_name}
                        </TableCell>
                        <TableCell>
                          {i.insurance_type_label}
                        </TableCell>
                        <TableCell>{i.provider || "-"}</TableCell>
                        <TableCell>{i.premium_amount_label}</TableCell>
                        <TableCell>{i.coverage_amount_label}</TableCell>
                        <TableCell className="text-sm">
                          {i.coverage_period_label}
                        </TableCell>
                        <TableCell>
                          <BackendBadge badge={i.status_badge} />
                        </TableCell>
                        <TableCell className="text-right space-x-1">
                          {i.update_action && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => {
                                setEditingInsurance(i);
                                setInsuranceDialogOpen(true);
                              }}
                            >
                              <Pencil className="h-4 w-4" />
                            </Button>
                          )}
                          {i.delete_action && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleDeleteInsurance(i.delete_action)}
                            >
                              <Trash2 className="h-4 w-4 text-red-500" />
                            </Button>
                          )}
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Pension Tab */}
        <TabsContent value="pension" className="space-y-4">
          <div className="flex justify-end">
            <Dialog
              open={pensionDialogOpen}
              onOpenChange={(o) => {
                setPensionDialogOpen(o);
                if (!o) setEditingPension(null);
              }}
            >
              <DialogTrigger asChild>
                <Button disabled={!overview?.pension_create_action}>
                  <Plus className="h-4 w-4 mr-2" /> 新增年金计划
                </Button>
              </DialogTrigger>
              <DialogContent className="max-w-lg">
                <DialogHeader>
                  <DialogTitle>
                    {editingPension ? "编辑年金计划" : "新增年金计划"}
                  </DialogTitle>
                </DialogHeader>
                <form onSubmit={handlePenSubmit} className="space-y-4">
                  <div>
                    <Label>计划名称 *</Label>
                    <Input
                      name="plan_name"
                      required
                      defaultValue={editingPension?.plan_name}
                    />
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>计划类型 *</Label>
                      <Select
                        name="plan_type"
                        defaultValue={
                          editingPension?.plan_type || "national_pension"
                        }
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {pensionTypeOptions.map((option: any) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <div>
                      <Label>适用国家</Label>
                      <Select
                        name="country"
                        defaultValue={editingPension?.country || "KR"}
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {countryOptions.map((option: any) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>雇主缴纳</Label>
                      <Input
                        name="contribution_employer"
                        type="number"
                        step="0.01"
                        defaultValue={editingPension?.contribution_employer}
                      />
                    </div>
                    <div>
                      <Label>员工缴纳</Label>
                      <Input
                        name="contribution_employee"
                        type="number"
                        step="0.01"
                        defaultValue={editingPension?.contribution_employee}
                      />
                    </div>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <Label>管理机构</Label>
                      <Input
                        name="provider"
                        defaultValue={editingPension?.provider}
                      />
                    </div>
                    <div>
                      <Label>当前价值</Label>
                      <Input
                        name="current_value"
                        type="number"
                        step="0.01"
                        defaultValue={editingPension?.current_value}
                      />
                    </div>
                  </div>
                  <div>
                    <Label>备注</Label>
                    <Textarea
                      name="notes"
                      defaultValue={editingPension?.notes}
                    />
                  </div>
                  <Button type="submit" className="w-full">
                    {editingPension ? "保存" : "创建"}
                  </Button>
                </form>
              </DialogContent>
            </Dialog>
          </div>
          <Card>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>计划名称</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>国家</TableHead>
                    <TableHead>雇主缴纳</TableHead>
                    <TableHead>员工缴纳</TableHead>
                    <TableHead>当前价值</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={8} className="text-center py-8">
                        加载中...
                      </TableCell>
                    </TableRow>
                  ) : pensions.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={8}
                        className="text-center py-8 text-muted-foreground"
                      >
                        暂无年金/社保计划
                      </TableCell>
                    </TableRow>
                  ) : (
                    pensions.map((p: any) => (
                      <TableRow key={p.id}>
                        <TableCell className="font-medium">
                          {p.plan_name}
                        </TableCell>
                        <TableCell>
                          {p.plan_type_label}
                        </TableCell>
                        <TableCell>{p.country_label}</TableCell>
                        <TableCell>{p.contribution_employer_label}</TableCell>
                        <TableCell>{p.contribution_employee_label}</TableCell>
                        <TableCell>{p.current_value_label}</TableCell>
                        <TableCell>
                          <BackendBadge badge={p.status_badge} />
                        </TableCell>
                        <TableCell className="text-right space-x-1">
                          {p.update_action && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => {
                                setEditingPension(p);
                                setPensionDialogOpen(true);
                              }}
                            >
                              <Pencil className="h-4 w-4" />
                            </Button>
                          )}
                          {p.delete_action && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleDeletePension(p.delete_action)}
                            >
                              <Trash2 className="h-4 w-4 text-red-500" />
                            </Button>
                          )}
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default ComplianceManagement;
