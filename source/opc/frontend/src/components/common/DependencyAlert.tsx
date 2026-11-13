import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { AlertCircle } from "lucide-react";
import { useNavigate } from "react-router-dom";

export type DependencyType =
  | "company-info"
  | "employees"
  | "asset-categories"
  | "branches";

interface DependencyStatus {
  dependency: DependencyType;
  exists: boolean;
  count: number;
  message: string;
  action_text: string;
  navigate_to?: string | null;
}

interface DependencyAlertProps {
  dependency: DependencyType;
  onAction?: () => void;
  customMessage?: string;
  customActionText?: string;
}

export const DependencyAlert = ({
  dependency,
  onAction,
  customMessage,
  customActionText,
}: DependencyAlertProps) => {
  const navigate = useNavigate();

  const { data: dependencyStatus, isLoading } = useQuery({
    queryKey: [`dependency-check-${dependency}`],
    queryFn: async () => {
      return api.get<DependencyStatus>(`/api/dependencies/${dependency}`);
    },
  });

  // 不显示 alert 的情况
  if (isLoading || dependencyStatus?.exists) {
    return null;
  }

  const handleAction = () => {
    if (onAction) {
      onAction();
    } else if (dependencyStatus?.navigate_to) {
      navigate(dependencyStatus.navigate_to);
    }
  };

  return (
    <Alert>
      <AlertCircle className="h-4 w-4" />
      <AlertTitle>提示</AlertTitle>
      <AlertDescription className="flex items-center justify-between">
        <span>{customMessage || dependencyStatus?.message}</span>
        {(onAction || dependencyStatus?.navigate_to) && (
          <Button variant="outline" size="sm" onClick={handleAction}>
            {customActionText || dependencyStatus?.action_text}
          </Button>
        )}
      </AlertDescription>
    </Alert>
  );
};
