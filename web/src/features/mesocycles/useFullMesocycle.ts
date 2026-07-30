import { useQuery } from "@tanstack/react-query";
import { api } from "../../lib/apiClient";

export function fullMesocycleKey(id: number) {
  return ["mesocycle", id, "full"] as const;
}

export function useFullMesocycle(id: number) {
  return useQuery({
    queryKey: fullMesocycleKey(id),
    queryFn: () => api.getFullMesocycle(id),
  });
}
