import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { MesocycleList } from "../features/mesocycles/MesocycleList";

export const Route = createFileRoute("/")({
  component: MesocycleListPage,
});

function MesocycleListPage() {
  const navigate = useNavigate();
  return (
    <MesocycleList onSelect={(id) => navigate({ to: "/mesocycles/$id", params: { id } })} />
  );
}
