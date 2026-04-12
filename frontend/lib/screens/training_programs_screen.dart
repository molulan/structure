import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:structure/providers/selected_training_program_provider.dart';
import 'package:structure/providers/training_program_list_provider.dart';
import 'package:structure/screens/widgets/create_training_program_dialog.dart';
import 'package:structure/src/bridge/lib.dart';
import 'package:structure/src/bridge/api.dart' as bridge;

class TrainingProgramsScreen extends ConsumerStatefulWidget {
  const TrainingProgramsScreen({super.key});

  @override
  ConsumerState<TrainingProgramsScreen> createState() =>
      _TrainingProgramsScreenState();
}

class _TrainingProgramsScreenState
    extends ConsumerState<TrainingProgramsScreen> {
  void _createTrainingProgram() async {
    final name = await showDialog(
      context: context,
      builder: (context) => const CreateTrainingProgramDialog(),
    );

    if (name != null && name.isNotEmpty) {
      //bridge calls should have errorhandling in case rust returns some error
      bridge.createMesocycle(name: name);
      ref.invalidate(trainingProgramListProvider);
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
  final List<Mesocycle> programs;
  const _TrainingProgramList({required this.programs});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selectedIndex = ref.watch(selectedTrainingProgramProvider);

    return ListView.separated(
      padding: const EdgeInsets.all(8),
      itemCount: programs.length,
      separatorBuilder: (_, _) => const Divider(),
      itemBuilder: (_, index) {
        final isSelected = (selectedIndex ?? 0) == index;
        return Card(
          elevation: isSelected ? 4 : 0,
          child: ListTile(
            title: Text(programs[index].name),
            onTap: () => ref
                .read(selectedTrainingProgramProvider.notifier)
                .select(index),
          ),
        );
      },
    );
  }
}

class _TrainingProgramDetail extends ConsumerWidget {
  final List<Mesocycle> programs;
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

    final effectiveIndex = (selectedIndex ?? 0).clamp(0, programs.length -1 );
    final program = programs[effectiveIndex];
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Text(program.name, style: const TextStyle(fontSize: 24)),
    );
  }
}
