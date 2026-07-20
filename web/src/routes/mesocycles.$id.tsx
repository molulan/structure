import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { MesocycleDetail } from "../features/mesocycles/MesocycleDetail";

export const Route = createFileRoute("/mesocycles/$id")({
  // Parse the raw string param into a typed number; a non-numeric id fails the
  // match (returns false) rather than reaching the component as NaN.
  params: {
    parse: ({ id }) => {
      const parsed = Number(id);
      return Number.isInteger(parsed) && parsed > 0 ? { id: parsed } : false;
    },
    stringify: ({ id }) => ({ id: String(id) }),
  },
  component: MesocycleDetailPage,
});

function MesocycleDetailPage() {
  const { id } = Route.useParams();
  const navigate = useNavigate();
  return <MesocycleDetail id={id} onBack={() => navigate({ to: "/" })} />;
}
