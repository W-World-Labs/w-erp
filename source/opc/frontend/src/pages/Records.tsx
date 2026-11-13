import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Download, Filter, Search } from "lucide-react";
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
    checkout: 0,
    transfer: 0,
    maintenance: 0,
  },
  records: [] as any[],
  action_options: [] as any[],
  export: {
    filename: "asset_records.csv",
    csv: "",
  },
};

const Records = () => {
  const [searchTerm, setSearchTerm] = useState("");
  const [actionFilter, setActionFilter] = useState("all");

  const { data: overview = defaultOverview, isLoading } = useQuery({
    queryKey: ["asset-records-overview", searchTerm, actionFilter],
    queryFn: async () => {
      const params = new URLSearchParams();
      const trimmedSearch = searchTerm.trim();
      if (trimmedSearch) {
        params.set("search", trimmedSearch);
      }
      if (actionFilter !== "all") {
        params.set("action", actionFilter);
      }
      const query = params.toString();
      return api.get<typeof defaultOverview>(
        `${API_ENDPOINTS.assetRecordsOverview}${query ? `?${query}` : ""}`,
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
  const records = overview.records;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">使用记录</h1>
          <p className="text-muted-foreground mt-2">资产流转与使用记录管理</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" className="gap-2" onClick={handleExport}>
            <Download className="h-4 w-4" />
            导出记录
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">总记录数</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="text-xs text-muted-foreground">所有操作记录</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">领用记录</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.checkout}</div>
            <p className="text-xs text-muted-foreground">资产领用次数</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">调拨记录</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.transfer}</div>
            <p className="text-xs text-muted-foreground">跨部门调拨</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">维修记录</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.maintenance}</div>
            <p className="text-xs text-muted-foreground">维修次数</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>流水记录</CardTitle>
          </div>
        </CardHeader>
        <CardContent>
          <div className="mb-4 flex gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="搜索资产、使用人、部门..."
                className="pl-10"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
              />
            </div>
            <Select value={actionFilter} onValueChange={setActionFilter}>
              <SelectTrigger className="w-[180px]">
                <Filter className="mr-2 h-4 w-4" />
                <SelectValue placeholder="操作类型" />
              </SelectTrigger>
              <SelectContent>
                {overview.action_options.map((action) => (
                  <SelectItem key={action.value} value={action.value}>
                    {action.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {isLoading ? (
            <div className="text-center py-8 text-muted-foreground">
              加载中...
            </div>
          ) : records.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              暂无记录
            </div>
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>资产编号</TableHead>
                    <TableHead>资产名称</TableHead>
                    <TableHead>使用人</TableHead>
                    <TableHead>部门</TableHead>
                    <TableHead>操作类型</TableHead>
                    <TableHead>时间</TableHead>
                    <TableHead>位置</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {records.map((record) => {
                    const badge = record.action_badge || {};
                    return (
                      <TableRow key={record.id}>
                        <TableCell className="font-medium font-mono">
                          {record.asset_serial_number}
                        </TableCell>
                        <TableCell>{record.asset_name}</TableCell>
                        <TableCell>{record.handler_name || "-"}</TableCell>
                        <TableCell>{record.department || "-"}</TableCell>
                        <TableCell>
                          <Badge
                            variant={badge.variant || "outline"}
                            className={cn(badge.class_name)}
                          >
                            {badge.label}
                          </Badge>
                        </TableCell>
                        <TableCell>{record.display_date}</TableCell>
                        <TableCell>{record.location_label}</TableCell>
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

export default Records;
