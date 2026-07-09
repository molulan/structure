import { useQuery } from "@tanstack/react-query";
import { api } from "../../lib/apiClient";

export function useFullMesocycle(id: number) {
  return useQuery({
    queryKey: ["mesocycle", id, "full"],
    queryFn: () => api.getFullMesocycle(id),
  });
}
