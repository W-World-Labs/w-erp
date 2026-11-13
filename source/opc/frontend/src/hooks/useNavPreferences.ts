import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api/contracts";
import {
  applyNavPreferences,
  emptyNavPreferences,
  sanitizeNavPreferences,
  type NavPreferences,
} from "@/lib/nav-preferences";
import { useAuth } from "@/contexts/AuthContext";

export const NAV_PREFERENCES_QUERY_KEY = ["nav-preferences"] as const;

/** Backend-owned per-user sidebar show/hide/order. */
export function useNavPreferences() {
  const { isAdmin } = useAuth();
  const queryClient = useQueryClient();
  const query = useQuery({
    queryKey: NAV_PREFERENCES_QUERY_KEY,
    queryFn: async () => {
      const raw = await api.get<unknown>(API_ENDPOINTS.navPreferences);
      return sanitizeNavPreferences(raw);
    },
  });
  const mutation = useMutation({
    mutationFn: (prefs: NavPreferences) =>
      api.put<unknown>(API_ENDPOINTS.navPreferences, sanitizeNavPreferences(prefs)),
    onMutate: async (prefs) => {
      await queryClient.cancelQueries({ queryKey: NAV_PREFERENCES_QUERY_KEY });
      const previous = queryClient.getQueryData<NavPreferences>(
        NAV_PREFERENCES_QUERY_KEY,
      );
      queryClient.setQueryData(
        NAV_PREFERENCES_QUERY_KEY,
        sanitizeNavPreferences(prefs),
      );
      return { previous };
    },
    onError: (_error, _prefs, context) => {
      if (context?.previous) {
        queryClient.setQueryData(NAV_PREFERENCES_QUERY_KEY, context.previous);
      }
    },
    onSuccess: (saved) => {
      queryClient.setQueryData(
        NAV_PREFERENCES_QUERY_KEY,
        sanitizeNavPreferences(saved),
      );
    },
  });
  const preferences = query.data ?? emptyNavPreferences();
  return {
    preferences,
    categories: applyNavPreferences(preferences, { isAdmin }),
    savePreferences: mutation.mutateAsync,
    isSaving: mutation.isPending,
    isLoading: query.isLoading,
  };
}
