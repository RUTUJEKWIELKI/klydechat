# Klyde Migration Guide

:::warning

This is not yet finished.

:::

## Endpoint changes

| Service    | Old URL                             | New URL                                 |
| ---------- | ----------------------------------- | --------------------------------------- |
| **API**    | `https://api.revolt.chat`           | `https://klyde.chat/api`                |
|            | `https://app.revolt.chat/api`       | `https://klyde.chat/api`                |
|            | `https://revolt.chat/api`           | No equivalent                           |
| **Events** | `wss://ws.revolt.chat`              | `wss://klyde.chat/events`               |
|            | `wss://app.revolt.chat/events`      | `wss://klyde.chat/events`               |
|            | `wss://revolt.chat/events`          | No equivalent                           |
| **Files**  | `https://autumn.revolt.chat`        | `https://cdn.klydeusercontent.com`      |
|            | `https://cdn.revoltusercontent.com` | `https://cdn.klydeusercontent.com`      |
| **Proxy**  | `https://jan.revolt.chat`           | `https://external.klydeusercontent.com` |
| **Voice**  | `https://vortex.revolt.chat`        | Superseded by Voice Chats v2            |
