# TerrieFlow Client Technology Selection

## Статус документа

Документ сравнивает технологические направления для TerrieFlow Client и готовит входные данные для будущего ADR-001.

Окончательное решение здесь не принимается. Версии инструментов, platform requirements и состояние интеграций необходимо повторно проверить непосредственно перед ADR.

Дата анализа: 2026-07-21.

## Контекст

TerrieFlow Client развивается одним разработчиком по этапам:

- `0.1` — UI-прототип на synthetic mock data;
- `0.2` — test API и реальное защищённое хранилище;
- `1.0` — реальный VPN engine для Android, iOS и выбранных desktop-платформ.

Главный архитектурный факт: ни один рассматриваемый UI-стек не устраняет платформенный код VPN. Android требует интеграцию с `VpnService`, а Apple packet tunnel работает как Network Extension app extension и требует соответствующий entitlement. Поэтому выбор оценивается не только по скорости UI-разработки, но и по стоимости безопасных нативных адаптеров.

## Критерии

### Скорость разработки одним разработчиком

Оцениваются скорость создания UI, количество языков и toolchains, hot reload, объём дублирования и сложность сборки.

### Android VpnService

Оцениваются прямота доступа к Android API, управление service lifecycle, permissions, foreground service, always-on и передача TUN file descriptor движку.

### Apple Network Extension

Оцениваются создание отдельного extension target, entitlement, Swift/Objective-C interoperability, жизненный цикл extension и обмен состоянием с главным приложением.

### Desktop

Оценивается UI-поддержка Windows, macOS и Linux. Наличие desktop UI не означает готовую системную VPN-интеграцию: привилегии, TUN и lifecycle остаются платформенными.

### Интеграция Xray/sing-box

Оцениваются FFI, C/Go bindings, gomobile/xcframework, упаковка бинарников, передача TUN и поддержка платформ.

### Безопасность

Оцениваются размер bridge-поверхности, размещение секретов, доступ к системному secure storage, изоляция VPN-процесса и supply-chain.

### Долгосрочная поддержка и сопровождение

Оцениваются зрелость экосистемы, обновления платформ, количество нативных адаптеров, сложность CI и вероятность накопления framework-specific debt.

## Сводное сравнение

Оценки являются относительными для условий TerrieFlow, а не универсальным рейтингом технологий.

| Критерий | Flutter | Kotlin Multiplatform | Native Android + iOS | React Native |
|---|---|---|---|---|
| Скорость UI v0.1 одним разработчиком | Высокая | Средняя–высокая | Низкая | Высокая |
| Android `VpnService` | Через Kotlin/Java plugin | Очень сильная сторона | Наилучший прямой доступ | Через native module |
| Apple Network Extension | Через Swift/Obj-C plugin и extension | Через iOS target/interop, часто со Swift extension | Наилучший прямой доступ | Через native module и extension |
| Общий mobile UI | Высокий | Высокий с Compose либо общий core с native UI | Отсутствует | Высокий |
| Desktop UI | Официальные Windows/macOS/Linux targets | Compose Multiplatform для Windows/macOS/Linux | Отдельная реализация для каждой ОС | Windows/macOS вне основного core; Linux слабее |
| Xray/sing-box integration | FFI или platform plugins | gomobile/C interop/platform wrappers | Прямые platform wrappers | Turbo Native Modules/C++/platform wrappers |
| Bridge-поверхность для VPN | Средняя | Низкая на Android, средняя на Apple/desktop | Минимальная | Средняя–высокая |
| Toolchain complexity | Средняя | Средняя–высокая | Высокая | Средняя–высокая с desktop |
| Дублирование кода | Низкое | Низкое–среднее | Высокое | Низкое для UI, среднее для native adapters |
| Долгосрочная стоимость для одного разработчика | Средняя | Средняя | Высокая | Средняя–высокая |

## 1. Flutter

### Сильные стороны

- Быстрое создание единого UI для mobile и desktop.
- Одна основная кодовая база и предсказуемая визуальная модель.
- Официальная поддержка интеграции с платформенным кодом через plugins и platform channels.
- Официальный FFI-механизм для вызова C API и упаковки native code.
- Удобен для быстрого прототипа `0.1` одним разработчиком.

Flutter официально поддерживает несколько целевых платформ и предусматривает собственные plugins, если готовой платформенной функции нет. Для C API предусмотрен `dart:ffi` и FFI package workflow.

### Ограничения

- Android `VpnService` и Apple Network Extension всё равно реализуются на Kotlin/Java и Swift/Objective-C.
- Apple VPN extension является отдельным target; Flutter UI не должен запускаться внутри packet tunnel extension.
- Для Windows, macOS и Linux потребуются разные системные VPN adapters.
- Интеграция Go-движка может потребовать C ABI, gomobile-generated framework или отдельный процесс.
- Неконтролируемые community plugins увеличивают supply-chain и secret-handling риски.

### Безопасность

- Секреты и полная VPN-конфигурация не должны пересекать UI bridge без необходимости.
- `VpnEngine` plugin должен иметь узкий типизированный контракт.
- Secure storage должен опираться на платформенные механизмы, а не на обычное Dart-хранилище.
- Необходим review нативного кода, lifecycle и error mapping для каждой платформы.

### Итог

Сильный кандидат для `0.1` и единого UI, если заранее признать наличие нативного VPN-слоя и подтвердить интеграцию выбранного движка отдельным техническим исследованием.

## 2. Kotlin Multiplatform

### Сильные стороны

- Естественный доступ к Android и отличная совместимость с `VpnService`.
- Можно разделять domain/application code, а UI оставить нативным или использовать Compose Multiplatform.
- Compose Multiplatform поддерживает общий UI для Android, iOS и desktop.
- Kotlin/Native предоставляет interoperability с C и Objective-C.
- Хорошо соответствует hexagonal architecture: общий Core и платформенные adapters.

### Ограничения

- Gradle, Kotlin/Native и Xcode образуют более сложный toolchain.
- Apple Network Extension остаётся отдельным Xcode target; часть кода и lifecycle обычно удобнее реализовывать на Swift.
- Kotlin C interop имеет участки API со статусом Beta/Experimental, что важно для долгоживущего VPN bridge.
- Compose Multiplatform добавляет собственный цикл совместимости Kotlin/Compose и platform-specific UI-особенности.
- Для разработчика без опыта Kotlin/Gradle скорость `0.1` может быть ниже Flutter или React Native.

### Безопасность

- Позволяет держать VPN boundary ближе к platform code, особенно на Android.
- Общий Core не должен получать TUN descriptors, raw packets или platform credentials.
- C/Go interop требует отдельного review памяти, callbacks, lifecycle и очистки ресурсов.
- KMP не отменяет Keychain, entitlement и extension-specific требования Apple.

### Итог

Сильный кандидат, если приоритет — Android-first, Kotlin уже знаком разработчику и важнее нативный контроль, чем максимальная скорость первого UI-прототипа.

## 3. Native Android + iOS

### Сильные стороны

- Прямой доступ ко всем VPN, background и secure storage API.
- Минимум framework bridges между UI и системным VPN-компонентом.
- Наиболее ясное соответствие platform lifecycle и требованиям магазинов.
- Проще использовать platform-specific debugging и security tooling.
- Меньше ограничений при работе с Network Extension entitlement и Android services.

### Ограничения

- Две mobile-кодовые базы и как минимум третья реализация для desktop.
- Максимальное дублирование UI, navigation, state и части бизнес-логики.
- Самая низкая скорость `0.1` для одного разработчика.
- Исправления и security changes требуется синхронно переносить между платформами.
- Desktop-фаза фактически становится отдельным проектом.

### Безопасность

- Прямой API-доступ уменьшает bridge-риск, но не гарантирует безопасность автоматически.
- Больший объём дублированного кода увеличивает вероятность расхождения security policy.
- Каждая платформа требует отдельного threat model refinement, тестов утечек и release pipeline.

### Итог

Лучший вариант по глубине платформенной интеграции, но несоразмерно дорогой для быстрого UI-прототипа и одного разработчика, если одновременная поддержка Android, iOS и desktop остаётся целью.

## 4. React Native

### Сильные стороны

- Высокая скорость UI-разработки при опыте React/TypeScript.
- Общий mobile UI и развитая ecosystem.
- Native Modules позволяют вызывать Kotlin, Swift и C++.
- Существуют отдельные поддерживаемые проекты для Windows и macOS.

### Ограничения

- VPN functionality отсутствует в основном React Native API и требует собственных native modules.
- Текущая архитектура ориентируется на Turbo Native Modules/Fabric; legacy modules постепенно уходят.
- Windows и macOS являются out-of-tree platforms, а Linux не имеет сопоставимой официальной основной поддержки.
- Реальный desktop увеличивает число toolchains и platform-specific module implementations.
- Секреты и большие конфигурации нежелательно передавать через JavaScript runtime и bridge.
- Обновления React Native architecture могут повышать стоимость сопровождения custom native modules.

### Безопасность

- VPN engine и secure storage должны оставаться в native layer.
- JavaScript получает только минимальное безопасное состояние и команды.
- Требуется строгий контроль npm supply chain, debug tooling, source maps и сериализации ошибок.
- Native module contract должен исключать raw tokens, configs и packets.

### Итог

Подходит для быстрого mobile UI при сильном опыте React, но менее убедителен для TerrieFlow из-за desktop-фрагментации и значимого объёма custom native VPN code.

## Интеграция Xray и sing-box

Выбор UI-фреймворка не равен выбору VPN engine.

Наблюдения, требующие проверки в ADR-002:

- Xray-core написан на Go и предоставляет сборки для нескольких платформ.
- Проект `libXray` описывает сборку для Android, Apple, Linux и Windows, включая варианты, пригодные для Kotlin, Swift и Dart FFI.
- Xray TUN integration требует корректной передачи platform TUN descriptor и отдельно предупреждает о риске routing loops.
- sing-box также написан на Go, имеет собственные platform clients и TUN-функции, но лицензия и правила распространения должны быть проверены до выбора.
- Наличие готовой сборки не доказывает соответствие требованиям App Store, Google Play, threat model или конкретному UI-фреймворку.

Для каждого кандидата нужно проверить:

1. лицензию и возможность коммерческого распространения;
2. официальный способ embedding, а не запуск неподконтрольного executable;
3. Android TUN descriptor и socket protection;
4. Apple Network Extension memory limits и lifecycle;
5. Windows/macOS/Linux privileges и service model;
6. подпись, воспроизводимость и обновление core;
7. IPv4/IPv6, DNS, routing и leak protection;
8. минимальную версию ОС и поддерживаемые архитектуры;
9. процесс реагирования на security advisories.

## Security requirements, общие для любого стека

- UI runtime не получает raw packets.
- Полная VPN-конфигурация не хранится в UI state.
- Токены и ключевой материал не сериализуются через bridge без необходимости.
- Platform VPN adapter является отдельной проверяемой границей.
- Все внешние данные валидируются до передачи движку.
- Debug logs, crash reports и analytics используют allowlist безопасных полей.
- Development/test сборки технически не могут использовать production endpoints.
- Native dependencies фиксируются, проверяются и обновляются контролируемо.
- Logout, revoke, app restart и process death имеют определённую политику очистки.
- Решение по stack проходит security review до принятия ADR-001, а engine — отдельный review перед ADR-002.

## Рекомендация

Для текущих целей рекомендуется рассматривать **Flutter как основной кандидат для ADR-001**, но не считать его выбранным.

Причины:

- высокая скорость создания `0.1` одним разработчиком;
- единый UI для mobile и desktop;
- официальные механизмы platform integration и FFI;
- возможность сохранить предложенную hexagonal boundary вокруг `VpnEngine`.

Рекомендация условна. До принятия решения необходимо документально или экспериментально подтвердить:

- создание Android `VpnService` adapter без передачи секретов в Dart;
- архитектуру отдельного Apple Network Extension target;
- способ embedding выбранного Xray/sing-box core;
- перечень реально поддерживаемых desktop ОС;
- release и signing model для native core.

**Kotlin Multiplatform является предпочтительной альтернативой**, если проект становится Android-first, разработчик уверенно владеет Kotlin/Gradle или приоритет нативного VPN-контроля выше скорости UI-прототипа.

Native Android + iOS следует выбирать, если platform fidelity и контроль оказываются важнее стоимости нескольких реализаций. React Native стоит выбирать прежде всего при сильной React/TypeScript экспертизе и снижении требований к единому desktop-клиенту.

## Что должно войти в ADR-001

- Первая целевая платформа `0.1`.
- Опыт разработчика с Dart, Kotlin, Swift и TypeScript.
- Решение о shared UI или только shared Core.
- Перечень desktop ОС.
- Результат проверки native VPN bridge.
- Результат предварительного license/security review engine-кандидатов.
- Стоимость CI и сборок на Android/iOS/desktop.
- План миграции mock adapters к platform adapters.
- Известные риски и условие пересмотра решения.

## Официальные источники

- [Flutter: multi-platform and platform integration](https://docs.flutter.dev/platform-integration)
- [Flutter: native code through FFI](https://docs.flutter.dev/platform-integration/bind-native-code)
- [Kotlin Multiplatform overview](https://kotlinlang.org/docs/multiplatform.html)
- [Compose Multiplatform](https://kotlinlang.org/docs/multiplatform/compose-multiplatform.html)
- [Kotlin/Native C interoperability](https://kotlinlang.org/docs/native-c-interop.html)
- [React Native native platform APIs](https://reactnative.dev/docs/native-platform)
- [React Native out-of-tree platforms](https://reactnative.dev/docs/next/out-of-tree-platforms)
- [React Native Windows](https://microsoft.github.io/react-native-windows/)
- [Android VPN developer guide](https://developer.android.com/develop/connectivity/vpn)
- [Apple NEPacketTunnelProvider](https://developer.apple.com/documentation/networkextension/nepackettunnelprovider)
- [Xray-core](https://github.com/XTLS/Xray-core)
- [libXray platform wrapper](https://github.com/XTLS/libXray)
- [sing-box](https://github.com/SagerNet/sing-box)
- [sing-box platform clients](https://sing-box.sagernet.org/clients/)
