import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useApi } from "../../lib/ApiProvider";
import type { CreateLibraryExercise, Phase } from "../../api/types";
import { fullMesocycleKey } from "./useFullMesocycle";
import { mesocyclesKey } from "./useMesocycles";
import { usePlanErrorSink } from "./PlanErrors";

// Every plan edit changes some part of one mesocycle's grid, so each mutation
// refetches that grid on success — the simplest correct update. Optimistic
// patching can come later, where the latency actually shows.
function useGridMutation<TArgs>(mesocycleId: number, mutationFn: (args: TArgs) => Promise<unknown>) {
  const queryClient = useQueryClient();
  const errors = usePlanErrorSink();
  return useMutation({
    mutationFn,
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
  const api = useApi();
  return useGridMutation<void>(mesocycleId, () => api.addMicrocycle(mesocycleId));
}

export function useDeleteWeek(mesocycleId: number) {
  const api = useApi();
  return useGridMutation<number>(mesocycleId, (microcycleId) => api.deleteMicrocycle(microcycleId));
}

export function useSetPhase(mesocycleId: number) {
  const api = useApi();
  return useGridMutation<{ microcycleId: number; phase: Phase | null }>(
    mesocycleId,
    ({ microcycleId, phase }) => api.setMicrocyclePhase(microcycleId, phase),
  );
}

export function useAddWorkout(mesocycleId: number) {
  const api = useApi();
  return useGridMutation<string>(mesocycleId, (name) => api.addWorkout(mesocycleId, name));
}

export function useRenameWorkout(mesocycleId: number) {
  const api = useApi();
  return useGridMutation<{ workoutId: number; name: string }>(mesocycleId, ({ workoutId, name }) =>
    api.renameWorkout(workoutId, name),
  );
}

export function useDeleteWorkout(mesocycleId: number) {
  const api = useApi();
  return useGridMutation<number>(mesocycleId, (workoutId) => api.deleteWorkout(workoutId));
}

export function useAddPlannedExercise(mesocycleId: number) {
  const api = useApi();
  return useGridMutation<{ workoutId: number; libraryExerciseId: number }>(
    mesocycleId,
    ({ workoutId, libraryExerciseId }) => api.addPlannedExercise(workoutId, libraryExerciseId),
  );
}

export function useDeletePlannedExercise(mesocycleId: number) {
  const api = useApi();
  return useGridMutation<number>(mesocycleId, (plannedExerciseId) =>
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
