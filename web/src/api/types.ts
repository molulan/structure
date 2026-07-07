// Hand-written mirror of structure-server's wire types. These are assumptions
// about the server's JSON contract; the contract tests (tests/contract) verify
// them against a real server, since typechecking cannot.

export type MesocycleMode = "Algorithmic" | "Manual";

/** A row from `GET /mesocycles` — includes the computed microcycle count. */
export interface MesocycleRow {
  id: number;
  name: string;
  mode: MesocycleMode;
  microcycle_count: number;
}

/** The bare mesocycle returned by `POST`/`PUT /mesocycles`. */
export interface Mesocycle {
  id: number;
  name: string;
  mode: MesocycleMode;
}

export interface CreateMesocycle {
  name: string;
  mode: MesocycleMode;
}
