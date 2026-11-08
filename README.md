<p align="center">
  <img src="brand/TerrieFlow_Logo_Kit/01_primary_horizontal/png/terrieflow-horizontal-on-dark_512.png" width="360" alt="TerrieFlow">
</p>

<p align="center"><strong>Официальный Windows-клиент TerrieFlow</strong></p>
<p align="center"><a href="README.en.md">English</a> · <a href="PRIVACY.md">Конфиденциальность</a> · <a href="SECURITY.md">Безопасность</a></p>

TerrieFlow Client — настольный клиент для подключения к TerrieFlow и совместимым VPN-подпискам. Он объединяет вход в аккаунт, автоматический импорт подписки, выбор маршрута, управление Xray/Mihomo и безопасное обновление приложения.

## Возможности

- вход в TerrieFlow через Telegram или почту и пароль;
- автоматическая синхронизация подписки из аккаунта;
- ручной импорт внешней VPN-ссылки без обязательного входа;
- автоопределение лучшего маршрута, режимы скорости и доступности;
- маршруты по странам, измерение задержки и Quick Panel в трее;
- корректные состояния активной, grace- и истёкшей подписки;
- Xray и пропатченный Mihomo в одном проверяемом комплекте;
- обновления только из `Terrieggwp/terrieflow-client` с проверкой версии, архитектуры и SHA-256.

## Скачать

Первый официальный beta-релиз поддерживает только **Windows 10/11 x64**.

[Скачать TerrieFlow Client для Windows x64](https://github.com/Terrieggwp/terrieflow-client/releases/latest/download/TerrieFlow-Client-Setup-x64.exe)

Проверьте файл по `RELEASE-SHA256SUMS.txt` из того же релиза. Текущая beta-сборка пока не подписана сертификатом издателя, поэтому Windows SmartScreen может показать предупреждение. Не устанавливайте файл, если имя, источник или контрольная сумма отличаются.

## Начало работы

1. Установите клиент и запустите TerrieFlow.
2. Войдите через Telegram или почту — доступная подписка добавится автоматически.
3. Либо вставьте ссылку внешней VPN-подписки вручную.
4. Выберите маршрут и включите защиту.

Данные приложения по историческим причинам хранятся в `%LOCALAPPDATA%\MultiCore`. Этот путь пока сохранён ради совместимости с уже установленными версиями. Приложение устанавливается как TerrieFlow.

## Статус beta

Windows x64 — единственная проверенная платформа первого публичного beta-цикла. Linux, Windows x86 и Windows ARM64 остаются в исходном коде как полезная база разработки, но не заявлены как поддерживаемые релизные платформы. Известные ограничения и критерии выпуска перечислены в [дорожной карте](docs/roadmap.md) и [процессе релиза](docs/release-process.md).

Актуальные скриншоты должны сниматься только с тестовым аккаунтом и обезличенными данными. Чек-лист находится в [docs/screenshots.md](docs/screenshots.md); старые изображения MultiCore не считаются изображениями текущего продукта.

## Разработка

Требуются Rust `1.98.1`, Windows SDK/MSVC и зависимости, описанные в [docs/development.md](docs/development.md).

```powershell
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-release-workflow-contract.ps1 -ExpectedVersion 1.1.17
```

Архитектура и границы привилегий описаны в [docs/architecture.md](docs/architecture.md), пользовательский поток — в [docs/how-it-works.md](docs/how-it-works.md).

## Лицензия и происхождение

Проект распространяется по лицензии [GPL-3.0](LICENSE). TerrieFlow Client создан на основе открытого проекта [MultiCore](https://github.com/fatyzzz/multicore). Уведомления и исходный код поставляемых GPL/MPL-компонентов публикуются вместе с релизом. Торговые марки и брендовые материалы TerrieFlow не меняют условия лицензирования исходного кода.
