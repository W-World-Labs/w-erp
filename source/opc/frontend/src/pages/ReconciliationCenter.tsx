import { useState } from "react";
import { Link } from "react-router-dom";
import { Card } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  AlertCircle,
  CheckCircle,
  Download,
  Landmark,
  Link as LinkIcon,
} from "lucide-react";
import { QueryStateGate } from "@/components/common/QueryStateGate";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const RECONCILIATION_OVERVIEW_ENDPOINT = "/api/v1/reconciliation/overview";

interface ReconciliationBadge {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
}

interface ReconciliationCell {
  key: string;
  value: string;
  class_name?: string;
  badge?: ReconciliationBadge;
}

interface ReconciliationAction {
  label: string;
  icon?: string;
  enabled: boolean;
}

interface ReconciliationRow {
  id: string;
  cells: ReconciliationCell[];
  action?: ReconciliationAction;
}

interface ReconciliationSection {
  key: string;
  title: string;
  export_label: string;
  headers: string[];
  rows: ReconciliationRow[];
  empty_message: string;
}

interface ReconciliationStat {
  key: string;
  label: string;
  value: number | string;
  class_name: string;
}

interface ReconciliationTab {
  value: string;
  label: string;
}

interface ReconciliationOverview {
  title: string;
  subtitle: string;
  tabs: ReconciliationTab[];
  stats: ReconciliationStat[];
  unmatched_sections: ReconciliationSection[];
  matched_section: ReconciliationSection;
  empty_messages: {
    unmatched: string;
    matched: string;
  };
}

const emptyOverview: ReconciliationOverview = {
  title: "财务对账中心",
  subtitle: "银行账单、支付记录、发票信息三方对账核验",
  tabs: [
    { value: "unmatched", label: "待匹配项目" },
    { value: "matched", label: "已匹配记录" },
  ],
  stats: [],
  unmatched_sections: [],
  matched_section: {
    key: "matched",
    title: "已匹配记录",
    export_label: "导出",
    headers: [],
    rows: [],
    empty_message: "暂无已匹配记录",
  },
  empty_messages: {
    unmatched: "暂无待匹配项目",
    matched: "暂无已匹配记录",
  },
};

function ReconciliationCellView({ cell }: { cell: ReconciliationCell }) {
  if (cell.badge) {
    return (
      <Badge
        variant={cell.badge.variant || "default"}
        className={cell.class_name}
      >
        {cell.key === "status" && <CheckCircle className="h-3 w-3 mr-1" />}
        {cell.badge.label}
      </Badge>
    );
  }

  return <span className={cell.class_name}>{cell.value}</span>;
}

function ReconciliationTable({
  section,
  fallback,
}: {
  section: ReconciliationSection;
  fallback: string;
}) {
  return (
    <Card className="p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-foreground">
          {section.title}
        </h2>
        <Button variant="outline" size="sm">
          <Download className="h-4 w-4 mr-2" />
          {section.export_label}
        </Button>
      </div>
      <Table>
        <TableHeader>
          <TableRow>
            {section.headers.map((header) => (
              <TableHead key={header}>{header}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {section.rows.length === 0 ? (
            <TableRow>
              <TableCell
                colSpan={Math.max(section.headers.length, 1)}
                className="text-center py-8 text-muted-foreground"
              >
                {section.empty_message || fallback}
              </TableCell>
            </TableRow>
          ) : (
            section.rows.map((row) => (
              <TableRow key={row.id}>
                {row.cells.map((cell) => (
                  <TableCell key={cell.key}>
                    <ReconciliationCellView cell={cell} />
                  </TableCell>
                ))}
                {row.action && (
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="sm"
                      disabled={!row.action.enabled}
                    >
                      <LinkIcon className="h-4 w-4 mr-1" />
                      {row.action.label}
                    </Button>
                  </TableCell>
                )}
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </Card>
  );
}

export default function ReconciliationCenter() {
  const [selectedTab, setSelectedTab] = useState("unmatched");
  const {
    data: overview = emptyOverview,
    isLoading,
    isError,
    refetch,
  } = useQuery({
    queryKey: ["reconciliation_overview"],
    queryFn: () =>
      api.get<ReconciliationOverview>(RECONCILIATION_OVERVIEW_ENDPOINT),
  });

  const loadingFallback = (
    <div className="container mx-auto p-6 space-y-6">
      <Skeleton className="h-12 w-64" />
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {[1, 2, 3, 4, 5, 6].map((item) => (
          <Skeleton key={item} className="h-32" />
        ))}
      </div>
    </div>
  );

  return (
    <QueryStateGate
      isLoading={isLoading}
      isError={isError}
      refetch={refetch}
      loadingFallback={loadingFallback}
    >
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-foreground">{overview.title}</h1>
          <p className="text-muted-foreground mt-2">{overview.subtitle}</p>
        </div>
        <Button variant="outline" asChild>
          <Link to="/bank-accounts">
            <Landmark className="h-4 w-4 mr-2" />
            银行账户
          </Link>
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4">
        {overview.stats.map((stat) => (
          <Card key={stat.key} className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-muted-foreground">{stat.label}</p>
                <p className="text-2xl font-bold text-foreground mt-1">
                  {stat.value}
                </p>
              </div>
              <div className={`p-3 rounded-full ${stat.class_name}`}>
                <AlertCircle className="h-6 w-6" />
              </div>
            </div>
          </Card>
        ))}
      </div>

      <Tabs value={selectedTab} onValueChange={setSelectedTab}>
        <TabsList>
          {overview.tabs.map((tab) => (
            <TabsTrigger key={tab.value} value={tab.value}>
              {tab.label}
            </TabsTrigger>
          ))}
        </TabsList>

        <TabsContent value="unmatched" className="space-y-6">
          {overview.unmatched_sections.length === 0 ? (
            <Card className="p-6 text-center text-muted-foreground">
              {overview.empty_messages.unmatched}
            </Card>
          ) : (
            overview.unmatched_sections.map((section) => (
              <ReconciliationTable
                key={section.key}
                section={section}
                fallback={overview.empty_messages.unmatched}
              />
            ))
          )}
        </TabsContent>

        <TabsContent value="matched">
          <ReconciliationTable
            section={overview.matched_section}
            fallback={overview.empty_messages.matched}
          />
        </TabsContent>
      </Tabs>
    </div>
    </QueryStateGate>
  );
}
