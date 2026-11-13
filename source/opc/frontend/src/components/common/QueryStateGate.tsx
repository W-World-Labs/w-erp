import type { ReactNode } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";

export interface QueryStateGateProps {
  isLoading?: boolean;
  isError?: boolean;
  /** Called when the user clicks Retry. Accepts optional query-refetch signature. */
  refetch?: (...args: unknown[]) => unknown;
  loadingFallback?: ReactNode;
  errorTitle?: string;
  errorDescription?: string;
  children: ReactNode;
}

/**
 * Shared loading / error gate so failed queries never fall through to empty states.
 * Error branch is evaluated before loading and children.
 */
export function QueryStateGate({
  isLoading = false,
  isError = false,
  refetch,
  loadingFallback,
  errorTitle = "数据加载失败",
  errorDescription = "请检查网络后重试",
  children,
}: QueryStateGateProps) {
  if (isError) {
    return (
      <div
        className="flex min-h-[320px] items-center justify-center p-6"
        data-testid="query-state-error"
      >
        <Card className="w-full max-w-md">
          <CardContent className="flex flex-col items-center justify-center py-12 text-center">
            <AlertTriangle
              className="mb-4 h-16 w-16 text-destructive"
              aria-hidden
            />
            <h3 className="mb-2 text-lg font-semibold">{errorTitle}</h3>
            <p className="mb-6 max-w-md text-muted-foreground">
              {errorDescription}
            </p>
            {refetch && (
              <Button
                type="button"
                onClick={() => {
                  void refetch();
                }}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                重试
              </Button>
            )}
          </CardContent>
        </Card>
      </div>
    );
  }

  if (isLoading) {
    return (
      <>
        {loadingFallback ?? (
          <div
            className="flex h-full min-h-[200px] items-center justify-center p-6 text-muted-foreground"
            data-testid="query-state-loading"
          >
            加载中...
          </div>
        )}
      </>
    );
  }

  return <>{children}</>;
}

export default QueryStateGate;
