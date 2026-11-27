import 'package:material_ui/material_ui.dart';

import 'screens/field_log_screen.dart';
import 'storage/sqlite_schema.dart';
import 'sync/sync_engine.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final store = await SqfliteQueueStore.open();
  runApp(TernakMatricMobileApp(store: store));
}

class TernakMatricMobileApp extends StatelessWidget {
  const TernakMatricMobileApp({super.key, this.store});

  final SyncQueueStore? store;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'TernakMatric Field',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark(useMaterial3: true).copyWith(
        scaffoldBackgroundColor: const Color(0xFF020617),
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFF10B981),
          secondary: Color(0xFF34D399),
          surface: Color(0xFF0F172A),
        ),
      ),
      home: FieldLogScreen(store: store),
    );
  }
}
