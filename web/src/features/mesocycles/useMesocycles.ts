import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../../lib/apiClient";
import type { CreateMesocycle } from "../../api/types";

export const mesocyclesKey = ["mesocycles"] as const;

export function useMesocycles() {
  return useQuery({
    queryKey: mesocyclesKey,
    queryFn: () => api.listMesocycles(),
  });
}

export function useCreateMesocycle() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateMesocycle) => api.createMesocycle(body),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: mesocyclesKey }),
  });
}
