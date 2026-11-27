import 'package:material_ui/material_ui.dart';
import 'package:uuid/uuid.dart';

import '../offline/conflict_policy.dart';
import '../sync/sync_engine.dart';

class FieldLogScreen extends StatefulWidget {
  const FieldLogScreen({
    super.key,
    this.store,
  });

  /// Opsional; bila null (mis. pada widget test terisolasi), antrean memakai state lokal.
  final SyncQueueStore? store;

  @override
  State<FieldLogScreen> createState() => _FieldLogScreenState();
}

class _FieldLogScreenState extends State<FieldLogScreen> {
  final _formKey = GlobalKey<FormState>();
  final _batchCodeController = TextEditingController(text: 'B-NILA-2611');
  final _feedKgController = TextEditingController();
  final _mortalityController = TextEditingController(text: '0');

  int _queuedCount = 0;
  String? _statusMessage;

  @override
  void initState() {
    super.initState();
    _loadQueuedCount();
  }

  Future<void> _loadQueuedCount() async {
    final store = widget.store;
    if (store == null) {
      return;
    }
    final count = await store.countPending();
    if (!mounted) {
      return;
    }
    setState(() {
      _queuedCount = count;
    });
  }

  @override
  void dispose() {
    _batchCodeController.dispose();
    _feedKgController.dispose();
    _mortalityController.dispose();
    super.dispose();
  }

  Future<void> _enqueueOffline() async {
    if (!(_formKey.currentState?.validate() ?? false)) {
      return;
    }

    final batchCode = _batchCodeController.text.trim();
    final feedDecimal = _feedKgController.text.trim().replaceAll(',', '.');
    final mortalityCount = int.parse(_mortalityController.text.trim());

    final store = widget.store;
    if (store != null) {
      final command = buildFieldLogCommand(
        batchCode: batchCode,
        entityId: const Uuid().v4(),
        feedKgDecimal: feedDecimal,
        mortalityCount: mortalityCount,
        occurredAtUtc: DateTime.now().toUtc(),
      );
      await store.enqueue(command);
      final updatedCount = await store.countPending();
      if (!mounted) {
        return;
      }
      setState(() {
        _queuedCount = updatedCount;
        _statusMessage = 'Tersimpan di antrean SQLite untuk $batchCode.';
      });
      return;
    }

    setState(() {
      _queuedCount += 1;
      _statusMessage = 'Tersimpan di antrean offline lokal untuk $batchCode.';
    });
  }

  String? _validateBatchCode(String? value) {
    if (value == null || value.trim().isEmpty) {
      return 'Kode batch wajib diisi';
    }
    return null;
  }

  String? _validateDecimal(String? value) {
    final normalized = (value ?? '').trim().replaceAll(',', '.');
    if (normalized.isEmpty) {
      return 'Jumlah pakan (kg) wajib diisi';
    }
    final pattern = RegExp(r'^\d+(\.\d{1,3})?$');
    if (!pattern.hasMatch(normalized)) {
      return 'Masukkan angka desimal valid (maks. 3 desimal)';
    }
    return null;
  }

  String? _validateMortality(String? value) {
    final parsed = int.tryParse((value ?? '').trim());
    if (parsed == null || parsed < 0) {
      return 'Mortalitas harus bilangan bulat >= 0';
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF020617),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(20),
          child: Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                const Text(
                  'TERNAKMATRIC · OFFLINE FIELD INPUT',
                  style: TextStyle(
                    color: Color(0xFF34D399),
                    fontWeight: FontWeight.w700,
                    fontSize: 12,
                    letterSpacing: 0.8,
                  ),
                ),
                const SizedBox(height: 6),
                const Text(
                  'Catat Pakan & Mortalitas',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 22,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  'Antrean SQLite lokal menunggu sinkronisasi: $_queuedCount operasi',
                  style: const TextStyle(color: Color(0xFF94A3B8), fontSize: 14),
                ),
                const SizedBox(height: 16),
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: const Color(0xFF0F172A),
                    borderRadius: BorderRadius.circular(10),
                    border: Border.all(color: const Color(0xFF1E293B)),
                  ),
                  child: Text(
                    describeConflictPolicy('feeding_records'),
                    style: const TextStyle(color: Color(0xFFCBD5E1), fontSize: 12),
                  ),
                ),
                const SizedBox(height: 20),
                TextFormField(
                  controller: _batchCodeController,
                  validator: _validateBatchCode,
                  style: const TextStyle(color: Colors.white),
                  decoration: _inputDecoration('Kode Batch'),
                ),
                const SizedBox(height: 12),
                TextFormField(
                  controller: _feedKgController,
                  validator: _validateDecimal,
                  keyboardType: const TextInputType.numberWithOptions(decimal: true),
                  style: const TextStyle(color: Colors.white),
                  decoration: _inputDecoration('Pakan (kg)'),
                ),
                const SizedBox(height: 12),
                TextFormField(
                  controller: _mortalityController,
                  validator: _validateMortality,
                  keyboardType: TextInputType.number,
                  style: const TextStyle(color: Colors.white),
                  decoration: _inputDecoration('Mortalitas (ekor)'),
                ),
                const SizedBox(height: 20),
                ElevatedButton(
                  onPressed: _enqueueOffline,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF10B981),
                    foregroundColor: const Color(0xFF022C22),
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                  ),
                  child: const Text(
                    'Simpan ke Antrean Offline',
                    style: TextStyle(fontWeight: FontWeight.w700),
                  ),
                ),
                if (_statusMessage != null) ...<Widget>[
                  const SizedBox(height: 12),
                  Text(
                    _statusMessage!,
                    style: const TextStyle(color: Color(0xFF34D399), fontSize: 13),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }

  InputDecoration _inputDecoration(String label) {
    return InputDecoration(
      labelText: label,
      labelStyle: const TextStyle(color: Color(0xFF94A3B8)),
      filled: true,
      fillColor: const Color(0xFF0F172A),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: Color(0xFF1E293B)),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: Color(0xFF1E293B)),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: Color(0xFF10B981)),
      ),
    );
  }
}
