import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useApi } from "../../lib/ApiProvider";
import type { ApiClient } from "../../api/client";
import type { CreateLibraryExercise, Phase } from "../../api/types";
import { fullMesocycleKey } from "./useFullMesocycle";
import { mesocyclesKey } from "./useMesocycles";
import { usePlanErrorSink } from "./PlanErrors";

// Every plan edit changes some part of one mesocycle's grid, so each mutation
// refetches that grid on success — the simplest correct update. Optimistic
// patching can come later, where the latency actually shows.
//
// The client is handed to `request` rather than read by each caller, for the
// same reason the query client and the error sink are: this is where a grid
// mutation's surroundings live.
function useGridMutation<TArgs>(
  mesocycleId: number,
  request: (api: ApiClient, args: TArgs) => Promise<unknown>,
) {
  const api = useApi();
  const queryClient = useQueryClient();
  const errors = usePlanErrorSink();
  return useMutation({
    mutationFn: (args: TArgs) => request(api, args),
    onSuccess: () => errors.clear(),
    // Failure has to reach the user: these controls sit in table cells with no
    // room of their own to report, so they share the grid's error banner.
    onError: (error) => errors.report(error),
    // Refetch either way. A request that reached the server but lost its
    // response looks like a failure here, and leaving the grid on pre-edit
    // state would invite a retry that applies the edit twice.
    onSettled: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: fullMesocycleKey(mesocycleId) }),
        // The list renders a week count per mesocycle, which week edits change.
        queryClient.invalidateQueries({ queryKey: mesocyclesKey }),
      ]),
  });
}

export function useAddWeek(mesocycleId: number) {
  return useGridMutation<void>(mesocycleId, (api) => api.addMicrocycle(mesocycleId));
}

export function useDeleteWeek(mesocycleId: number) {
  return useGridMutation<number>(mesocycleId, (api, microcycleId) =>
    api.deleteMicrocycle(microcycleId),
  );
}

export function useSetPhase(mesocycleId: number) {
  return useGridMutation<{ microcycleId: number; phase: Phase | null }>(
    mesocycleId,
    (api, { microcycleId, phase }) => api.setMicrocyclePhase(microcycleId, phase),
  );
}

export function useAddWorkout(mesocycleId: number) {
  return useGridMutation<string>(mesocycleId, (api, name) => api.addWorkout(mesocycleId, name));
}

export function useRenameWorkout(mesocycleId: number) {
  return useGridMutation<{ workoutId: number; name: string }>(
    mesocycleId,
    (api, { workoutId, name }) => api.renameWorkout(workoutId, name),
  );
}

export function useDeleteWorkout(mesocycleId: number) {
  return useGridMutation<number>(mesocycleId, (api, workoutId) => api.deleteWorkout(workoutId));
}

export function useAddPlannedExercise(mesocycleId: number) {
  return useGridMutation<{ workoutId: number; libraryExerciseId: number }>(
    mesocycleId,
    (api, { workoutId, libraryExerciseId }) =>
      api.addPlannedExercise(workoutId, libraryExerciseId),
  );
}

export function useDeletePlannedExercise(mesocycleId: number) {
  return useGridMutation<number>(mesocycleId, (api, plannedExerciseId) =>
    api.deletePlannedExercise(plannedExerciseId),
  );
}

export const libraryExercisesKey = ["library-exercises"] as const;

export function useLibraryExercises() {
  const api = useApi();
  return useQuery({
    queryKey: libraryExercisesKey,
    queryFn: () => api.listLibraryExercises(),
  });
}

/** Adds to the library; the caller decides whether to place it in a workout. */
export function useCreateLibraryExercise() {
  const api = useApi();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateLibraryExercise) => api.createLibraryExercise(body),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: libraryExercisesKey }),
  });
}
