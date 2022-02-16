# Read Me

## サービス

### サービスの配置

各サービスは、`services`モジュールにエントリポイントを作成する。
エントリポイントとなる`main`関数を持つファイルは、`services/src/bin/`ディレクトリに配置する。

### サービスの実行

各サービスは、以下の通り実行する。

```bash
cargo run --bin <main-fun-file>

# Web APIサービスを起動する場合
cargo run --bin web_api_service
```

## モジュール構成

| モジュール名 | 内容                                           |
| ------------ | ---------------------------------------------- |
| `services`   | サービスのエントリポイントを配置               |
| `adapters`   | サービスとユースケースを接続するアダプタを配置 |
| `usecases`   | ユースケースで実行する処理を配置               |
| `infra`      | データストアを配置                             |
| `domains`    | エンティティ、値オブジェクトを配置             |
| `common`     | 上記モジュールから呼び出す一般的な機能を配置   |

### モジュールの依存関係

* `services` -> `common`
* `services` -> `adapters`
* `adapters` -> `common`
* `adapters` -> `usecases`
* `adapters` -> `infra`
* `infrastructures` -> `usecases`
* `infrastructures` -> `domains`
* `usecases` -> `common`
* `usecases` -> `domains`
* `domains` -> `common`

## sqlx

### 準備

`sqlx-cli`を以下の通りインストールする。

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

`.env`ファイルを以下の通り作成する。

```text
DATABASE_URL=postgres://postgres:password@localhost:5432/axum-seaorm-ddd-example
```

`Docker`サービスを起動した後、`Docker`コンテナを以下の通り起動する。

```bash
docker-compose up -d    # d: バックグラウンドでコンテナを起動
```

#### Dockerコマンド

| コマンド | 説明                                                                                                |
| -------- | --------------------------------------------------------------------------------------------------- |
| up       | コンテナの構築、作成、起動、アタッチ。                                                              |
| down     | コンテナを停止して、upで作成したコンテナ、ネットワーク、ボリューム及びイメージを削除。              |
| start    | 既存のコンテナを起動。                                                                              |
| stop     | 起動中のコンテナを削除。                                                                            |
| restart  | 起動中のコンテナを再起動。                                                                          |
| kill     | 起動中のコンテナにシグナルを送信。`-s`オプションでシグナルを指定。デフォルトのシグナルは`SIGKILL`。 |

### データベースの作成

データベースを以下の通り作成する。

```bash
sqlx database create

# データベースを削除する場合
# sqlx database drop
```

### マイグレーションファイルの作成と実行

マイグレーションファイルを以下の通り作成する。

```bash
sqlx migrate add <name>
```

`migrations/<timestamp>-<name>.sql`ファイルが作成される。
このファイルにデータベーススキーマの変更を記述する。

マイグレーションを以下の通り実行する。

```bash
sqlx migrate run
```

実行しているデータベースのマイグレーション履歴と、`migrations`ディレクトリのマイグレーションファイルを比較して、実行されていないスクリプトを実行する。

### マイグレーションを戻す

`up`と`down`スクリプトに対応する戻すことが可能なマイグレションを作成したい場合は、マイグレーションを作成するときに、`-r`フラグを追加する。

```bash
sqlx migrate add -r <name>
# Creating migrations/<timestamp>_<name>.up.sql
# Creating migrations/<timestamp>_<name>.down.sql
```

その後、マイグレーションを以下の通り実行する。

```bash
sqlx migrate run
# Applied <timestamp>-<name>
```

マイグレーションを戻す場合は以下を実行する。

```bash
sqlx migrate revert
# Applied <timestamp>/revert <name>
```

戻すことが不可能なマイグレーションと、戻すことが可能なマイグレーションを混在させると、エラーが発生する。

```bash
# 戻すことが不可能なマイグレーションを作成
sqlx migrate add <name1>
Creating migrations/20211001154420_<name>.sql

# 戻すことが可能なマイグレーションを作成(エラーが発生)
sqlx migrate add -r <name2>
error: cannot mix reversible migrations with simple migrations. All migrations should be reversible or simple migrations
```
## Actix Web

バージョン3の安定バージョンが`SeaORM`が依存している`tokio`とバージョンが合わないため、バージョン`4.0.0-rc.3`を使用している。

## SeaORM

### 準備

`sea-orm-cli`コマンドを以下の通りインストールする。

```bash
cargo install sea-orm-cli
```

### エンティティファイルの作成

データベースに存在するテーブルを表現するエンティティファイルを、以下の通り作成する。

```bash
sea-orm-cli generate entity -o infrastructures/src/postgres/schema
```

### Cargo.toml

注意: 以下で説明する方法で、開発環境に限定して`mock`を有効にできなかった。

`SeaORM`の`MockDatabaseConnection`が`Clone`を実装しない。
よって、`axum`の`Extension`でデータベースコネクションを状態として記録できない。
このため、`SeaORM`の`mock`フィーチャを、以下の通り開発環境のみで有効にする。

```toml
[dependencies.sea-orm]
version = "0.5"
features = ["sqlx-postgres", "runtime-tokio-rustls", "macros", "debug-print"]
default-features = false

[dev-dependencies.sea-orm]
version = "0.5"
features = ["sqlx-postgres", "runtime-tokio-rustls", "macros", "debug-print", "mock"]
default-features = false
```

## JWT

トークンはデータベースに蓄積され続けるため、定期的にデータベースに記録しているJWTトークンを削除する必要がある。
