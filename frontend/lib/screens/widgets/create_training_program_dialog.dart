import 'dart:io' show Platform;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:structure/src/bridge/dto/planning.dart';

class ProgramCreationResult {
  final String name;
  final int durationWeeks;
  final MesocycleModeDTO mode;

  const ProgramCreationResult({
    required this.name,
    required this.durationWeeks,
    required this.mode,
  });
}

class CreateTrainingProgramDialog extends StatefulWidget {
  const CreateTrainingProgramDialog({super.key});

  @override
  createState() => _CreateTrainingProgramDialogState();
}

class _CreateTrainingProgramDialogState
    extends State<CreateTrainingProgramDialog> {
  late final TextEditingController _nameController;
  late final TextEditingController _durationController;

  int _step = 1;
  MesocycleModeDTO _selectedMode = MesocycleModeDTO.algorithmic;
  late final bool _isMobile;

  @override
  void initState() {
    super.initState();

    _isMobile = Platform.isAndroid || Platform.isIOS;

    _nameController = TextEditingController();
    _durationController = TextEditingController();

    _nameController.addListener(_onFormChanged);
    _durationController.addListener(_onFormChanged);
  }

  void _onFormChanged() => setState(() {});

  @override
  void dispose() {
    _nameController.dispose();
    _durationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('New Training Plan'),
      content: SizedBox(
        width: 400,
        height: 280,
        child: SingleChildScrollView(
          child: _step == 1 ? _buildStep1Content() : _buildStep2Content(),
        ),
      ),
      actions: _step == 1 ? _buildStep1Actions() : _buildStep2Actions(),
    );
  }

  Widget _buildStep1Content() {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TextField(
          controller: _nameController,
          autofocus: true,
          decoration: const InputDecoration(
            labelText: 'Program name',
            hintText: 'e.g. Giant Arms FTW',
          ),
        ),
        const SizedBox(
          height: 16,
        ), // is this a legit way to add vertical space?
        TextField(
          controller: _durationController,
          keyboardType: TextInputType.number,
          inputFormatters: [FilteringTextInputFormatter.digitsOnly],
          decoration: InputDecoration(
            labelText: 'Duration (weeks)',
            hintText: 'e.g. 6',
          ),
        ),
        const SizedBox(height: 20),
        if (_isMobile)
          _buildModeCard(
            label: 'Algorithmic',
            description: 'Sets and load adjusted automatically',
            isSelected: true,
            onTap: null, //not rappable on mobile
          )
        else
          Row(
            children: [
              Expanded(
                child: _buildModeCard(
                  label: 'Algorithmic',
                  description: 'Sets and load adjusted automatically',
                  isSelected: _selectedMode == MesocycleModeDTO.algorithmic,
                  onTap: () => setState(
                    () => _selectedMode = MesocycleModeDTO.algorithmic,
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: _buildModeCard(
                  label: 'Manual',
                  description: 'You control everything week by week',
                  isSelected: _selectedMode == MesocycleModeDTO.manual,
                  onTap: () =>
                      setState(() => _selectedMode = MesocycleModeDTO.manual),
                ),
              ),
            ],
          ),
      ],
    );
  }

  Widget _buildModeCard({
    required String label,
    required String description,
    required bool isSelected,
    required VoidCallback? onTap,
  }) {
    final colorScheme = Theme.of(context).colorScheme;

    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          border: Border.all(
            color: isSelected
                ? colorScheme.primary
                : colorScheme.outlineVariant,
            width: 2,
          ),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label, style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: 4),
            Text(description, style: Theme.of(context).textTheme.bodySmall),
          ],
        ),
      ),
    );
  }

  List<Widget> _buildStep1Actions() {
    return [
      TextButton(
        onPressed: () => Navigator.pop(context),
        child: const Text('Cancel'),
      ),
      TextButton(
        onPressed: _isStep1Valid ? () => setState(() => _step = 2) : null,
        child: const Text('Next ->'),
      ),
    ];
  }

  bool get _isStep1Valid =>
      _nameController.text.trim().isNotEmpty && _isDurationValid;

  bool get _isDurationValid {
    final value = int.tryParse(_durationController.text);
    return value != null && value >= 1 && value <= 52;
  }

  Widget _buildStep2Content() {
    if (_selectedMode == MesocycleModeDTO.algorithmic) {
      return const Text(
        'Each week the algorithm will pre-fill the workouts with suggested volume and load based on progression and preferences',
      );
    } else {
      return const Text(
        'You pre plan exercises, sets, and load for the duration of the entire program',
      );
    }
  }

  List<Widget> _buildStep2Actions() {
    return [
      TextButton(
        onPressed: () => setState(() => _step = 1),
        child: const Text('<- Back'),
      ),
      TextButton(
        onPressed: () {
          final result = ProgramCreationResult(
            name: _nameController.text.trim(),
            durationWeeks: int.parse(_durationController.text),
            mode: _selectedMode,
          );
          Navigator.pop(context, result);
        },
        child: const Text('Create Program ->'),
      ),
    ];
  }
}
