import { useState } from "react";
import type { MesocycleMode } from "../../api/types";
import { ApiError } from "../../api/client";
import { useCreateMesocycle, useMesocycles } from "./useMesocycles";
import styles from "./MesocycleList.module.css";

interface Props {
  onSelect: (id: number) => void;
}

export function MesocycleList({ onSelect }: Props) {
  const mesocycles = useMesocycles();
  const create = useCreateMesocycle();
  const [name, setName] = useState("");
  const [mode, setMode] = useState<MesocycleMode>("Manual");

  function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    create.mutate({ name: trimmed, mode }, { onSuccess: () => setName("") });
  }

  return (
    <div className={styles.wrap}>
      <form className={styles.form} onSubmit={onSubmit}>
        <input
          className={styles.input}
          placeholder="New mesocycle name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          aria-label="New mesocycle name"
        />
        <select
          className={styles.select}
          value={mode}
          onChange={(e) => setMode(e.target.value as MesocycleMode)}
          aria-label="Mode"
        >
          <option value="Manual">Manual</option>
          <option value="Algorithmic">Algorithmic</option>
        </select>
        <button className={styles.button} type="submit" disabled={create.isPending}>
          {create.isPending ? "Creating…" : "Create"}
        </button>
      </form>

      {create.error instanceof ApiError && (
        <p className={styles.error}>Could not create: {create.error.message}</p>
      )}

      {mesocycles.isPending ? (
        <p className={styles.status}>Loading…</p>
      ) : mesocycles.isError ? (
        <p className={styles.error}>Could not load mesocycles.</p>
      ) : mesocycles.data.length === 0 ? (
        <p className={styles.empty}>No mesocycles yet. Create your first above.</p>
      ) : (
        <ul className={styles.list}>
          {mesocycles.data.map((m) => (
            <li key={m.id}>
              <button className={styles.item} onClick={() => onSelect(m.id)}>
                <span className={styles.name}>{m.name}</span>
                <span className={styles.meta}>
                  {m.mode} · {m.microcycle_count} weeks
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
