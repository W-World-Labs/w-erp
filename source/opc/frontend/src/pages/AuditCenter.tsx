import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Brain, FileText, Users, DollarSign, Briefcase, Target, TrendingUp, Clock } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { AIAuditDialog } from "@/components/applications/AIAuditDialog";
import { API_ENDPOINTS } from "@/lib/api";

type AuditType = 'application' | 'material' | 'contract' | 'employee' | 'budget' | 'project';

type AuditQuickItem = {
  id: string;
  title: string;
  audit_type: AuditType;
  audit_title: string;
  audit_request: {
    audit_type: AuditType;
    target_id: string;
  };
};

type AuditModule = {
  audit_type: AuditType;
  icon_key: string;
  title: string;
  description: string;
  color_class: string;
  background_class: string;
  count: number;
  count_label: string;
  quick_title: string;
  quick_items: AuditQuickItem[];
  empty_message: string;
};

type AuditCenterOverview = {
  stats: {
    total_auditable_label: string;
    coverage_label: string;
    module_count_label: string;
    module_caption: string;
    analysis_label: string;
    analysis_caption: string;
  };
  modules: AuditModule[];
  capability_title: string;
  capability_description: string;
  capability_sections: Array<{ title: string; items: string[] }>;
};

const emptyOverview: AuditCenterOverview = {
  stats: {
    total_auditable_label: "0",
    coverage_label: "",
    module_count_label: "0",
    module_caption: "",
    analysis_label: "",
    analysis_caption: "",
  },
  modules: [],
  capability_title: "",
  capability_description: "",
  capability_sections: [],
};

const auditIconMap = {
  FileText,
  Briefcase,
  Users,
  DollarSign,
  Target,
  Brain,
};

const auditStyleClasses = {
  "text-primary": "text-primary",
  "text-accent": "text-accent",
  "text-secondary": "text-secondary",
  "text-warning": "text-warning",
  "text-success": "text-success",
  "bg-primary/10": "bg-primary/10",
  "bg-accent/10": "bg-accent/10",
  "bg-secondary/10": "bg-secondary/10",
  "bg-warning/10": "bg-warning/10",
  "bg-success/10": "bg-success/10",
};

function styleClass(value: string) {
  return auditStyleClasses[value as keyof typeof auditStyleClasses] || "";
}

export default function AuditCenter() {
  const [aiAuditOpen, setAiAuditOpen] = useState(false);
  const [auditConfig, setAuditConfig] = useState<{
    type: AuditType;
    title: string;
    request: AuditQuickItem["audit_request"];
  } | null>(null);

  const { data: overview = emptyOverview } = useQuery({
    queryKey: ["audit-center-overview"],
    queryFn: () => api.get<AuditCenterOverview>(API_ENDPOINTS.auditCenterOverview),
  });

  const handleQuickAudit = (item: AuditQuickItem) => {
    setAuditConfig({
      type: item.audit_type,
      title: item.audit_title,
      request: item.audit_request,
    });
    setAiAuditOpen(true);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Brain className="h-8 w-8 text-primary" />
            AI 审计中心
          </h1>
          <p className="text-muted-foreground mt-2">
            全方位智能审计，助力企业健康发展
          </p>
        </div>
      </div>

      {/* 审计概览 */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">可审计对象</CardTitle>
            <TrendingUp className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {overview.stats.total_auditable_label}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {overview.stats.coverage_label}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">审计模块</CardTitle>
            <Brain className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {overview.stats.module_count_label}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {overview.stats.module_caption}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">智能分析</CardTitle>
            <Clock className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {overview.stats.analysis_label}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {overview.stats.analysis_caption}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* 审计模块 */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {overview.modules.map((module) => {
          const Icon = auditIconMap[module.icon_key as keyof typeof auditIconMap] || Brain;
          const backgroundClass = styleClass(module.background_class);
          const colorClass = styleClass(module.color_class);
          return (
            <Card key={module.title} className="relative overflow-hidden">
              <div className={`absolute top-0 right-0 w-32 h-32 ${backgroundClass} rounded-full -mr-16 -mt-16 opacity-20`} />
              <CardHeader>
                <div className="flex items-center gap-3">
                  <div className={`p-3 rounded-lg ${backgroundClass}`}>
                    <Icon className={`h-6 w-6 ${colorClass}`} />
                  </div>
                  <div>
                    <CardTitle className="text-lg">{module.title}</CardTitle>
                    <CardDescription className="text-xs mt-1">
                      {module.description}
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex items-center justify-between text-sm">
                  <span className="text-muted-foreground">可审计对象</span>
                  <span className="font-bold">{module.count_label}</span>
                </div>
                
                {module.count > 0 ? (
                  <div className="space-y-2">
                    <p className="text-xs text-muted-foreground">
                      {module.quick_title}
                    </p>
                    {module.quick_items.map((item) => (
                      <Button
                        key={item.id}
                        variant="outline"
                        size="sm"
                        className="w-full justify-start text-left"
                        onClick={() => handleQuickAudit(item)}
                      >
                        <Brain className="h-3 w-3 mr-2 flex-shrink-0" />
                        <span className="truncate">
                          {item.title}
                        </span>
                      </Button>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-4 text-sm text-muted-foreground">
                    {module.empty_message}
                  </div>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* AI 审计功能说明 */}
      <Card>
        <CardHeader>
          <CardTitle>{overview.capability_title}</CardTitle>
          <CardDescription>{overview.capability_description}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2">
            {overview.capability_sections.map((section) => (
              <div key={section.title} className="space-y-2">
                <h4 className="font-semibold text-sm">{section.title}</h4>
                <ul className="text-sm text-muted-foreground space-y-1">
                  {section.items.map((item) => (
                    <li key={item}>• {item}</li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* AI 审计对话框 */}
      {auditConfig && (
        <AIAuditDialog
          open={aiAuditOpen}
          onOpenChange={setAiAuditOpen}
          auditType={auditConfig.type}
          auditRequest={auditConfig.request}
          title={auditConfig.title}
        />
      )}
    </div>
  );
}
