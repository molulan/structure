import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:structure/screens/training_programs_screen.dart';
import 'src/bridge/frb_generated.dart' show RustLib;

void main() async {
  await RustLib.init();

  runApp(const ProviderScope(child: StructureApp()));
}

class StructureApp extends StatelessWidget {
  const StructureApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'StructureApp',
      theme: ThemeData(colorScheme: .fromSeed(seedColor: Colors.amber)),
      home: const TrainingProgramsScreen(),
    );
  }
}
