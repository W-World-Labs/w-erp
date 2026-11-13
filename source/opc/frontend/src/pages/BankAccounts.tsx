import { useState } from "react";
import { Link } from "react-router-dom";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
import { Skeleton } from "@/components/ui/skeleton";
import { Building2, Download, Plus, Scale, Upload } from "lucide-react";
import { QueryStateGate } from "@/components/common/QueryStateGate";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";

const defaultOverview = {
  stats: {
    total_balance_label: "¥0",
    active_accounts: 0,
    total_accounts: 0,
  },
  accounts: [] as any[],
};

export default function BankAccounts() {
  const [, setShowAccountDialog] = useState(false);

  const {
    data: overview = defaultOverview,
    isLoading,
    isError,
    refetch,
  } = useQuery({
    queryKey: ["bank-accounts-overview"],
    queryFn: async () =>
      api.get<typeof defaultOverview>(API_ENDPOINTS.bankAccountsOverview),
  });

  const loadingFallback = (
    <div className="container mx-auto p-6 space-y-6">
      <Skeleton className="h-12 w-64" />
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {[1, 2, 3].map((i) => (
          <Skeleton key={i} className="h-32" />
        ))}
      </div>
    </div>
  );

  const { stats, accounts } = overview;

  return (
    <QueryStateGate
      isLoading={isLoading}
      isError={isError}
      refetch={refetch}
      loadingFallback={loadingFallback}
    >
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">银行账户管理</h1>
          <p className="text-muted-foreground mt-2">
            管理公司银行账户，导入银行流水
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" asChild>
            <Link to="/reconciliation">
              <Scale className="h-4 w-4 mr-2" />
              对账中心
            </Link>
          </Button>
          <Button variant="outline">
            <Upload className="h-4 w-4 mr-2" />
            导入流水
          </Button>
          <Button onClick={() => setShowAccountDialog(true)}>
            <Plus className="h-4 w-4 mr-2" />
            添加账户
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card className="p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">账户总余额</p>
              <p className="text-2xl font-bold text-foreground mt-1">
                {stats.total_balance_label}
              </p>
            </div>
            <div className="p-3 bg-primary/10 rounded-full">
              <Building2 className="h-6 w-6 text-primary" />
            </div>
          </div>
        </Card>

        <Card className="p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">活跃账户</p>
              <p className="text-2xl font-bold text-foreground mt-1">
                {stats.active_accounts}
              </p>
            </div>
            <div className="p-3 bg-green-500/10 rounded-full">
              <Building2 className="h-6 w-6 text-green-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">总账户数</p>
              <p className="text-2xl font-bold text-foreground mt-1">
                {stats.total_accounts}
              </p>
            </div>
            <div className="p-3 bg-blue-500/10 rounded-full">
              <Building2 className="h-6 w-6 text-blue-600" />
            </div>
          </div>
        </Card>
      </div>

      <Card className="p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-foreground">账户列表</h2>
          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-2" />
            导出
          </Button>
        </div>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>账户名称</TableHead>
              <TableHead>开户行</TableHead>
              <TableHead>账号</TableHead>
              <TableHead>账户类型</TableHead>
              <TableHead>余额</TableHead>
              <TableHead>交易笔数</TableHead>
              <TableHead>状态</TableHead>
              <TableHead>操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {accounts.map((account: any) => {
              const summary = account.transaction_stats || {};
              const badge = account.status_badge || {};
              return (
                <TableRow key={account.id}>
                  <TableCell className="font-medium">
                    {account.account_name}
                  </TableCell>
                  <TableCell>{account.bank_name}</TableCell>
                  <TableCell className="font-mono text-sm">
                    {account.account_number}
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline">
                      {account.account_type_label}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-foreground font-medium">
                    {account.balance_label}
                  </TableCell>
                  <TableCell>
                    <div className="text-sm">
                      <p className="text-muted-foreground">
                        共 {summary.count} 笔
                      </p>
                      <p className="text-green-600">
                        收 {summary.income_label}
                      </p>
                      <p className="text-red-600">付 {summary.expense_label}</p>
                    </div>
                  </TableCell>
                  <TableCell>
                    <Badge variant={badge.variant || "secondary"}>
                      {badge.label}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <div className="flex gap-2">
                      <Button variant="ghost" size="sm">
                        查看流水
                      </Button>
                      <Button variant="ghost" size="sm">
                        编辑
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              );
            })}
          </TableBody>
        </Table>
      </Card>
    </div>
    </QueryStateGate>
  );
}
