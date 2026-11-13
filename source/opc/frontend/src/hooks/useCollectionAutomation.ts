import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";

export type CollectionSettings = {
  enabled: boolean;
  friendly_days: number;
  formal_days: number;
  final_days: number;
  bad_debt_days: number;
  delivery_help?: string;
};

export const defaultCollectionSettings: CollectionSettings = {
  enabled: true,
  friendly_days: 1,
  formal_days: 7,
  final_days: 30,
  bad_debt_days: 60,
};

const endpoint = "/api/v1/settings/collections";

/** Backend-owned collection settings query and mutation boundary. */
export function useCollectionAutomation() {
  const queryClient = useQueryClient();
  const query = useQuery({
    queryKey: ["collection-settings"],
    queryFn: () => api.get<CollectionSettings>(endpoint),
  });
  const mutation = useMutation({
    mutationFn: (settings: CollectionSettings) =>
      api.put<CollectionSettings>(endpoint, settings),
    onSuccess: (saved) => {
      queryClient.setQueryData(["collection-settings"], saved);
    },
  });
  return {
    settings: query.data ?? defaultCollectionSettings,
    saveSettings: mutation.mutateAsync,
    isSaving: mutation.isPending,
  };
}
