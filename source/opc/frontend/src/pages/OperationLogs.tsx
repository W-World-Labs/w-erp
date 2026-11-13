import { useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Download, Search, Shield } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { API_ENDPOINTS } from "@/lib/api/contracts";
import { cn } from "@/lib/utils";

const defaultOverview = {
  stats: {
    total: 0,
    create: 0,
    update: 0,
    delete: 0,
  },
  logs: [] as any[],
  type_options: [] as any[],
  table_options: [] as any[],
  export: {
    filename: "operation_logs.csv",
    csv: "",
  },
  permissions: {
    can_view: true,
    can_export: false,
  },
};

const OperationLogs = () => {
  const [searchTerm, setSearchTerm] = useState("");
  const [typeFilter, setTypeFilter] = useState("all");
  const [tableFilter, setTableFilter] = useState("all");

  const { data: overview = defaultOverview, isLoading } = useQuery({
    queryKey: ["operation-logs-overview", searchTerm, typeFilter, tableFilter],
    queryFn: async () => {
      const params = new URLSearchParams();
      const trimmedSearch = searchTerm.trim();
      if (trimmedSearch) {
        params.set("search", trimmedSearch);
      }
      if (typeFilter !== "all") {
        params.set("type", typeFilter);
      }
      if (tableFilter !== "all") {
        params.set("table", tableFilter);
      }
      const query = params.toString();
      return api.get<typeof defaultOverview>(
        `${API_ENDPOINTS.operationLogsOverview}${query ? `?${query}` : ""}`,
      );
    },
  });

  const handleExport = () => {
    const blob = new Blob([`\uFEFF${overview.export.csv}`], {
      type: "text/csv;charset=utf-8;",
    });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = overview.export.filename;
    link.click();
    URL.revokeObjectURL(link.href);
  };

  const stats = overview.stats;
  const logs = overview.logs;
  const permissions = overview.permissions;

  if (!isLoading && !permissions.can_view) {
    return (
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5 text-muted-foreground" />
              操作日志
            </CardTitle>
            <CardDescription>当前账号无权查看系统操作日志</CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground flex items-center gap-2">
            <Shield className="h-8 w-8 text-primary" />
            操作日志
          </h1>
          <p className="text-muted-foreground mt-2">系统操作审计与追踪</p>
        </div>
        <Button
          variant="outline"
          className="gap-2"
          onClick={handleExport}
          disabled={!permissions.can_export}
        >
          <Download className="h-4 w-4" />
          导出日志
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">总操作数</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="text-xs text-muted-foreground">最近200条记录</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">新建操作</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.create}</div>
            <p className="text-xs text-muted-foreground">CREATE</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">修改操作</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.update}</div>
            <p className="text-xs text-muted-foreground">UPDATE</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">删除操作</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-destructive">
              {stats.delete}
            </div>
            <p className="text-xs text-muted-foreground">DELETE</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>操作记录</CardTitle>
          <CardDescription>最近的系统操作日志</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="mb-4 flex gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="搜索表名、操作类型..."
                className="pl-10"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
              />
            </div>
            <Select value={typeFilter} onValueChange={setTypeFilter}>
              <SelectTrigger className="w-[150px]">
                <SelectValue placeholder="操作类型" />
              </SelectTrigger>
              <SelectContent>
                {overview.type_options.map((kind) => (
                  <SelectItem key={kind.value} value={kind.value}>
                    {kind.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={tableFilter} onValueChange={setTableFilter}>
              <SelectTrigger className="w-[150px]">
                <SelectValue placeholder="数据表" />
              </SelectTrigger>
              <SelectContent>
                {overview.table_options.map((table) => (
                  <SelectItem key={table.value} value={table.value}>
                    {table.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {isLoading ? (
            <div className="text-center py-8 text-muted-foreground">
              加载中...
            </div>
          ) : logs.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              暂无日志记录
            </div>
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>时间</TableHead>
                    <TableHead>操作类型</TableHead>
                    <TableHead>数据表</TableHead>
                    <TableHead>记录ID</TableHead>
                    <TableHead>详情</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {logs.map((log) => {
                    const badge = log.operation_badge || {};
                    return (
                      <TableRow key={log.id}>
                        <TableCell className="font-mono text-sm">
                          {log.created_at_label}
                        </TableCell>
                        <TableCell>
                          <Badge
                            variant={badge.variant || "outline"}
                            className={cn(badge.class_name)}
                          >
                            {badge.label}
                          </Badge>
                        </TableCell>
                        <TableCell>
                          <Badge variant="outline">{log.table_label}</Badge>
                        </TableCell>
                        <TableCell className="font-mono text-xs">
                          {log.record_id_short}
                        </TableCell>
                        <TableCell className="max-w-xs truncate text-sm text-muted-foreground">
                          {log.details_label}
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
};

export default OperationLogs;
