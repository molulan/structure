import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:structure/providers/selected_training_program_provider.dart';
import 'package:structure/providers/training_program_list_provider.dart';
import 'package:structure/screens/widgets/create_training_program_dialog.dart';
import 'package:structure/src/bridge/dto/planning.dart';
import 'package:structure/src/bridge/api/mesocycles.dart' as bridge_meso;
import 'package:structure/src/bridge/api/microcycles.dart' as bridge_micro;

class TrainingProgramsScreen extends ConsumerStatefulWidget {
  const TrainingProgramsScreen({super.key});

  @override
  ConsumerState<TrainingProgramsScreen> createState() =>
      _TrainingProgramsScreenState();
}

class _TrainingProgramsScreenState
    extends ConsumerState<TrainingProgramsScreen> {
  void _createTrainingProgram() async {
    final result = await showDialog<ProgramCreationResult>(
      context: context,
      builder: (context) => const CreateTrainingProgramDialog(),
    );

    if (result == null) return;

    MesocycleDTO? mesocycle;
    //bridge calls should have errorhandling in case rust returns some error
    try {
      mesocycle = bridge_meso.createMesocycle(
        name: result.name,
        mode: result.mode,
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Failed to create program: $e')));
      }
      return;
    }
    try {
      for (var i = 0; i < result.durationWeeks; i++) {
        bridge_micro.createMicrocycle(mesocycleId: mesocycle.id);
      }
    } catch (e) {
      ref.invalidate(trainingProgramListProvider);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Program partially created (weeks incomplete): $e'),
          ),
        );
      }
      return;
    }
    ref.invalidate(trainingProgramListProvider);

    if (mounted) {
      //TODO: navigate to builder screen
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Program created!')));
    }
  }

  @override
  Widget build(BuildContext context) {
    final programsAsync = ref.watch(trainingProgramListProvider);

    return Scaffold(
      body: programsAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, _) => Center(child: Text('Error: $err')),
        data: (programs) => Row(
          children: [
            SizedBox(
              width: 300,
              child: Column(
                children: [
                  Expanded(child: _TrainingProgramList(programs: programs)),
                ],
              ),
            ),
            const VerticalDivider(width: 1),
            Expanded(child: _TrainingProgramDetail(programs: programs)),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _createTrainingProgram,
        tooltip: 'Create New Program',
        child: const Icon(Icons.add),
      ),
    );
  }
}

class _TrainingProgramList extends ConsumerWidget {
  final List<MesocycleDTO> programs;
  const _TrainingProgramList({required this.programs});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selectedIndex = ref.watch(selectedTrainingProgramProvider);

    return ListView.separated(
      padding: const EdgeInsets.all(8),
      itemCount: programs.length,
      separatorBuilder: (_, _) => const SizedBox(height: 6),
      itemBuilder: (_, index) {
        final isSelected = (selectedIndex ?? 0) == index;
        final colorScheme = Theme.of(context).colorScheme;
        return Ink(
          decoration: BoxDecoration(
            color: isSelected
                ? colorScheme.secondaryContainer
                : colorScheme.surfaceContainerLow,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: isSelected
                  ? colorScheme.primary
                  : colorScheme.outlineVariant,
              width: 1,
            ),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withValues(alpha: isSelected ? 0.12 : 0.06),
                blurRadius: isSelected ? 4 : 2,
                offset: Offset(0, isSelected ? 2 : 1),
              ),
            ],
          ),
          child: InkWell(
            onTap: () => ref
                .read(selectedTrainingProgramProvider.notifier)
                .select(index),
            borderRadius: BorderRadius.circular(12),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    programs[index].name,
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                  const SizedBox(height: 4),
                  Row(
                    spacing: 6,
                    children: [
                      _ModeBadge(mode: programs[index].mode),
                      _WeekCountBadge(count: programs[index].microcycleCount),
                    ],
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}

class _ModeBadge extends StatelessWidget {
  final MesocycleModeDTO mode;
  const _ModeBadge({required this.mode});

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final label = switch (mode) {
      MesocycleModeDTO.algorithmic => 'Algorithmic',
      MesocycleModeDTO.manual => 'Manual',
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: colorScheme.primaryContainer,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(label, style: Theme.of(context).textTheme.labelSmall),
    );
  }
}

class _WeekCountBadge extends StatelessWidget {
  final int count;
  const _WeekCountBadge({required this.count});

  @override
    Widget build(BuildContext context) {
      final colorScheme = Theme.of(context).colorScheme;
      final label = '$count ${count == 1 ? 'week' : 'weeks'}';
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
        decoration: BoxDecoration(
          color: colorScheme.primaryContainer,
          borderRadius: BorderRadius.circular(4),
        ),
        child: Text(label, style: Theme.of(context).textTheme.labelSmall),
      );
    }
}

class _TrainingProgramDetail extends ConsumerWidget {
  final List<MesocycleDTO> programs;
  const _TrainingProgramDetail({required this.programs});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selectedIndex = ref.watch(selectedTrainingProgramProvider);
    if (programs.isEmpty) {
      return const Center(
        child: Text(
          'You have no training programs. Create one by pressing "+"',
        ),
      );
    }

    final effectiveIndex = (selectedIndex ?? 0).clamp(0, programs.length - 1);
    final program = programs[effectiveIndex];
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Text(program.name, style: const TextStyle(fontSize: 24)),
    );
  }
}
