# Security Requirements

## Sensitive Data

К чувствительным данным относятся:

- VPN subscription URL
- UUID
- access tokens
- server configs

## Storage

Использовать:

Android:
- Keystore

iOS:
- Keychain

Запрещено:

- SharedPreferences
- plain files
- logs

## Network

Все соединения должны:

- использовать TLS;
- проверять сертификаты;
- валидировать ответы сервера.

## Logging

Запрещено логировать:
- UUID пользователей
- токены
- subscription URL
- полные VPN-конфигурации
- персональные идентификаторы

Разрешены:
- обезличенные события;
- коды ошибок;
- технические метрики без пользовательских данных.
