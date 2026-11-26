import React, { useState } from "react";
import { Pressable, Text, TextInput, View } from "react-native";

export interface FieldLogDraft {
  batchCode: string;
  feedKg: string;
  mortalityCount: string;
}

export function FieldLogScreen() {
  const [draft, setDraft] = useState<FieldLogDraft>({
    batchCode: "B-NILA-2611",
    feedKg: "",
    mortalityCount: "0",
  });
  const [queuedCount, setQueuedCount] = useState(0);

  const enqueueOffline = () => {
    setQueuedCount((prev) => prev + 1);
  };

  return (
    <View style={{ flex: 1, padding: 20, backgroundColor: "#020617" }}>
      <Text style={{ color: "#34d399", fontWeight: "700", fontSize: 12 }}>
        TERNAKMATRIC · OFFLINE FIELD INPUT
      </Text>
      <Text style={{ color: "#ffffff", fontSize: 22, fontWeight: "700", marginTop: 4 }}>
        Catat Pakan &amp; Mortalitas
      </Text>
      <Text style={{ color: "#94a3b8", marginTop: 8 }}>
        Antrean SQLite lokal menunggu sinkronisasi: {queuedCount} operasi
      </Text>

      <TextInput
        value={draft.batchCode}
        onChangeText={(batchCode) => setDraft((d) => ({ ...d, batchCode }))}
        placeholder="Kode Batch"
        placeholderTextColor="#64748b"
        style={{
          marginTop: 16,
          padding: 12,
          borderRadius: 8,
          backgroundColor: "#0f172a",
          color: "#ffffff",
        }}
      />

      <TextInput
        value={draft.feedKg}
        onChangeText={(feedKg) => setDraft((d) => ({ ...d, feedKg }))}
        placeholder="Pakan (kg)"
        keyboardType="numeric"
        placeholderTextColor="#64748b"
        style={{
          marginTop: 12,
          padding: 12,
          borderRadius: 8,
          backgroundColor: "#0f172a",
          color: "#ffffff",
        }}
      />

      <Pressable
        onPress={enqueueOffline}
        style={{
          marginTop: 20,
          padding: 14,
          borderRadius: 10,
          backgroundColor: "#10b981",
          alignItems: "center",
        }}
      >
        <Text style={{ color: "#022c22", fontWeight: "700" }}>
          Simpan ke Antrean Offline
        </Text>
      </Pressable>
    </View>
  );
}
