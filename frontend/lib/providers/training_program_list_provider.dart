import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:structure/models/mock_training_program.dart';

final trainingProgramListProvider = NotifierProvider<TrainingProgramListNotifier, List<MockTrainingProgram>>(
  TrainingProgramListNotifier.new,
);

class TrainingProgramListNotifier extends Notifier<List<MockTrainingProgram>> {
  @override
  List<MockTrainingProgram> build() => []; // initial state
  void add(MockTrainingProgram plan) => state = [...state, plan];
}