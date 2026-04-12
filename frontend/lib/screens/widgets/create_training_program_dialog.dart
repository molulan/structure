
import 'package:flutter/material.dart';

class CreateTrainingProgramDialog extends StatefulWidget {
  const CreateTrainingProgramDialog({super.key});

  @override
  createState() => _CreateTrainingProgramDialogState();
}

class _CreateTrainingProgramDialogState extends State<CreateTrainingProgramDialog> {
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
     return AlertDialog(
        title: const Text('New Training Plan'),
        content: TextField(
          controller: _controller,
          autofocus: true,
          decoration: const InputDecoration(hintText: 'Enter name'),
        ),
        actions: [
          TextButton(
            child: const Text('Cancel'),
            onPressed: () => Navigator.pop(context),
          ),
          TextButton(
            child: const Text('Create'),
            onPressed: () => Navigator.pop(context, _controller.text),
          ),
        ],
      );
  }

}