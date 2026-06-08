import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:structure/providers/mesocycle_grid_provider.dart';
import 'package:structure/src/bridge/dto/planning.dart';
import 'package:structure/src/bridge/api/workouts.dart' as bridge_workout;

// Layout constants for the frame. Cells are content-sized in height; only
// the column *width* and the label/summary *widths* are fixed.
const double _kRowLabelWidth = 120;
const double _kSummaryWidth = 220;
const double _kColumnWidth = 260;
const double _kHeaderHeight = 48;

class TrainingProgramBuilderScreen extends ConsumerStatefulWidget {
  final MesocycleDTO mesocycle;

  const TrainingProgramBuilderScreen({super.key, required this.mesocycle});

  @override
  ConsumerState<TrainingProgramBuilderScreen> createState() =>
      _TrainingProgramBuilderScreenState();
}

class _TrainingProgramBuilderScreenState
    extends ConsumerState<TrainingProgramBuilderScreen> {
  // VERTICAL: a single scrollable drives the whole body. The left labels,
  // body cells, and right summary for a given week are siblings in ONE
  // IntrinsicHeight row, so they can never drift or clamp each other.
  final ScrollController _vCtrl = ScrollController();

  // HORIZONTAL: two scrollables (pinned top header band + body columns) kept
  // in sync. Safe to sync because both have identical horizontal extent
  // (columnCount * _kColumnWidth), so there is no maxScrollExtent mismatch.
  final ScrollController _headerHCtrl = ScrollController();
  final ScrollController _bodyHCtrl = ScrollController();

  // Guards against the two horizontal listeners ping-ponging updates.
  bool _syncingH = false;

  @override
  void initState() {
    super.initState();
    _headerHCtrl.addListener(_syncFromHeader);
    _bodyHCtrl.addListener(_syncFromBody);
  }

  void _syncFromHeader() => _mirror(_headerHCtrl, _bodyHCtrl);
  void _syncFromBody() => _mirror(_bodyHCtrl, _headerHCtrl);

  void _mirror(ScrollController source, ScrollController target) {
    if (_syncingH) return;
    if (!target.hasClients) return;
    if (target.offset == source.offset) return;
    _syncingH = true;
    target.jumpTo(source.offset);
    _syncingH = false;
  }

  @override
  void dispose() {
    _headerHCtrl.removeListener(_syncFromHeader);
    _bodyHCtrl.removeListener(_syncFromBody);
    _headerHCtrl.dispose();
    _bodyHCtrl.dispose();
    _vCtrl.dispose();
    super.dispose();
  }

  Future<void> _promptAddWorkout(MesocycleGrid grid) async {
    if (grid.rows.isEmpty) {
      // Nothing to add a column to — every workout must span all microcycles.
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Add microcycles before adding workouts.'),
        ),
      );
      return;
    }

    var draft = '';
    final name = await showDialog<String>(
      context: context,
      builder: (dialogContext) {
        return AlertDialog(
          title: const Text('Add workout'),
          content: TextField(
            autofocus: true,
            decoration: const InputDecoration(
              labelText: 'Workout name',
              hintText: 'e.g. Push, Pull, Legs',
            ),
            onChanged: (value) => draft = value,
            onSubmitted: (value) =>
                Navigator.of(dialogContext).pop(value.trim()),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(dialogContext).pop(),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(dialogContext).pop(draft.trim()),
              child: const Text('Add'),
            ),
          ],
        );
      },
    );

    if (name == null || name.isEmpty) return;

    try {
      // A workout column spans every microcycle at the same position, so we
      // create one workout per microcycle. createWorkout appends, which keeps
      // positions aligned across rows as long as every row had equal counts.
      for (final row in grid.rows) {
        bridge_workout.createWorkout(
          microcycleId: row.microcycle.id,
          name: name,
        );
      }
    } catch (err) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Failed to add workout: $err')));
      return;
    }

    ref.invalidate(mesocycleGridProvider(widget.mesocycle.id));
  }

  @override
  Widget build(BuildContext context) {
    final gridAsync = ref.watch(mesocycleGridProvider(widget.mesocycle.id));

    return Scaffold(
      appBar: AppBar(
        title: Text(widget.mesocycle.name),
        actions: [
          // Always available. The handler itself guards the no-microcycle case.
          gridAsync.maybeWhen(
            data: (grid) => TextButton.icon(
              onPressed: () => _promptAddWorkout(grid),
              icon: const Icon(Icons.add),
              label: const Text('Workout'),
            ),
            orElse: () => const SizedBox.shrink(),
          ),
        ],
      ),
      body: gridAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, _) => Center(child: Text('Failed to load plan: $err')),
        data: (grid) => _buildGrid(context, grid),
      ),
    );
  }

  Widget _buildGrid(BuildContext context, MesocycleGrid grid) {
    if (grid.rows.isEmpty) {
      return const Center(child: Text('This program has no microcycles.'));
    }

    final columnCount = grid.columnCount;

    // Empty-column state: microcycles exist but no workouts yet. The grid is
    // untestable while empty, so surface an obvious affordance here too.
    if (columnCount == 0) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              'No workouts yet',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              'A workout spans every week at the same position.',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: () => _promptAddWorkout(grid),
              icon: const Icon(Icons.add),
              label: const Text('Add your first workout'),
            ),
          ],
        ),
      );
    }

    final headerWorkouts = grid.rows.first.cells.map((c) => c.workout).toList();

    return Column(
      children: [
        // ---- Pinned top band: TL corner | h-scrolling headers | TR summary ----
        // Outside the vertical scrollable, so it stays pinned as rows scroll.
        Row(
          children: [
            _CornerCell(
              width: _kRowLabelWidth,
              height: _kHeaderHeight,
              child: const SizedBox.shrink(),
            ),
            Expanded(
              child: SingleChildScrollView(
                controller: _headerHCtrl,
                scrollDirection: Axis.horizontal,
                physics: const NeverScrollableScrollPhysics(),
                child: Row(
                  children: [
                    for (final w in headerWorkouts)
                      _HeaderCell(width: _kColumnWidth, label: w.name),
                  ],
                ),
              ),
            ),
            _CornerCell(
              width: _kSummaryWidth,
              height: _kHeaderHeight,
              child: Text(
                'Volume',
                style: Theme.of(context).textTheme.labelLarge,
              ),
            ),
          ],
        ),
        const Divider(height: 1),

        // ---- THE single vertical scrollable: one row per week ----
        // Each week is an IntrinsicHeight Row whose children are the label,
        // the (horizontally scrolling) body cells, and the summary. Because
        // they are siblings in one row, they share height exactly and there
        // is exactly one vertical maxScrollExtent — the last week is always
        // reachable.
        Expanded(
          child: SingleChildScrollView(
            controller: _vCtrl,
            scrollDirection: Axis.vertical,
            child: Column(
              children: [
                for (var r = 0; r < grid.rows.length; r++)
                  _WeekRow(
                    weekNumber: r + 1,
                    cells: grid.rows[r].cells,
                    columnCount: columnCount,
                    bodyHCtrl: _bodyHCtrl,
                  ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// A single week: pinned label | horizontally-scrolling body | pinned summary.
// All three are siblings in one IntrinsicHeight row so their heights match.
// ---------------------------------------------------------------------------

class _WeekRow extends StatelessWidget {
  final int weekNumber;
  final List<MesocycleGridCell> cells;
  final int columnCount;
  final ScrollController bodyHCtrl;

  const _WeekRow({
    required this.weekNumber,
    required this.cells,
    required this.columnCount,
    required this.bodyHCtrl,
  });

  @override
  Widget build(BuildContext context) {
    return IntrinsicHeight(
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _RowLabelCell(weekNumber: weekNumber),
          const VerticalDivider(width: 1),
          // Body cells scroll horizontally, synced to the top header band.
          Expanded(
            child: SingleChildScrollView(
              controller: bodyHCtrl,
              scrollDirection: Axis.horizontal,
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  for (var c = 0; c < columnCount; c++)
                    _BodyCell(width: _kColumnWidth, cell: cells[c]),
                ],
              ),
            ),
          ),
          const VerticalDivider(width: 1),
          const _SummaryPlaceholderCell(),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Cell widgets — all const-constructible, all StatelessWidget.
// ---------------------------------------------------------------------------

class _CornerCell extends StatelessWidget {
  final double width;
  final double height;
  final Widget child;
  const _CornerCell({
    required this.width,
    required this.height,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
      height: height,
      alignment: Alignment.center,
      color: Theme.of(context).colorScheme.surfaceContainerHigh,
      child: child,
    );
  }
}

class _HeaderCell extends StatelessWidget {
  final double width;
  final String label;
  const _HeaderCell({required this.width, required this.label});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
      height: _kHeaderHeight,
      alignment: Alignment.centerLeft,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      color: Theme.of(context).colorScheme.surfaceContainerHigh,
      child: Text(
        label,
        style: Theme.of(context).textTheme.titleSmall,
        overflow: TextOverflow.ellipsis,
      ),
    );
  }
}

class _RowLabelCell extends StatelessWidget {
  final int weekNumber;
  const _RowLabelCell({required this.weekNumber});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: _kRowLabelWidth,
      constraints: const BoxConstraints(minHeight: 96),
      alignment: Alignment.topLeft,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Theme.of(context).dividerColor),
        ),
      ),
      child: Text(
        'Week $weekNumber',
        style: Theme.of(context).textTheme.titleSmall,
      ),
    );
  }
}

class _BodyCell extends StatelessWidget {
  final double width;
  final MesocycleGridCell cell;
  const _BodyCell({required this.width, required this.cell});

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    return Container(
      width: width,
      constraints: const BoxConstraints(minHeight: 96),
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        border: Border(
          right: BorderSide(color: colorScheme.outlineVariant),
          bottom: BorderSide(color: colorScheme.outlineVariant),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Step 1: placeholder content only. Real exercise/set rows arrive
          // in Step 2; add-exercise/add-set actions in Step 3.
          if (cell.plannedExercises.isEmpty)
            Text(
              'No exercises',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: colorScheme.onSurfaceVariant,
              ),
            )
          else
            Text(
              '${cell.plannedExercises.length} exercise(s)',
              style: Theme.of(context).textTheme.bodySmall,
            ),
        ],
      ),
    );
  }
}

class _SummaryPlaceholderCell extends StatelessWidget {
  const _SummaryPlaceholderCell();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: _kSummaryWidth,
      constraints: const BoxConstraints(minHeight: 96),
      alignment: Alignment.topLeft,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Theme.of(context).dividerColor),
        ),
      ),
      child: Text(
        '—',
        style: Theme.of(context).textTheme.bodySmall?.copyWith(
          color: Theme.of(context).colorScheme.onSurfaceVariant,
        ),
      ),
    );
  }
}
