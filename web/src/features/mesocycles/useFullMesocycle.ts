import { useQuery } from "@tanstack/react-query";
import { useApi } from "../../lib/ApiProvider";

export function fullMesocycleKey(id: number) {
  return ["mesocycle", id, "full"] as const;
}

export function useFullMesocycle(id: number) {
  const api = useApi();
  return useQuery({
    queryKey: fullMesocycleKey(id),
    queryFn: () => api.getFullMesocycle(id),
  });
}
