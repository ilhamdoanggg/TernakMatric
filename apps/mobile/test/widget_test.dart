import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ternakmatric_mobile/main.dart';

void main() {
  testWidgets('FieldLogScreen queues valid offline feeding entry', (WidgetTester tester) async {
    await tester.pumpWidget(const TernakMatricMobileApp());

    expect(find.text('Catat Pakan & Mortalitas'), findsOneWidget);
    expect(find.text('Antrean SQLite lokal menunggu sinkronisasi: 0 operasi'), findsOneWidget);

    await tester.enterText(
      find.widgetWithText(TextFormField, 'Pakan (kg)'),
      '12.500',
    );
    await tester.tap(find.text('Simpan ke Antrean Offline'));
    await tester.pump();

    expect(find.text('Antrean SQLite lokal menunggu sinkronisasi: 1 operasi'), findsOneWidget);
    expect(find.text('Tersimpan di antrean offline lokal untuk B-NILA-2611.'), findsOneWidget);
  });
}
