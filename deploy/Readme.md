Коротко: в `deploy/` обычно лежит **всё, что нужно, чтобы развернуть проект вживую**, а не сам код.

Типичное содержимое `deploy/`:

1. 🐳 **Описания окружений**
   - `dev/`, `stage/`, `prod/` подкаталоги.
   - В каждом – свои конфиги и манифесты.
2. ☸️ **Kubernetes / Helm**
   - `k8s/*.yaml` – Deployment, Service, Ingress, ConfigMap, Secret (шаблоны).
   - `helm/` – чарты, values-файлы.
3. 📦 **Docker orchestration**
   - `docker-compose.yml` для локального и стейдж-режимов.
   - Overlays типа `docker-compose.prod.yml`.
4. 🧩 **Infra-as-code / provisioning**
   - `terraform/` – описания VPC, БД, Bucket-ов, нод и т.п.
   - `ansible/`/`salt/` – плейбуки для настройки серверов.
5. ⚙️ **Юниты и сервисы**
   - `systemd/*.service` – юниты для `z00z_rollup_node`, `z00z_aggregator`, `z00z_walletd`.
   - Nginx/Traefik конфиги, если нужен reverse-proxy.
6. 🔐 **Шаблоны конфигов**
   - `config.example.yaml`, `env.prod.example` – примеры, без секретов.
   - Mapping: как переменные окружения превращаются в настройки ноды/кошелька.

Разделение по смыслу:

- `crates/` – **как работает Z00Z**.
- `deploy/` – **как это запускать в конкретной инфраструктуре**.
