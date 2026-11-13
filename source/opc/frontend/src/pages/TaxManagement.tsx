import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  AlertCircle,
  Edit,
  FileText,
  Plus,
  Trash2,
  TrendingUp,
} from "lucide-react";
import { TaxDeclarationDialog } from "@/components/tax/TaxDeclarationDialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
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
import { toast } from "sonner";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

type BadgeVariant = "default" | "secondary" | "destructive" | "outline";

interface TaxBadge {
  value: string;
  label: string;
  variant: BadgeVariant;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

interface TaxDeclaration {
  id: string;
  declaration_type_label: string;
  declaration_period_label: string;
  period_label: string;
  deadline_label: string;
  declaration_date_label: string;
  tax_amount_label: string;
  declared_amount_label: string;
  status_badge: TaxBadge;
  can_delete: boolean;
  update_action: BackendAction | null;
  delete_action: DeleteAction | null;
}

interface TaxManagementSummary {
  taxpayer: {
    company_name?: string | null;
    taxpayer_type?: string | null;
    taxpayer_type_label: string;
    tax_registration_number?: string | null;
    tax_authority?: string | null;
    vat_rate?: number | null;
    vat_rate_label: string;
    invoice_limit?: number | null;
    invoice_limit_label: string;
  };
  declaration_counts: {
    pending: number;
    declared: number;
    overdue: number;
    total: number;
  };
  invoice_tax: {
    deductible_total: number;
    deductible_total_label: string;
    output_tax_due: number;
    output_tax_due_label: string;
  };
  declarations: TaxDeclaration[];
  pending_declarations: TaxDeclaration[];
  declared_declarations: TaxDeclaration[];
  permissions: {
    can_delete: boolean;
  };
  empty_messages: {
    all: string;
    pending: string;
    declared: string;
  };
  declaration_create_action: BackendAction | null;
}

const TAX_MANAGEMENT_SUMMARY_ENDPOINT = "/api/v1/tax/management-summary";

const emptyTaxSummary: TaxManagementSummary = {
  taxpayer: {
    taxpayer_type_label: "未设置",
    vat_rate_label: "3%",
    invoice_limit_label: "未设置",
  },
  declaration_counts: {
    pending: 0,
    declared: 0,
    overdue: 0,
    total: 0,
  },
  invoice_tax: {
    deductible_total: 0,
    deductible_total_label: "¥0",
    output_tax_due: 0,
    output_tax_due_label: "¥0",
  },
  declarations: [],
  pending_declarations: [],
  declared_declarations: [],
  permissions: {
    can_delete: false,
  },
  empty_messages: {
    all: "暂无税务申报记录",
    pending: "暂无待申报事项",
    declared: "暂无已申报记录",
  },
  declaration_create_action: null,
};

function TaxStatusBadge({ badge }: { badge: TaxBadge }) {
  return <Badge variant={badge.variant}>{badge.label}</Badge>;
}

const TaxManagement = () => {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedDeclaration, setSelectedDeclaration] =
    useState<TaxDeclaration | null>(null);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [declarationToDelete, setDeclarationToDelete] =
    useState<DeleteAction | null>(null);

  const { data: taxSummary = emptyTaxSummary, refetch: refetchSummary } =
    useQuery({
      queryKey: ["tax-management-summary"],
      queryFn: async () => {
        return api.get<TaxManagementSummary>(TAX_MANAGEMENT_SUMMARY_ENDPOINT);
      },
    });

  const companyInfo = taxSummary.taxpayer;
  const declarationCounts = taxSummary.declaration_counts;
  const invoiceTax = taxSummary.invoice_tax;
  const declarations = taxSummary.declarations;
  const pendingDeclarations = taxSummary.pending_declarations;
  const declaredDeclarations = taxSummary.declared_declarations;
  const canDelete = taxSummary.permissions.can_delete;

  const refreshTaxData = () => {
    refetchSummary();
  };

  const handleAdd = () => {
    setSelectedDeclaration(null);
    setDialogOpen(true);
  };

  const handleEdit = (declaration: TaxDeclaration) => {
    setSelectedDeclaration(declaration);
    setDialogOpen(true);
  };

  const handleDeleteClick = (declaration: TaxDeclaration) => {
    setDeclarationToDelete(declaration.delete_action);
    setDeleteDialogOpen(true);
  };

  const handleDelete = async () => {
    if (!declarationToDelete) return;

    try {
      await api.runAction(declarationToDelete);
      toast.success(declarationToDelete.success_message);
      refreshTaxData();
    } catch (error) {
      console.error("Error deleting declaration:", error);
      toast.error("删除失败");
    } finally {
      setDeleteDialogOpen(false);
      setDeclarationToDelete(null);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="flex items-center gap-2 text-3xl font-bold">
            <FileText className="h-8 w-8 text-primary" />
            税务管理
          </h1>
          <p className="mt-2 text-muted-foreground">
            管理纳税人资格、税务申报和进项税额
          </p>
        </div>
        <Button onClick={handleAdd} disabled={!taxSummary.declaration_create_action}>
          <Plus className="mr-2 h-4 w-4" />
          添加申报
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>纳税人信息</CardTitle>
          <CardDescription>当前企业的纳税人资格和税务配置</CardDescription>
        </CardHeader>
        <CardContent>
          {companyInfo ? (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
              <div>
                <div className="text-sm text-muted-foreground">纳税人类型</div>
                <div className="mt-1 font-medium">
                  <Badge variant="outline">
                    {companyInfo.taxpayer_type_label}
                  </Badge>
                </div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground">增值税税率</div>
                <div className="mt-1 font-medium">
                  {companyInfo.vat_rate_label}
                </div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground">
                  单张发票开具限额
                </div>
                <div className="mt-1 font-medium">
                  {companyInfo.invoice_limit_label}
                </div>
              </div>
              {companyInfo.tax_registration_number && (
                <div>
                  <div className="text-sm text-muted-foreground">
                    税务登记号
                  </div>
                  <div className="mt-1 font-medium">
                    {companyInfo.tax_registration_number}
                  </div>
                </div>
              )}
              {companyInfo.tax_authority && (
                <div className="md:col-span-2">
                  <div className="text-sm text-muted-foreground">
                    主管税务机关
                  </div>
                  <div className="mt-1 font-medium">
                    {companyInfo.tax_authority}
                  </div>
                </div>
              )}
            </div>
          ) : (
            <div className="py-6 text-center text-muted-foreground">
              <p>请先在公司信息中配置纳税人类型</p>
            </div>
          )}
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">待申报</CardTitle>
            <AlertCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {declarationCounts.pending}
            </div>
            <p className="text-xs text-muted-foreground">需要处理的申报</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">已申报</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {declarationCounts.declared}
            </div>
            <p className="text-xs text-muted-foreground">已完成的申报</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">进项税额</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {invoiceTax.deductible_total_label}
            </div>
            <p className="text-xs text-muted-foreground">可抵扣金额</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">销项税额</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {invoiceTax.output_tax_due_label}
            </div>
            <p className="text-xs text-muted-foreground">应纳税额估算</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>税务申报记录</CardTitle>
          <CardDescription>管理所有税务申报事项</CardDescription>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="all" className="w-full">
            <TabsList>
              <TabsTrigger value="all">全部</TabsTrigger>
              <TabsTrigger value="pending">待申报</TabsTrigger>
              <TabsTrigger value="declared">已申报</TabsTrigger>
            </TabsList>

            <TabsContent value="all" className="mt-4">
              {declarations.length > 0 ? (
                <div className="rounded-md border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>申报类型</TableHead>
                        <TableHead>申报周期</TableHead>
                        <TableHead>申报期间</TableHead>
                        <TableHead>截止日期</TableHead>
                        <TableHead>应纳税额</TableHead>
                        <TableHead>状态</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {declarations.map((declaration) => (
                        <TableRow key={declaration.id}>
                          <TableCell className="font-medium">
                            {declaration.declaration_type_label}
                          </TableCell>
                          <TableCell>
                            {declaration.declaration_period_label}
                          </TableCell>
                          <TableCell>{declaration.period_label}</TableCell>
                          <TableCell>{declaration.deadline_label}</TableCell>
                          <TableCell>{declaration.tax_amount_label}</TableCell>
                          <TableCell>
                            <TaxStatusBadge badge={declaration.status_badge} />
                          </TableCell>
                          <TableCell className="text-right">
                            <div className="flex justify-end gap-2">
                              {declaration.update_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => handleEdit(declaration)}
                                >
                                  <Edit className="h-4 w-4" />
                                </Button>
                              )}
                              {declaration.can_delete &&
                                canDelete &&
                                declaration.delete_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => handleDeleteClick(declaration)}
                                >
                                  <Trash2 className="h-4 w-4 text-destructive" />
                                </Button>
                              )}
                            </div>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              ) : (
                <div className="py-12 text-center text-muted-foreground">
                  <FileText className="mx-auto mb-4 h-16 w-16 opacity-50" />
                  <p>{taxSummary.empty_messages.all}</p>
                </div>
              )}
            </TabsContent>

            <TabsContent value="pending" className="mt-4">
              {pendingDeclarations.length > 0 ? (
                <div className="rounded-md border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>申报类型</TableHead>
                        <TableHead>申报周期</TableHead>
                        <TableHead>截止日期</TableHead>
                        <TableHead>应纳税额</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {pendingDeclarations.map((declaration) => (
                        <TableRow key={declaration.id}>
                          <TableCell className="font-medium">
                            {declaration.declaration_type_label}
                          </TableCell>
                          <TableCell>
                            {declaration.declaration_period_label}
                          </TableCell>
                          <TableCell>{declaration.deadline_label}</TableCell>
                          <TableCell>{declaration.tax_amount_label}</TableCell>
                          <TableCell className="text-right">
                            {declaration.update_action && (
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => handleEdit(declaration)}
                              >
                                处理申报
                              </Button>
                            )}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              ) : (
                <div className="py-12 text-center text-muted-foreground">
                  <p>{taxSummary.empty_messages.pending}</p>
                </div>
              )}
            </TabsContent>

            <TabsContent value="declared" className="mt-4">
              {declaredDeclarations.length > 0 ? (
                <div className="rounded-md border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>申报类型</TableHead>
                        <TableHead>申报日期</TableHead>
                        <TableHead>申报金额</TableHead>
                        <TableHead>应纳税额</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {declaredDeclarations.map((declaration) => (
                        <TableRow key={declaration.id}>
                          <TableCell className="font-medium">
                            {declaration.declaration_type_label}
                          </TableCell>
                          <TableCell>
                            {declaration.declaration_date_label}
                          </TableCell>
                          <TableCell>
                            {declaration.declared_amount_label}
                          </TableCell>
                          <TableCell>{declaration.tax_amount_label}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              ) : (
                <div className="py-12 text-center text-muted-foreground">
                  <p>{taxSummary.empty_messages.declared}</p>
                </div>
              )}
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      <TaxDeclarationDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        declaration={selectedDeclaration}
        action={
          selectedDeclaration
            ? selectedDeclaration.update_action
            : taxSummary.declaration_create_action
        }
        onSuccess={refreshTaxData}
      />

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {declarationToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete}>删除</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

export default TaxManagement;
