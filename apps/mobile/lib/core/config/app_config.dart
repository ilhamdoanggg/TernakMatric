/// Konfigurasi runtime aplikasi mobile dari `--dart-define` / `--dart-define-from-file`.
class AppConfig {
  const AppConfig({
    required this.appEnv,
    required this.apiUrl,
  });

  final String appEnv;
  final String apiUrl;

  factory AppConfig.fromEnvironment() {
    const appEnv = String.fromEnvironment('APP_ENV', defaultValue: 'development');
    const rawUrl = String.fromEnvironment(
      'API_URL',
      defaultValue: 'http://10.0.2.2:8080',
    );
    return AppConfig(
      appEnv: appEnv,
      apiUrl: rawUrl.replaceFirst(RegExp(r'/+$'), ''),
    );
  }
}
