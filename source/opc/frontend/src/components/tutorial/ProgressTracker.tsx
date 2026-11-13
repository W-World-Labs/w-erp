import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { api } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  type LucideIcon,
  CheckCircle2,
  Circle,
  Building2,
  Users,
  FileText,
  DollarSign,
  FolderKanban,
  Zap,
  TrendingUp,
  ArrowRight,
  Home,
  FileSpreadsheet,
  Clock,
  Wallet,
  Package,
} from "lucide-react";
import { cn } from "@/lib/utils";

type DisplayBadge = {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  class_name?: string;
};

interface OnboardingModule {
  id: string;
  name: string;
  icon_key: string;
  route: string;
  completed: boolean;
  count: number;
  count_label: string;
  description: string;
  status_badge: DisplayBadge;
  tier: number;
  section_title: string;
}

interface OnboardingProgress {
  summary: {
    progress_rate: number;
    progress_rate_label: string;
    completed_label: string;
    total_records_label: string;
    all_completed: boolean;
  };
  modules: OnboardingModule[];
  tutorial_steps: unknown[];
  incomplete_tip: {
    title: string;
    description: string;
  };
  completed_tip: {
    title: string;
    description: string;
  };
}

const emptyProgress: OnboardingProgress = {
  summary: {
    progress_rate: 0,
    progress_rate_label: "0%",
    completed_label: "已完成 0/0 个模块",
    total_records_label: "共录入 0 条数据",
    all_completed: false,
  },
  modules: [],
  tutorial_steps: [],
  incomplete_tip: {
    title: "",
    description: "",
  },
  completed_tip: {
    title: "",
    description: "",
  },
};

const iconMap: Record<string, LucideIcon> = {
  Building2,
  Users,
  FileText,
  DollarSign,
  FolderKanban,
  Zap,
  TrendingUp,
  Home,
  FileSpreadsheet,
  Clock,
  Wallet,
  Package,
};

function BackendBadge({ badge }: { badge?: DisplayBadge | null }) {
  if (!badge) return null;
  return (
    <Badge variant={badge.variant || "outline"} className={badge.class_name || ""}>
      {badge.label}
    </Badge>
  );
}

function ModuleIcon({
  iconKey,
  className,
}: {
  iconKey: string;
  className: string;
}) {
  const Icon = iconMap[iconKey] || Building2;
  return <Icon className={className} />;
}

export function ProgressTracker() {
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);

  const { data } = useQuery({
    queryKey: ["onboarding-progress"],
    queryFn: () => api.get<OnboardingProgress>(API_ENDPOINTS.onboardingProgress),
  });

  const progress = data || emptyProgress;

  const handleModuleClick = (route: string) => {
    navigate(route);
    setOpen(false);
  };

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className="relative gap-2 border-primary/20 hover:border-primary/40"
        >
          <Zap className="h-4 w-4 text-primary" />
          <span className="hidden sm:inline">信息完成度</span>
          <Badge
            variant={progress.summary.all_completed ? "default" : "secondary"}
            className={cn(
              "ml-1",
              progress.summary.all_completed &&
                "bg-accent text-accent-foreground hover:bg-accent",
            )}
          >
            {progress.summary.progress_rate_label}
          </Badge>
          {!progress.summary.all_completed && (
            <span className="absolute -top-1 -right-1 flex h-3 w-3">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span>
              <span className="relative inline-flex rounded-full h-3 w-3 bg-primary"></span>
            </span>
          )}
        </Button>
      </SheetTrigger>
      <SheetContent className="w-full sm:max-w-md overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            <Zap className="h-5 w-5 text-primary" />
            信息完成度追踪
          </SheetTitle>
          <SheetDescription>
            查看各模块数据录入进度，快速导航到需要完善的功能
          </SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="space-y-3 p-4 rounded-lg bg-muted border border-border">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-semibold text-lg text-foreground">总体进度</h3>
                <p className="text-sm text-muted-foreground">
                  {progress.summary.completed_label}
                </p>
              </div>
              <div className="text-right">
                <div className="text-2xl font-bold text-primary">
                  {progress.summary.progress_rate_label}
                </div>
                <p className="text-xs text-muted-foreground">完成率</p>
              </div>
            </div>
            <Progress value={progress.summary.progress_rate} className="h-2" />
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">
                {progress.summary.total_records_label}
              </span>
              {progress.summary.all_completed && (
                <Badge className="bg-accent text-accent-foreground">
                  <CheckCircle2 className="h-3 w-3 mr-1" />
                  全部完成
                </Badge>
              )}
            </div>
          </div>

          <Separator />

          <div className="space-y-2">
            <h3 className="font-semibold text-sm text-muted-foreground mb-3">
              各模块详情
            </h3>
            <div className="space-y-2">
              {progress.modules.map((module) => (
                <div key={module.id}>
                  {module.section_title && (
                    <div className="text-xs font-semibold text-muted-foreground/80 uppercase tracking-wide mt-4 mb-2 first:mt-0">
                      {module.section_title}
                    </div>
                  )}
                  <button
                    onClick={() => handleModuleClick(module.route)}
                    className={cn(
                      "w-full flex items-center gap-3 p-3 rounded-lg border-2 transition-all text-left group hover:shadow-md",
                      module.completed
                        ? "border-accent/50 bg-accent/10 hover:bg-accent/20"
                        : "border-border hover:border-primary/40 hover:bg-muted",
                    )}
                  >
                    <div
                      className={cn(
                        "rounded-full p-2 transition-colors",
                        module.completed
                          ? "bg-accent text-accent-foreground"
                          : "bg-primary/10 text-primary group-hover:bg-primary/20",
                      )}
                    >
                      <ModuleIcon iconKey={module.icon_key} className="h-4 w-4" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <span className="font-medium text-foreground">{module.name}</span>
                        {module.completed ? (
                          <CheckCircle2 className="h-4 w-4 text-accent flex-shrink-0" />
                        ) : (
                          <Circle className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                        )}
                        <BackendBadge badge={module.status_badge} />
                      </div>
                      <p className="text-xs text-muted-foreground truncate">
                        {module.description}
                      </p>
                      {module.count > 0 && (
                        <Badge variant="secondary" className="mt-1 text-xs">
                          {module.count_label}
                        </Badge>
                      )}
                    </div>
                    <ArrowRight className="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors flex-shrink-0" />
                  </button>
                </div>
              ))}
            </div>
          </div>

          {!progress.summary.all_completed && progress.incomplete_tip.title && (
            <>
              <Separator />
              <div className="p-4 rounded-lg bg-primary/10 border border-primary/20">
                <div className="flex gap-3">
                  <Zap className="h-5 w-5 text-primary flex-shrink-0 mt-0.5" />
                  <div className="space-y-1">
                    <h4 className="font-semibold text-foreground text-sm">
                      {progress.incomplete_tip.title}
                    </h4>
                    <p className="text-xs text-muted-foreground">
                      {progress.incomplete_tip.description}
                    </p>
                  </div>
                </div>
              </div>
            </>
          )}

          {progress.summary.all_completed && progress.completed_tip.title && (
            <>
              <Separator />
              <div className="p-4 rounded-lg bg-accent/10 border border-accent/20">
                <div className="flex gap-3">
                  <CheckCircle2 className="h-5 w-5 text-accent flex-shrink-0 mt-0.5" />
                  <div className="space-y-1">
                    <h4 className="font-semibold text-foreground text-sm">
                      {progress.completed_tip.title}
                    </h4>
                    <p className="text-xs text-muted-foreground">
                      {progress.completed_tip.description}
                    </p>
                  </div>
                </div>
              </div>
            </>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
