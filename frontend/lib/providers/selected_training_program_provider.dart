import 'package:flutter_riverpod/flutter_riverpod.dart';

final selectedTrainingProgramProvider = NotifierProvider<SelectedTrainingProgramNotifier, int?>(
  SelectedTrainingProgramNotifier.new,
);

class SelectedTrainingProgramNotifier extends Notifier<int?> {
  @override
  int? build() => null;

  void select(int index) => state = index;
}