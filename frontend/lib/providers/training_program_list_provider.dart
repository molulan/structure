import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../src/bridge/api.dart' as bridge;
import '../src/bridge/lib.dart';

final trainingProgramListProvider = FutureProvider<List<Mesocycle>>((ref) async {
  return bridge.getMesocycles();
});

