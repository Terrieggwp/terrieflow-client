# TerrieFlow Client Implementation Risks

## Статус документа

Этот документ фиксирует открытые технические риски будущей реализации TerrieFlow Client и служит входом для ADR-001 и последующих архитектурных решений.

Документ:

- не выбирает Flutter, Kotlin Multiplatform или Native Android + iOS;
- не выбирает VPN engine;
- не утверждает целевые desktop-платформы;
- не заменяет threat model, security review или юридическую проверку;
- не означает, что перечисленные меры уже реализованы.

Дата оценки: 2026-07-21.

## Шкала оценки

### Вероятность

- **Низкая** — сценарий возможен, но уже существуют понятные и проверенные способы его предотвращения.
- **Средняя** — риск реалистичен и требует отдельной проверки.
- **Высокая** — при текущей неопределённости риск, вероятно, проявится без целенаправленной работы.

### Влияние

- **Среднее** — локальная переработка, задержка или ограничение функциональности.
- **Высокое** — существенная переработка архитектуры, платформы или release process.
- **Критическое** — невозможность выпуска, утечка трафика или секретов, компрометация пользователей либо нарушение требований магазина.

Вероятность оценивает текущую неопределённость, а не статистику инцидентов. После получения новых фактов оценки должны пересматриваться.

## Когда риск считается закрытым

Риск закрывается только при наличии проверяемого результата:

- официальное подтверждение ограничения или возможности;
- time-boxed technical spike;
- воспроизводимый тест;
- security или license review;
- утверждённая политика;
- принятое ADR с указанными последствиями и остаточным риском.

Фраза «библиотека это поддерживает» без проверки на целевой платформе не закрывает риск.

## 1. Выбор Flutter, Kotlin Multiplatform или Native

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| TECH-01 | Выбранный стек позволяет быстро создать UI, но делает интеграцию системного VPN и native engine чрезмерно сложной или нестабильной. | Средняя | Критическое | Для каждого финального кандидата подготовить одинаковый feasibility checklist: Android service boundary, Apple extension target, native bindings, secure storage, desktop packaging. Подтвердить критические пункты документацией и минимальными техническими spikes. | До принятия ADR-001. |
| TECH-02 | Реальная стоимость поддержки нескольких toolchains одним разработчиком окажется выше ожидаемой. Flutter требует Dart плюс native adapters; KMP — Kotlin/Gradle/Xcode; Native — отдельные Android/iOS и desktop реализации. | Высокая | Высокое | Оценить setup, clean build, debugging, signing и upgrade workflow на чистых окружениях. Зафиксировать число языков, build systems и platform-specific компонентов, которыми придётся владеть. | До принятия ADR-001. |
| TECH-03 | Наличие общего desktop UI будет ошибочно принято за наличие готового desktop VPN support. | Высокая | Высокое | Для Windows, macOS и Linux отдельно подтвердить TUN API, privileges, service model, signing, packaging и update mechanism. Не учитывать UI target как доказательство VPN feasibility. | Перечень desktop ОС — до ADR-001; техническая осуществимость — до начала desktop-реализации. |
| TECH-04 | Framework bridge станет каналом передачи полных конфигураций, токенов или raw network data в UI runtime. | Средняя | Критическое | Спроектировать минимальный `VpnEngine` contract и провести security review потоков данных через Dart/Kotlin/Swift/JS/native boundaries. Проверить, что UI получает только команды, агрегированное состояние и безопасные ошибки. | Контракт границы — до ADR-001; финальный review — до интеграции engine. |
| TECH-05 | Выбор будет основан на привлекательности UI-прототипа, а не на ограничениях версии 1.0. | Высокая | Высокое | Использовать для ADR-001 взвешенные критерии с обязательными blocker-критериями: Apple extension, Android service, engine embedding, desktop scope, release и security. | До принятия ADR-001. |

## 2. Интеграция VPN engine

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| ENG-01 | Лицензия Xray, sing-box, wrapper или транзитивной зависимости окажется несовместима с моделью распространения TerrieFlow. | Средняя | Критическое | Провести license inventory для core, wrappers, build tools и optional components. Проверить требования к модификациям, linking, attribution, source distribution и использованию названия. При необходимости получить юридическое заключение. | До ADR-002 и до включения engine в репозиторий. |
| ENG-02 | Engine собирается для заявленной ОС, но не может безопасно работать внутри Android service или Apple Network Extension из-за ABI, lifecycle, memory или threading ограничений. | Средняя | Критическое | Собрать минимальный engine artifact для каждой обязательной архитектуры ОС; проверить запуск и остановку в реальном platform container, повторный lifecycle и корректное освобождение ресурсов. | До ADR-002 для обязательных mobile-платформ. |
| ENG-03 | Неправильная передача TUN descriptor, socket protection или маршрутов вызовет routing loop либо отсутствие подключения. | Высокая | Критическое | Подготовить platform integration tests для descriptor ownership, protected tunnel socket, повторного connect/disconnect и смены сети. Проверить отсутствие self-routing. | До первой internal alpha с реальным туннелем. |
| ENG-04 | Ошибки IPv4/IPv6, DNS или split routing приведут к утечке трафика вне туннеля. | Высокая | Критическое | Создать leak-test matrix для IPv4, IPv6, DNS, network switching, reconnect, process death и fail-open/fail-closed поведения на каждой ОС. | До любой внешней beta с реальным VPN. |
| ENG-05 | Недоверенная или устаревшая конфигурация приведёт к подключению к неверному endpoint, ослаблению параметров или раскрытию секретов. | Средняя | Критическое | Определить schema, допустимые протоколы и поля, size limits, freshness и trust mechanism. Добавить negative, malformed и fuzz tests до передачи данных engine. | До появления реальных конфигураций в версии 0.2/1.0. |
| ENG-06 | Обновление engine изменит формат конфигурации, ABI или поведение маршрутизации и сломает клиент. | Высокая | Высокое | Зафиксировать версии, contract tests и compatibility matrix. Проверять release notes, migration path и rollback на каждом обновлении. | Политика — до ADR-002; автоматический gate — до beta. |
| ENG-07 | У проекта engine или wrapper отсутствует достаточно быстрый security response. | Средняя | Критическое | Оценить release cadence, security policy, advisory process, bus factor и историю критических исправлений. Определить SLA TerrieFlow на обновление и аварийное отключение версии. | До ADR-002. |

## 3. Android VpnService

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| AND-01 | UI и системное состояние VPN рассинхронизируются после отзыва разрешения, process death, переключения на другой VPN или запуска always-on. | Высокая | Высокое | Проверить `prepare`, `onRevoke`, system-initiated start, process recreation и единственный активный VPN. Источником истины сделать platform service, а не UI state. | До Android internal alpha. |
| AND-02 | Background и foreground-service ограничения приведут к остановке VPN, некорректному уведомлению или повышенному расходу батареи. | Средняя | Высокое | Прогнать lifecycle/device matrix на минимальной и последней поддерживаемой Android версии: background start, idle, battery saver, reboot, долгий tunnel session. | До Android closed beta. |
| AND-03 | Неверные manifest permissions или exported components позволят другому приложению вызвать внутренний сервис либо приведут к отклонению сборки. | Низкая | Критическое | Провести manifest security review; подтвердить `BIND_VPN_SERVICE`, intent filters и exported flags; добавить статическую проверку release manifest. | До первого APK/AAB с реальным `VpnService`. |
| AND-04 | Ошибка socket protection направит собственное соединение engine обратно в TUN. | Средняя | Критическое | Интеграционный тест с реальным TUN descriptor и наблюдением маршрута tunnel socket до и после reconnect/network change. | До Android internal alpha. |
| AND-05 | Always-on или lockdown режим заблокирует сеть при невозможности подключиться. | Средняя | Критическое | До заявления поддержки определить fail policy, UX восстановления и ограничения. Проверить boot, expired credentials, unavailable server и broken config. Если режим не поддерживается, явно и корректно отказаться от него. | До включения always-on/lockdown в любой build. |

## 4. Apple Network Extension

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| APPLE-01 | Команда или аккаунт распространения не соответствует требованиям Apple для VPN apps либо необходимый entitlement недоступен. | Средняя | Критическое | Подтвердить тип Apple Developer enrollment, App ID capabilities, Network Extension entitlement и предполагаемые территории распространения по актуальным правилам Apple. | До ADR-001, если iOS обязательна для выбранного пути. |
| APPLE-02 | Engine не укладывается в runtime, memory и lifecycle ограничения Packet Tunnel Provider. | Средняя | Критическое | Запустить выбранный engine внутри реального `NEPacketTunnelProvider` target; измерить startup, steady-state memory, reconnect и завершение extension на поддерживаемых устройствах. | До ADR-002 или до окончательного обязательства поддерживать iOS. |
| APPLE-03 | App и extension неправильно разделяют credentials, конфигурацию или состояние через Keychain access groups/App Groups. | Средняя | Критическое | Создать data-flow design, минимизировать shared data, проверить entitlements и доступ при locked/unlocked device, reinstall, logout и extension restart. | До реализации real storage на Apple-платформах. |
| APPLE-04 | Подпись главного приложения, extension, embedded frameworks или engine artifacts будет неконсистентной и заблокирует архивирование/публикацию. | Средняя | Высокое | Рано собрать distribution-signed archive со всеми вложенными components и проверить entitlements/signing order на CI и чистой машине. | До Apple internal distribution. |
| APPLE-05 | Поведение reconnect, network switching или system stop приведёт к утечке трафика либо ложному статусу подключения. | Высокая | Критическое | Platform tests для sleep/wake, Wi-Fi/cellular switching, extension termination, device reboot и unavailable gateway; сопоставить UI state с system status. | До внешней iOS beta. |

## 5. Desktop VPN adapters

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| DESK-01 | Термин `desktop` скрывает три существенно разные реализации: Windows, macOS и Linux. | Высокая | Критическое | Зафиксировать обязательные ОС, версии, архитектуры CPU и каналы распространения. Для каждой создать отдельный feasibility report. | Scope — до ADR-001. |
| DESK-02 | Для TUN, routing и DNS потребуется privileged service/helper с небезопасным IPC или чрезмерными правами. | Высокая | Критическое | Спроектировать отдельную trust boundary; провести threat model для IPC, authentication, command allowlist, privilege escalation и update path. | До реализации первого privileged component. |
| DESK-03 | Конфликт с firewall, endpoint security, другим VPN или системными route/DNS managers сделает соединение нестабильным. | Высокая | Высокое | Составить compatibility matrix и тестировать установку, connect/disconnect, crash recovery и uninstall рядом с типичными системными средствами и другим VPN. | До desktop closed beta. |
| DESK-04 | Подпись драйвера/helper, notarization или packaging заблокируют выпуск за пределами dev-машины. | Средняя | Критическое | Для каждой ОС создать ранний signed installation artifact и проверить установку на чистой поддерживаемой системе. Для macOS проверить Developer ID/App Store path и notarization. | До начала полной desktop-функциональности. |
| DESK-05 | Crash или некорректное удаление оставит маршруты, DNS либо сервис в повреждённом состоянии. | Высокая | Критическое | Fault-injection tests: kill UI, kill service, power interruption, failed update и uninstall. Проверить идемпотентное восстановление системной конфигурации. | До desktop external beta. |

## 6. Secure storage

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| STORE-01 | Android Keystore будет ошибочно использован как универсальное хранилище произвольных токенов, хотя он прежде всего управляет криптографическими ключами. | Высокая | Критическое | Утвердить data classification и storage design: что хранится непосредственно, что защищается hardware-backed key, где находится ciphertext и как исключаются backups. | До ADR-004 и реализации версии 0.2. |
| STORE-02 | Keychain/Keystore access policy окажется несовместима с background VPN: слишком строгая блокирует reconnect, слишком слабая облегчает использование секрета при краже устройства. | Средняя | Критическое | Для каждого секрета определить доступность при locked/unlocked state и необходимость background access. Проверить на реальном устройстве с reboot, lock и biometric/passcode changes. | До ADR-004. |
| STORE-03 | Logout, revoke, reinstall, backup/restore или migration не удалят все копии секретов. | Высокая | Критическое | Создать data inventory и lifecycle tests для create/read/update/delete, logout, account switch, backup restore, app update и corrupted storage. | До версии 0.2 с реальной авторизацией. |
| STORE-04 | Секреты попадут в память UI, state serialization, logs, crash reports, screenshots или clipboard. | Высокая | Критическое | Провести data-flow review и dynamic inspection; использовать allowlist диагностических полей; запретить секретные типы в presentation state и generic string errors. | Правила — до версии 0.2; verification — перед каждой beta. |
| STORE-05 | Desktop secure storage будет сведен к plain file или единой кроссплатформенной обёртке с неодинаковыми гарантиями ОС. | Средняя | Критическое | Выбрать и проверить отдельную OS-backed реализацию для каждой поддерживаемой desktop ОС; документировать различия threat model и migration. | До начала desktop authentication/storage. |
| STORE-06 | Root/jailbreak или компрометация процесса позволит использовать секреты, несмотря на secure storage. | Средняя | Высокое | Зафиксировать attacker capabilities и остаточный риск; проверить возможность non-exportable keys, device-bound access и server-side revoke. Не считать root detection самостоятельной защитой. | Threat model и ADR-004 — до production design. |

## 7. Release pipeline

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| REL-01 | Компрометация signing keys или provisioning credentials позволит выпустить вредоносное обновление либо лишит проект возможности обновлять приложение. | Средняя | Критическое | Утвердить key ownership, hardware/managed storage, least privilege, rotation, recovery и audit. Не передавать долгоживущие signing secrets недоверенным jobs. | До создания первых distribution credentials. |
| REL-02 | CI соберёт артефакт из отличающегося или непроверенного native core. | Средняя | Критическое | Pin source revisions и toolchains; фиксировать hashes, dependency lockfiles, provenance и SBOM; запретить неявное скачивание latest binaries во время release build. | До первой beta с native engine. |
| REL-03 | Debug flags, test endpoints, mock data или diagnostic capabilities попадут в release. | Средняя | Критическое | Разделить build configurations; добавить release assertions и artifact inspection для endpoints, entitlements, debug symbols/flags и mock adapters. | До первого store/internal distribution build. |
| REL-04 | Multi-platform releases будут содержать разные версии Core, policy или исправлений безопасности. | Высокая | Высокое | Ввести release manifest с версиями UI, engine, config schema и security fixes; блокировать частичный выпуск критических исправлений без явного решения. | До одновременного выпуска второй платформы. |
| REL-05 | Невозможность быстро откатить или заблокировать уязвимую версию увеличит последствия инцидента. | Средняя | Критическое | Определить rollback, phased release, server-side minimum version/kill mechanism без удалённого произвольного исполнения и incident runbook. | До public beta. |
| REL-06 | Сборка и подпись вложенных native frameworks, extensions или helpers окажутся невоспроизводимыми вне локальной машины. | Высокая | Высокое | Выполнять clean CI builds, архивировать build metadata и регулярно проверять release candidate на чистом runner для каждой ОС. | До internal distribution соответствующей платформы. |

## 8. App Store и Google Play

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| MARKET-01 | Google Play отклонит приложение из-за несоответствия VpnService policy, отсутствия declaration, disclosure, consent или некорректного Data safety. | Средняя | Критическое | До submission сверить актуальную policy; подготовить VpnService declaration, in-app disclosure/consent, privacy policy, Data safety answers и требуемое review video. | До первого Play Console testing track с `VpnService`. |
| MARKET-02 | Apple отклонит приложение из-за типа developer account, использования VPN API, data-use policy или территориальных требований. | Средняя | Критическое | Подтвердить organization enrollment, использование разрешённых Network Extension API, privacy disclosure и список территорий. Для территорий с лицензированием получить юридическое подтверждение. | Account/entitlement — до ADR-001; legal/store package — до App Store submission. |
| MARKET-03 | Third-party SDK, analytics или VPN engine собирает данные, противоречащие store declarations или заявленной privacy policy. | Средняя | Критическое | Выполнить SDK data inventory, runtime traffic inspection и review privacy behavior каждой зависимости. Сопоставить результат с disclosures и store forms. | До включения dependency и повторно перед каждым store release. |
| MARKET-04 | Функции или формулировки UI создадут вводящее в заблуждение представление о защите трафика, логировании или анонимности. | Средняя | Высокое | Провести product/legal/security review всех claims, onboarding и status screens; сопоставить их с фактической маршрутизацией и data handling. | До marketing materials и public beta. |
| MARKET-05 | Изменение правил магазина после реализации потребует архитектурной или продуктовой переработки. | Высокая | Высокое | Назначить периодический policy review, хранить checklist с датой проверки и повторять его перед каждым major release. Не кодировать policy assumptions в неизменяемую архитектуру. | Процесс — до первой submission; риск пересматривается постоянно. |

## 9. Supply-chain

| ID | Описание риска | Вероятность | Влияние | Способ проверки | Когда закрыть |
|---|---|---|---|---|---|
| SUP-01 | Компрометация package, plugin, wrapper или build tool внедрит код с доступом к пользовательскому трафику и секретам. | Средняя | Критическое | Минимизировать зависимости; pin версии и hashes; проверять maintainers, release signatures, advisories и транзитивный граф; генерировать SBOM и выполнять dependency scanning. | Baseline — до создания проекта; обязательный gate — до beta. |
| SUP-02 | Prebuilt Xray/sing-box artifact не соответствует заявленному source revision. | Средняя | Критическое | Предпочесть контролируемую сборку из pinned source либо проверять официальные signatures/hashes и provenance. Сравнивать версию runtime с release manifest. | До включения первого engine binary. |
| SUP-03 | Wrapper или framework plugin окажется заброшенным и заблокирует обновление ОС или engine. | Высокая | Высокое | Оценить bus factor, активность, совместимость с актуальными platform SDK и возможность поддерживать fork. Предпочитать малые внутренние adapters над крупными непроверенными plugins. | До ADR-001/ADR-002 для критичных integrations. |
| SUP-04 | Build scripts выполняют сетевую загрузку или произвольный код без проверки целостности. | Средняя | Критическое | Аудировать install/build hooks; запретить fetch-and-execute; использовать allowlist sources, immutable versions и verified checksums; ограничить network в release jobs. | До допуска dependency в release pipeline. |
| SUP-05 | Уязвимость native dependency останется незамеченной из-за отсутствия единого advisory и ownership процесса. | Высокая | Критическое | Назначить владельца dependency inventory; агрегировать advisories для Dart/Kotlin/Swift/Go/C/C++; определить SLA triage и emergency release. | Процесс — до beta; действует постоянно. |
| SUP-06 | Зависимость или обновление изменит privacy behavior, telemetry либо набор сетевых endpoints. | Средняя | Критическое | Сравнивать SBOM и runtime network behavior между версиями; запрещать неутверждённую telemetry; повторять security/privacy review при major update. | Перед каждым обновлением критичной зависимости. |

## Риски, блокирующие ADR-001

До выбора технологического стека необходимо получить достаточные ответы минимум по следующим пунктам:

- `TECH-01` — осуществимость системного VPN и native integration;
- `TECH-02` — реальная стоимость toolchains для одного разработчика;
- `TECH-03` и `DESK-01` — точный desktop scope;
- `TECH-04` — допустимая граница UI/native и секретов;
- `APPLE-01` — доступность требуемого Apple distribution path, если iOS обязательна;
- `SUP-03` — жизнеспособность критичных framework plugins/wrappers.

ADR-001 может принять остаточный риск, но должен явно назвать его, определить владельца и срок повторной проверки.

## Риски, блокирующие ADR-002

До выбора VPN engine необходимо закрыть или явно принять:

- `ENG-01` — лицензирование;
- `ENG-02` — embedding на обязательных платформах;
- `ENG-03` — TUN/socket integration;
- `ENG-06` — compatibility/update policy;
- `ENG-07` — security response;
- `SUP-02` — provenance бинарников.

## Порядок пересмотра

Risk register пересматривается:

- перед каждым ADR из `docs/adr/README.md`;
- при добавлении новой целевой ОС;
- при смене VPN engine или major framework version;
- перед первой internal alpha с реальным VPN;
- перед public beta;
- перед первой store submission;
- после security incident или критического advisory.

При закрытии риска следует сохранять ссылку на доказательство: ADR, test report, policy review, signed build report или security assessment.

## Актуальные первичные источники

- [Flutter platform integration](https://docs.flutter.dev/platform-integration)
- [Flutter native code and FFI](https://docs.flutter.dev/platform-integration/bind-native-code)
- [Kotlin Multiplatform](https://kotlinlang.org/docs/multiplatform.html)
- [Kotlin/Native C interoperability](https://kotlinlang.org/docs/native-c-interop.html)
- [Android VPN developer guide](https://developer.android.com/develop/connectivity/vpn)
- [Android Keystore](https://developer.android.com/privacy-and-security/keystore)
- [Google Play VpnService policy](https://support.google.com/googleplay/android-developer/answer/12564964?hl=en-GB)
- [Apple App Review Guidelines, section 5.4](https://developer.apple.com/app-store/review/guidelines/)
- [Apple NEPacketTunnelProvider](https://developer.apple.com/documentation/networkextension/nepackettunnelprovider)
- [Apple Keychain Services](https://developer.apple.com/documentation/security/keychain-services/)
- [Apple distribution workflow](https://developer.apple.com/documentation/xcode/creating-a-workflow-that-builds-your-app-for-distribution)
- [Apple macOS distribution signing](https://developer.apple.com/documentation/xcode/creating-distribution-signed-code-for-the-mac/)
- [Xray-core](https://github.com/XTLS/Xray-core)
- [libXray](https://github.com/XTLS/libXray)
- [sing-box](https://github.com/SagerNet/sing-box)
