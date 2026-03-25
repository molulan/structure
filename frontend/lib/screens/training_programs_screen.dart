import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:structure/models/mock_training_program.dart';
import 'package:structure/providers/selected_training_program_provider.dart';
import 'package:structure/providers/training_program_list_provider.dart';

class TrainingProgramsScreen extends ConsumerStatefulWidget {
  const TrainingProgramsScreen({super.key, required this.title});

  final String title;

  @override
  ConsumerState<TrainingProgramsScreen> createState() => _TrainingProgramsScreenState();
}

class _TrainingProgramsScreenState extends ConsumerState<TrainingProgramsScreen> {

  void _createTrainingProgram() {
    final controller = TextEditingController();
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('New Training Plan'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(hintText: 'Enter name'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () {
              if (controller.text.isNotEmpty) {
                ref.read(trainingProgramListProvider.notifier).add(
                  MockTrainingProgram(name: controller.text),
                );
              }
              Navigator.pop(context);
            },
            child: const Text('Create'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          SizedBox(
            width: 300,
            child: Column(
              children: [
                Expanded(child: _TrainingProgramList()),
              ],
            ),
          ),
          const VerticalDivider(width: 1),
          Expanded(child: _TrainingProgramDetail()),
        ],
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
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final programs = ref.watch(trainingProgramListProvider);
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
            onTap: () => ref.read(selectedTrainingProgramProvider.notifier).select(index),
          ),
        );
      },
    );
  } 
}

class _TrainingProgramDetail extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final programs = ref.watch(trainingProgramListProvider);
    final selectedIndex = ref.watch(selectedTrainingProgramProvider);
    if (programs.isEmpty) {
      return const Center(child: Text('You have no training programs. Create one by pressing "+"'));
    }

    final program = programs[selectedIndex ?? 0];
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Text(program.name, style: const TextStyle(fontSize: 24)),
    );
  }
}