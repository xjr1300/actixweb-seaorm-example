-- 都道府県テーブル作成。
CREATE TABLE prefectures(
    -- 都道府県コード。
    code SMALLINT NOT NULL,
    -- 都道府県名。
    name VARCHAR(10) NOT NULL,
    -- 主キー制約。
    PRIMARY KEY (code)
);
-- 都道府県テーブル行挿入。
INSERT INTO prefectures
VALUES (1, '北海道'),
    (2, '青森県'),
    (3, '岩手県'),
    (4, '宮城県'),
    (5, '秋田県'),
    (6, '山形県'),
    (7, '福島県'),
    (8, '茨城県'),
    (9, '栃木県'),
    (10, '群馬県'),
    (11, '埼玉県'),
    (12, '千葉県'),
    (13, '東京都'),
    (14, '神奈川県'),
    (15, '新潟県'),
    (16, '富山県'),
    (17, '石川県'),
    (18, '福井県'),
    (19, '山梨県'),
    (20, '長野県'),
    (21, '岐阜県'),
    (22, '静岡県'),
    (23, '愛知県'),
    (24, '三重県'),
    (25, '滋賀県'),
    (26, '京都府'),
    (27, '大阪府'),
    (28, '兵庫県'),
    (29, '奈良県'),
    (30, '和歌山県'),
    (31, '鳥取県'),
    (32, '島根県'),
    (33, '岡山県'),
    (34, '広島県'),
    (35, '山口県'),
    (36, '徳島県'),
    (37, '香川県'),
    (38, '愛媛県'),
    (39, '高知県'),
    (40, '福岡県'),
    (41, '佐賀県'),
    (42, '長崎県'),
    (43, '熊本県'),
    (44, '大分県'),
    (45, '宮崎県'),
    (46, '鹿児島県'),
    (47, '沖縄県');
-- アカウントテーブル作成。
CREATE TABLE accounts (
    -- アカウントID。
    id CHAR(26) NOT NULL,
    -- Eメールアドレス。
    email VARCHAR(256) NOT NULL,
    -- アカウント名。
    name VARCHAR(20) NOT NULL,
    -- ハッシュ化したパスワード。
    password VARCHAR(512) NOT NULL,
    -- アクティブフラグ。
    is_active BOOLEAN NOT NULL,
    -- 固定電話番号。
    fixed_number VARCHAR(20),
    -- 携帯電話番号。
    mobile_number VARCHAR(20),
    -- 郵便番号。
    postal_code CHAR(8) NOT NULL,
    -- 都道府県コード。
    prefecture_code SMALLINT NOT NULL,
    -- 市区町村以下住所。
    address_details VARCHAR(100) NOT NULL,
    -- 最終ログイン日時。
    logged_in_at TIMESTAMP WITh TIME ZONE,
    -- 登録日時。
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    -- 更新日時。
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    -- 主キー制約。
    PRIMARY KEY (id)
);
-- Eメールアドレスユニークインデックス作成。
CREATE UNIQUE INDEX IF NOT EXISTS accounts_email_index ON accounts (email);
-- 都道府県コード外部参照制約。
ALTER TABLE accounts
ADD CONSTRAINT accounts_prefecture_code_to_prefectures FOREIGN KEY (prefecture_code) REFERENCES prefectures (code) ON DELETE RESTRICT;
-- JWTトークンテーブル作成。
CREATE TABLE jwt_tokens (
    -- ID。
    id CHAR(26) NOT NULL,
    -- アカウントID。
    account_id CHAR(26) NOT NULL,
    -- アクセストークン。
    access VARCHAR(8192) NOT NULL,
    -- アクセストークン有効期限。
    access_expired_at TIMESTAMP WITH TIME ZONE NOT NULL,
    -- リフレッシュトークン。
    refresh VARCHAR(8192) NOT NULL,
    -- リフレッシュトークン有効期限。
    refresh_expired_at TIMESTAMP WITH TIME ZONE NOT NULL,
    -- 主キー制約。
    PRIMARY KEY (id)
);
-- アカウントID外部参照制約。
ALTER TABLE jwt_tokens
ADD CONSTRAINT jwt_tokens_id_to_accounts FOREIGN KEY (account_id) REFERENCES accounts (id) ON DELETE CASCADE;
-- JWTトークンテーブルインデックス
CREATE UNIQUE INDEX IF NOT EXISTS jwt_tokens_access_index ON jwt_tokens (access);
CREATE UNIQUE INDEX IF NOT EXISTS jwt_tokens_refresh_index ON jwt_tokens (refresh);
