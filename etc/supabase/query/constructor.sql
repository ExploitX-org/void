DROP TYPE IF EXISTS registration_type CASCADE;
DROP TYPE IF EXISTS gender CASCADE;
DROP TYPE IF EXISTS payment_provider CASCADE;
DROP TYPE IF EXISTS payment_status CASCADE;
DROP TYPE IF EXISTS verification_target CASCADE;
DROP TYPE IF EXISTS team_state CASCADE;

ALTER DATABASE postgres SET timezone = 'Asia/Kolkata';
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SEQUENCE IF NOT EXISTS invoice_seq START 1;

DO $$ BEGIN
  CREATE TYPE registration_type AS ENUM ('single', 'couple');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE gender AS ENUM ('male', 'female', 'trans', 'others');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE payment_provider AS ENUM ('cashfree', 'razorpay');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE payment_status AS ENUM ('initiated', 'completed', 'failed', 'refunded');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE verification_target AS ENUM ('leader', 'member');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE team_state AS ENUM ('pending_verification', 'verified', 'payment_pending', 'paid', 'failed');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS teams (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registration_type   registration_type NOT NULL,
    team_name           TEXT NOT NULL,
    state               team_state NOT NULL DEFAULT 'pending_verification',
    base_price_paise    INT,
    final_price_paise   INT,
    discount_code       TEXT,
    resend_attempts     INT NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS leaders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id         UUID NOT NULL UNIQUE REFERENCES teams(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    gender          gender NOT NULL,
    email           TEXT NOT NULL UNIQUE,
    mobile_cc       TEXT NOT NULL DEFAULT '+91',
    mobile_number   TEXT NOT NULL,
    college_name    TEXT NOT NULL,
    degree          TEXT NOT NULL,
    department      TEXT NOT NULL,
    year_of_study   INT NOT NULL CHECK (year_of_study BETWEEN 1 AND 7),
    location        TEXT NOT NULL,
    UNIQUE (mobile_cc, mobile_number)
);

CREATE TABLE IF NOT EXISTS members (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id         UUID NOT NULL UNIQUE REFERENCES teams(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    gender          gender NOT NULL,
    email           TEXT NOT NULL UNIQUE,
    mobile_cc       TEXT NOT NULL DEFAULT '+91',
    mobile_number   TEXT NOT NULL,
    college_name    TEXT NOT NULL,
    degree          TEXT NOT NULL,
    department      TEXT NOT NULL,
    year_of_study   INT NOT NULL CHECK (year_of_study BETWEEN 1 AND 7),
    location        TEXT NOT NULL,
    UNIQUE (mobile_cc, mobile_number)
);

CREATE TABLE IF NOT EXISTS verifications (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id         UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    target          verification_target NOT NULL,
    email           TEXT NOT NULL,
    otp_hash        TEXT,
    otp_expires_at  TIMESTAMPTZ,
    otp_attempts    INT NOT NULL DEFAULT 0,
    verified        BOOLEAN NOT NULL DEFAULT FALSE,
    verified_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (team_id, target)
);

CREATE TABLE IF NOT EXISTS payments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id             UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    provider            payment_provider NOT NULL,
    provider_order_id   TEXT NOT NULL,
    amount              INT NOT NULL,
    currency            TEXT NOT NULL DEFAULT 'INR',
    status              payment_status NOT NULL DEFAULT 'initiated',
    payment_link        TEXT NOT NULL DEFAULT '',
    ordered_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at        TIMESTAMPTZ,
    UNIQUE (provider, provider_order_id)
);

CREATE TABLE IF NOT EXISTS razorpay_payment_details (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id        UUID NOT NULL UNIQUE REFERENCES payments(id) ON DELETE CASCADE,
    rzp_payment_id    TEXT,
    rzp_signature     TEXT
);

CREATE TABLE IF NOT EXISTS cashfree_payment_details (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_id            UUID NOT NULL UNIQUE REFERENCES payments(id) ON DELETE CASCADE,
    cf_order_id           TEXT NOT NULL,
    payment_session_id    TEXT
);

CREATE TABLE IF NOT EXISTS invoices (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id       UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    payment_id    UUID NOT NULL UNIQUE REFERENCES payments(id) ON DELETE CASCADE,
    invoice_no    TEXT NOT NULL UNIQUE,
    pdf           BYTEA,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

INSERT INTO settings (key, value) VALUES ('registration_open', 'true')
ON CONFLICT (key) DO NOTHING;

CREATE UNIQUE INDEX IF NOT EXISTS idx_teams_name_lower      ON teams (LOWER(team_name));
CREATE INDEX IF NOT EXISTS idx_leaders_team_id              ON leaders(team_id);
CREATE INDEX IF NOT EXISTS idx_members_team_id              ON members(team_id);
CREATE INDEX IF NOT EXISTS idx_verifications_team_id        ON verifications(team_id);
CREATE INDEX IF NOT EXISTS idx_verifications_email          ON verifications(email);
CREATE INDEX IF NOT EXISTS idx_payments_team_id             ON payments(team_id);
CREATE INDEX IF NOT EXISTS idx_payments_provider_order      ON payments(provider, provider_order_id);
CREATE INDEX IF NOT EXISTS idx_invoices_team_id             ON invoices(team_id);
CREATE INDEX IF NOT EXISTS idx_invoices_payment_id          ON invoices(payment_id);
