import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { 
  Clock, 
  AlertTriangle, 
  CheckCircle2, 
  ArrowRight,
  FileText,
  DollarSign,
  Users,
  Building2,
  TrendingUp,
  Zap,
  Target,
  Bell,
  ShieldCheck,
  Sparkles,
  Wand2,
  Circle,
  type LucideIcon
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";
import { MagicImport } from "./MagicImport";
import { AIInsights } from "./AIInsights";
import type {
  UrgentItem,
  WorkbenchGuidanceView,
  WorkbenchOverviewCard,
  WorkbenchQuickAction,
  WorkbenchSummaryView,
} from "@/hooks/useSmartWorkbench";

interface SmartWorkbenchProps {
  urgentItems: UrgentItem[];
  totalUrgentItems: number;
  survivalItems: UrgentItem[];
  survivalTitle: string;
  survivalSubtitle: string;
  survivalCountLabel: string;
  guidance: WorkbenchGuidanceView;
  stats: WorkbenchSummaryView["stats"];
  operatingControl: WorkbenchSummaryView["operatingControl"];
  greeting: string;
  generatedDateLabel: string;
  todayTasks: number;
  headerBadges: WorkbenchSummaryView["headerBadges"];
  overviewCards: WorkbenchOverviewCard[];
  quickActions: WorkbenchQuickAction[];
}

const iconMap: Record<string, LucideIcon> = {
  FileText,
  DollarSign,
  Users,
  Building2,
  TrendingUp,
  Target,
  Bell,
};

function IconByName({ name, className }: { name: string; className?: string }) {
  const Icon = iconMap[name] || Bell;
  return <Icon className={className} />;
}

export function SmartWorkbench({
  urgentItems,
  totalUrgentItems,
  survivalItems,
  survivalTitle,
  survivalSubtitle,
  survivalCountLabel,
  guidance,
  stats,
  operatingControl,
  greeting,
  generatedDateLabel,
  todayTasks,
  headerBadges,
  overviewCards,
  quickActions,
}: SmartWorkbenchProps) {
  const navigate = useNavigate();
  const [magicImportOpen, setMagicImportOpen] = useState(false);

  return (
    <div className="space-y-6">
      {/* AI 智能洞察 - 最顶部 */}
      <AIInsights />

      {/* 欢迎横幅 */}
      <Card className="overflow-hidden border-0 bg-gradient-to-r from-primary/10 via-primary/5 to-transparent">
        <CardContent className="p-6">
          <div className="flex items-center justify-between">
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <Sparkles className="h-6 w-6 text-primary animate-pulse-soft" />
                <h1 className="text-2xl font-bold">{greeting}</h1>
              </div>
              <p className="text-muted-foreground">
                {generatedDateLabel}
              </p>
              <div className="flex items-center gap-4 mt-4">
                <div className="flex items-center gap-2 text-sm">
                  <Bell className="h-4 w-4 text-warning" />
                  <span>今日待办 <strong className="text-foreground">{todayTasks}</strong> 项</span>
                </div>
                <div className="flex items-center gap-2 text-sm">
                  <Clock className="h-4 w-4 text-primary" />
                  <span>待处理事项 <strong className="text-foreground">{totalUrgentItems}</strong> 项</span>
                </div>
              </div>
            </div>
            <div className="hidden md:flex items-center gap-3">
              {headerBadges.map((badge) => (
                <Badge key={badge.label} variant={badge.variant} className={badge.className}>
                  {badge.label}
                </Badge>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {guidance.visible && (
        <Card className="animate-fade-in border-primary/30" style={{ animationDelay: "0.03s" }}>
          <CardHeader className="pb-3">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <div className="space-y-1">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <Sparkles className="h-5 w-5 text-primary" />
                  {guidance.title}
                </CardTitle>
                <p className="text-sm text-muted-foreground">{guidance.subtitle}</p>
                {guidance.progressLabel && (
                  <p className="text-xs text-muted-foreground">{guidance.progressLabel}</p>
                )}
              </div>
              <Button onClick={() => navigate(guidance.nextRoute)}>
                {guidance.nextLabel}
                <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            </div>
          </CardHeader>
          {guidance.steps.length > 0 && (
            <CardContent className="grid gap-2 md:grid-cols-2">
              {guidance.steps.map((step) => (
                <button
                  key={step.id}
                  type="button"
                  className="group text-left rounded-lg border p-3 hover:bg-muted/50 transition-colors"
                  onClick={() => navigate(step.route)}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <div className="flex items-center gap-2">
                        {step.completed ? (
                          <CheckCircle2 className="h-4 w-4 text-accent" />
                        ) : (
                          <Circle className="h-4 w-4 text-muted-foreground" />
                        )}
                        <span className="font-medium">{step.label}</span>
                      </div>
                      <p className="mt-1 text-sm text-muted-foreground">{step.description}</p>
                    </div>
                  </div>
                </button>
              ))}
            </CardContent>
          )}
        </Card>
      )}

      {survivalItems.length > 0 && (
        <Card className="animate-fade-in border-destructive/40 bg-destructive/5" style={{ animationDelay: '0.04s' }}>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between gap-3">
              <CardTitle className="flex items-center gap-2 text-lg">
                <AlertTriangle className="h-5 w-5 text-destructive" />
                {survivalTitle}
              </CardTitle>
              <Badge variant="destructive">{survivalCountLabel}</Badge>
            </div>
            <p className="text-sm text-muted-foreground">{survivalSubtitle}</p>
          </CardHeader>
          <CardContent className="space-y-3">
            {survivalItems.map((item) => (
              <div
                key={item.id}
                className={cn(
                  "group flex items-center justify-between p-3 rounded-lg border border-destructive/20 bg-background",
                  "hover:bg-muted/50 hover:border-destructive/40 cursor-pointer"
                )}
                onClick={() => navigate(item.path)}
              >
                <div className="flex items-center gap-3">
                  <div className={cn(
                    "h-10 w-10 rounded-full flex items-center justify-center",
                    item.priorityBadge.color_class + "/10"
                  )}>
                    <span className={item.priorityBadge.class_name}>
                      <IconByName name={item.typeIcon} className="h-4 w-4" />
                    </span>
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{item.title}</span>
                      <Badge variant="outline" className={cn("text-xs", item.priorityBadge.class_name)}>
                        {item.priorityBadge.label}
                      </Badge>
                    </div>
                    <p className="text-sm text-muted-foreground">{item.description}</p>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  {item.daysLeftLabel && (
                    <span className="text-xs text-muted-foreground">{item.daysLeftLabel}</span>
                  )}
                  <Button size="sm" variant="ghost">
                    {item.action}
                    <ArrowRight className="h-3 w-3 ml-1" />
                  </Button>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      <Card className="animate-fade-in border-primary/20" style={{ animationDelay: '0.05s' }}>
        <CardContent className="p-5">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div className="flex items-start gap-3">
              <div className="h-11 w-11 rounded-lg bg-primary/10 text-primary flex items-center justify-center">
                <ShieldCheck className="h-5 w-5" />
              </div>
              <div className="space-y-1">
                <div className="flex flex-wrap items-center gap-2">
                  <h2 className="text-lg font-semibold">{operatingControl.title}</h2>
                  <Badge variant="outline">{operatingControl.statusLabel}</Badge>
                  <Badge variant="secondary">{operatingControl.backboneRateLabel}</Badge>
                </div>
                <p className="text-sm text-muted-foreground">{operatingControl.summaryLabel}</p>
                <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                  <span>{operatingControl.riskCountLabel}</span>
                  <span>{operatingControl.nextAction.description}</span>
                </div>
              </div>
            </div>
            <Button
              variant="outline"
              className="w-full lg:w-auto"
              onClick={() => navigate(operatingControl.nextAction.route)}
            >
              {operatingControl.nextAction.label}
              <ArrowRight className="h-4 w-4 ml-2" />
            </Button>
          </div>

          {operatingControl.controlItems.length > 0 && (
            <div className="mt-4 grid gap-3 md:grid-cols-2">
              {operatingControl.controlItems.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  className="group text-left rounded-lg border p-3 hover:bg-muted/50 transition-colors"
                  onClick={() => navigate(item.route)}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-medium">{item.title}</span>
                        <Badge variant="outline" className={item.badge.className}>
                          {item.badge.label}
                        </Badge>
                      </div>
                      <p className="mt-1 text-sm text-muted-foreground">{item.description}</p>
                    </div>
                    <span className="text-xs text-primary whitespace-nowrap">
                      {item.actionLabel}
                    </span>
                  </div>
                </button>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* 紧急事项 + 快捷操作 */}
      <div className="grid gap-6 lg:grid-cols-3">
        {/* 紧急事项 */}
        <Card className="lg:col-span-2 animate-fade-in" style={{ animationDelay: '0.1s' }}>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="flex items-center gap-2 text-lg">
                <AlertTriangle className="h-5 w-5 text-warning" />
                需要您关注
              </CardTitle>
              <Button variant="ghost" size="sm" onClick={() => navigate('/reminders')}>
                查看全部 <ArrowRight className="h-4 w-4 ml-1" />
              </Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            {urgentItems.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                <CheckCircle2 className="h-12 w-12 mb-3 text-accent" />
                <p className="font-medium">{guidance.emptyUrgentTitle}</p>
                <p className="text-sm">{guidance.emptyUrgentDescription}</p>
              </div>
            ) : (
              urgentItems.map((item, index) => (
                <div
                  key={item.id}
                  className={cn(
                    "group flex items-center justify-between p-3 rounded-lg border transition-all duration-200",
                    "hover:bg-muted/50 hover:border-primary/30 hover:shadow-sm cursor-pointer",
                    "animate-fade-in-up"
                  )}
                  style={{ animationDelay: `${index * 0.05}s` }}
                  onClick={() => navigate(item.path)}
                >
                  <div className="flex items-center gap-3">
                    <div className={cn(
                      "h-10 w-10 rounded-full flex items-center justify-center",
                      item.priorityBadge.color_class + "/10"
                    )}>
                      <span className={item.priorityBadge.class_name}>
                        <IconByName name={item.typeIcon} className="h-4 w-4" />
                      </span>
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{item.title}</span>
                        <Badge variant="outline" className={cn(
                          "text-xs",
                          item.priorityBadge.class_name
                        )}>
                          {item.priorityBadge.label}
                        </Badge>
                      </div>
                      <p className="text-sm text-muted-foreground">{item.description}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    {item.daysLeftLabel && (
                      <span className="text-xs text-muted-foreground">
                        {item.daysLeftLabel}
                      </span>
                    )}
                    <Button 
                      size="sm" 
                      variant="ghost"
                      className="opacity-0 group-hover:opacity-100 transition-opacity"
                    >
                      {item.action}
                      <ArrowRight className="h-3 w-3 ml-1" />
                    </Button>
                  </div>
                </div>
              ))
            )}
          </CardContent>
        </Card>

        {/* 快捷操作 */}
        <Card className="animate-fade-in" style={{ animationDelay: '0.2s' }}>
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2 text-lg">
              <Zap className="h-5 w-5 text-primary" />
              快捷操作
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {/* 魔法导入按钮 */}
            <Button
              variant="ghost"
              className={cn(
                "w-full justify-start h-auto py-3 px-3 group",
                "hover:bg-gradient-to-r hover:from-primary/10 hover:to-purple-500/10",
                "border border-dashed border-primary/30 hover:border-primary/50",
                "transition-all duration-200"
              )}
              onClick={() => setMagicImportOpen(true)}
            >
              <div className={cn(
                "h-9 w-9 rounded-lg bg-gradient-to-br from-primary to-purple-500 flex items-center justify-center mr-3",
                "text-white group-hover:scale-110 transition-transform"
              )}>
                <Wand2 className="h-5 w-5" />
              </div>
              <div className="text-left">
                <div className="font-medium flex items-center gap-2">
                  魔法导入
                  <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                    AI
                  </Badge>
                </div>
                <div className="text-xs text-muted-foreground">拖拽文件，智能识别导入</div>
              </div>
            </Button>
            
            {quickActions.map((action) => (
              <Button
                key={action.path}
                variant="ghost"
                className={cn(
                  "w-full justify-start h-auto py-3 px-3 group",
                  "hover:bg-muted/50 transition-all duration-200"
                )}
                onClick={() => navigate(action.path)}
              >
                <div className={cn(
                  "h-9 w-9 rounded-lg bg-gradient-to-br flex items-center justify-center mr-3",
                  action.color,
                  "text-white group-hover:scale-110 transition-transform"
                )}>
                  <IconByName name={action.icon} className="h-5 w-5" />
                </div>
                <div className="text-left">
                  <div className="font-medium">{action.label}</div>
                  <div className="text-xs text-muted-foreground">{action.description}</div>
                </div>
              </Button>
            ))}
          </CardContent>
        </Card>
      </div>

      {/* 魔法导入对话框 */}
      <MagicImport 
        open={magicImportOpen} 
        onOpenChange={setMagicImportOpen}
        onSuccess={() => window.location.reload()}
      />

      {/* 数据概览卡片 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {overviewCards.map((card, index) => (
          <StatCard
            key={card.key}
            card={card}
            delay={`${0.3 + index * 0.1}s`}
          />
        ))}
      </div>
    </div>
  );
}

interface StatCardProps {
  card: WorkbenchOverviewCard;
  delay?: string;
}

function StatCard({ card, delay }: StatCardProps) {
  return (
    <Card 
      className={cn(
        "overflow-hidden group hover:shadow-md transition-all duration-300 animate-fade-in-up",
        "hover:scale-[1.02]"
      )}
      style={{ animationDelay: delay }}
    >
      <CardContent className="p-5">
        <div className="flex items-center justify-between">
          <div className={cn(
            "h-11 w-11 rounded-xl bg-gradient-to-br flex items-center justify-center",
            card.className,
            "group-hover:scale-110 transition-transform duration-300"
          )}>
            <IconByName name={card.icon} className="h-5 w-5" />
          </div>
          {card.trendLabel && (
            <Badge variant={card.trendVariant || "secondary"} className="text-xs">
              {card.trendLabel}
            </Badge>
          )}
        </div>
        <div className="mt-4">
          <p className="text-sm text-muted-foreground">{card.title}</p>
          <div className="flex items-baseline gap-1 mt-1">
            <span className="text-2xl font-bold">{card.valueLabel}</span>
            {card.subtitle && <span className="text-sm text-muted-foreground">{card.subtitle}</span>}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
