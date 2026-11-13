import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { TrendingUp, TrendingDown, AlertTriangle, AlertCircle } from "lucide-react";
import { cn } from "@/lib/utils";

interface KPIMetricCardProps {
  config: {
    id: string;
    name: string;
    current_value_label: string;
    previous_value_label: string;
    change_label: string;
    change_icon: string;
    change_class: string;
    has_previous_value: boolean;
    target_value_label?: string | null;
    alert_type: "normal" | "warning" | "danger";
    alert_badge: {
      label: string;
      class_name: string;
      icon?: string | null;
    };
  };
}

export const KPIMetricCard = ({ config }: KPIMetricCardProps) => {
  const hasAlert = config.alert_type !== "normal";
  const ChangeIcon = config.change_icon === "TrendingDown" ? TrendingDown : TrendingUp;
  const WarningIcon = config.alert_badge.icon === "AlertCircle" ? AlertCircle : AlertTriangle;

  return (
    <Card className={cn(
      "relative overflow-hidden transition-all hover:shadow-md",
      hasAlert && config.alert_badge.class_name
    )}>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium text-muted-foreground">
            {config.name}
          </CardTitle>
          {hasAlert && (
            <WarningIcon className={cn("h-4 w-4", config.alert_badge.class_name)} />
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <div className="flex items-baseline gap-2">
          <span className="text-3xl font-bold text-foreground">
            {config.current_value_label}
          </span>
        </div>

        {config.has_previous_value && (
          <div className="flex items-center gap-1 text-sm">
            <ChangeIcon className={cn("h-4 w-4", config.change_class)} />
            <span className={cn("font-medium", config.change_class)}>
              {config.change_label}
            </span>
            <span className="text-muted-foreground">vs 上期</span>
          </div>
        )}

        {config.target_value_label && (
          <div className="text-xs text-muted-foreground">
            {config.target_value_label}
          </div>
        )}

        {hasAlert && (
          <div className={cn("text-xs font-medium", config.alert_badge.class_name)}>
            {config.alert_badge.label}
          </div>
        )}
      </CardContent>
    </Card>
  );
};
