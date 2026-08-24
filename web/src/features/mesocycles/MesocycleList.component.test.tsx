import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithClient } from "../../test/renderWithClient";
import { ApiError } from "../../api/client";
import type { MesocycleRow } from "../../api/wire";
import { MesocycleList } from "./MesocycleList";

vi.mock("../../lib/apiClient", () => ({
  api: {
    listMesocycles: vi.fn(),
    createMesocycle: vi.fn(),
    getFullMesocycle: vi.fn(),
  },
}));
import { api } from "../../lib/apiClient";
const mockApi = vi.mocked(api);

const rows: MesocycleRow[] = [
  { id: 1, name: "Hypertrophy Block", mode: "Manual", microcycle_count: 4 },
  { id: 2, name: "Strength Block", mode: "Algorithmic", microcycle_count: 3 },
];

beforeEach(() => {
  vi.clearAllMocks();
});

describe("MesocycleList", () => {
  it("shows a loading state while the query is pending", () => {
    mockApi.listMesocycles.mockReturnValue(new Promise(() => {}));
    renderWithClient(<MesocycleList onSelect={() => {}} />);
    expect(screen.getByText("Loading…")).toBeInTheDocument();
  });

  it("shows an error state when the query rejects", async () => {
    mockApi.listMesocycles.mockRejectedValue(new ApiError(500, "boom"));
    renderWithClient(<MesocycleList onSelect={() => {}} />);
    expect(await screen.findByText("Could not load mesocycles.")).toBeInTheDocument();
  });

  it("shows the empty state when there are no mesocycles", async () => {
    mockApi.listMesocycles.mockResolvedValue([]);
    renderWithClient(<MesocycleList onSelect={() => {}} />);
    expect(
      await screen.findByText("No mesocycles yet. Create your first above."),
    ).toBeInTheDocument();
  });

  it("renders each mesocycle with its mode and week count", async () => {
    mockApi.listMesocycles.mockResolvedValue(rows);
    renderWithClient(<MesocycleList onSelect={() => {}} />);

    expect(await screen.findByText("Hypertrophy Block")).toBeInTheDocument();
    expect(screen.getByText("Manual · 4 weeks")).toBeInTheDocument();
    expect(screen.getByText("Strength Block")).toBeInTheDocument();
    expect(screen.getByText("Algorithmic · 3 weeks")).toBeInTheDocument();
  });

  it("calls onSelect with the mesocycle id when a row is clicked", async () => {
    mockApi.listMesocycles.mockResolvedValue(rows);
    const onSelect = vi.fn();
    renderWithClient(<MesocycleList onSelect={onSelect} />);

    await userEvent.click(await screen.findByRole("button", { name: /Strength Block/ }));
    expect(onSelect).toHaveBeenCalledWith(2);
  });

  it("submits a trimmed name and mode, then clears the input", async () => {
    mockApi.listMesocycles.mockResolvedValue([]);
    mockApi.createMesocycle.mockResolvedValue({ id: 3, name: "New Block", mode: "Algorithmic" });
    renderWithClient(<MesocycleList onSelect={() => {}} />);

    const input = await screen.findByLabelText("New mesocycle name");
    await userEvent.type(input, "  New Block  ");
    await userEvent.selectOptions(screen.getByLabelText("Mode"), "Algorithmic");
    await userEvent.click(screen.getByRole("button", { name: "Create" }));

    expect(mockApi.createMesocycle).toHaveBeenCalledWith({ name: "New Block", mode: "Algorithmic" });
    await waitFor(() => expect(input).toHaveValue(""));
  });

  it("does not submit when the name is blank", async () => {
    mockApi.listMesocycles.mockResolvedValue([]);
    renderWithClient(<MesocycleList onSelect={() => {}} />);

    await userEvent.type(await screen.findByLabelText("New mesocycle name"), "   ");
    await userEvent.click(screen.getByRole("button", { name: "Create" }));
    expect(mockApi.createMesocycle).not.toHaveBeenCalled();
  });

  it("surfaces a create failure without clearing the input", async () => {
    mockApi.listMesocycles.mockResolvedValue([]);
    mockApi.createMesocycle.mockRejectedValue(new ApiError(422, "name taken"));
    renderWithClient(<MesocycleList onSelect={() => {}} />);

    const input = await screen.findByLabelText("New mesocycle name");
    await userEvent.type(input, "Dupe");
    await userEvent.click(screen.getByRole("button", { name: "Create" }));

    expect(await screen.findByText("Could not create: name taken")).toBeInTheDocument();
    expect(input).toHaveValue("Dupe");
  });
});
