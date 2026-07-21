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

- UUID
- токены
- ссылки подписок
- конфиги подключения
