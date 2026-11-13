import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api/contracts";

export type MonthlyCloseFile = {
  id: string;
  period: string;
  original_name: string;
  url: string;
  created_at?: string | null;
};

export type MonthlyCloseResult = {
  period: string;
  empty: boolean;
  income_label: string;
  expense_label: string;
  net_label: string;
  file: MonthlyCloseFile;
};

const endpoint = API_ENDPOINTS.monthlyClose;

/** Backend-owned monthly-close list and generate mutation. */
export function useMonthlyClose() {
  const queryClient = useQueryClient();
  const history = useQuery({
    queryKey: ["monthly-close-reports"],
    queryFn: () => api.get<{ data: MonthlyCloseFile[] }>(endpoint),
  });
  const generate = useMutation({
    mutationFn: (period: string) =>
      api.post<MonthlyCloseResult>(`${endpoint}?period=${period}`, {}),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["monthly-close-reports"] });
    },
  });
  return {
    reports: history.data?.data ?? [],
    isLoadingReports: history.isLoading,
    generateClose: generate.mutateAsync,
    isGenerating: generate.isPending,
  };
}
