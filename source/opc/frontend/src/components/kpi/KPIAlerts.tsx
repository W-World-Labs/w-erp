import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { AlertTriangle, AlertCircle } from "lucide-react";
import type { LucideIcon } from "lucide-react";

export interface KpiAlert {
  type: 'warning' | 'danger';
  kpi: string;
  message: string;
  current_value?: number;
  threshold?: number;
  variant: 'default' | 'destructive';
  badge: {
    label: string;
    class_name: string;
    icon?: string | null;
  };
}

interface KPIAlertsProps {
  alerts: KpiAlert[];
}

const alertIcons: Record<string, LucideIcon> = {
  AlertCircle,
  AlertTriangle,
};

export const KPIAlerts = ({ alerts }: KPIAlertsProps) => {
  if (alerts.length === 0) return null;

  return (
    <div className="space-y-2">
      {alerts.map((alert, index) => {
        const AlertIcon = alertIcons[alert.badge.icon || ''] || AlertTriangle;
        return (
          <Alert key={`${alert.type}:${alert.kpi}:${index}`} variant={alert.variant}>
            <AlertIcon className="h-4 w-4" />
            <AlertTitle>{alert.kpi}</AlertTitle>
            <AlertDescription>{alert.message}</AlertDescription>
          </Alert>
        );
      })}
    </div>
  );
};
