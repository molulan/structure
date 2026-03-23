import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

void main() {
  runApp(
    const ProviderScope(
      child: StructureApp()
    ),
  );
}

class StructureApp extends StatelessWidget {
  const StructureApp({super.key});
  
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'StructureApp',
      theme: ThemeData(
        colorScheme: .fromSeed(seedColor: Colors.deepPurple),
      ),
      home: const TrainingPrograms(title: 'Training Programs'),
    );
  }
}

class TrainingPrograms extends StatefulWidget {
  const TrainingPrograms({super.key, required this.title});

  final String title;

  @override
  State<TrainingPrograms> createState() => _TrainingProgramsState();
}

class _TrainingProgramsState extends State<TrainingPrograms> {

  void _createTrainingPlan() {}

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: .center,
          children: [
            const Text("No training programs yet"),
            
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _createTrainingPlan,
        tooltip: 'Create New Trainingplan',
        child: const Icon(Icons.add),
      ),
    );
  }
}
