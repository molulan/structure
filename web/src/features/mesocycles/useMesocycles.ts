import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useApi } from "../../lib/ApiProvider";
import type { CreateMesocycleRequest } from "../../api/generated/CreateMesocycleRequest";

export const mesocyclesKey = ["mesocycles"] as const;

export function useMesocycles() {
  const api = useApi();
  return useQuery({
    queryKey: mesocyclesKey,
    queryFn: () => api.listMesocycles(),
  });
}

export function useCreateMesocycle() {
  const api = useApi();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateMesocycleRequest) => api.createMesocycle(body),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: mesocyclesKey }),
  });
}
