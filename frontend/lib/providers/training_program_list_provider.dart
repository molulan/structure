import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:structure/src/bridge/api/mesocycles.dart' as bridge;
import 'package:structure/src/bridge/dto/planning.dart';

final trainingProgramListProvider = FutureProvider<List<MesocycleDTO>>((ref) async {
  return bridge.getMesocycles();
});

