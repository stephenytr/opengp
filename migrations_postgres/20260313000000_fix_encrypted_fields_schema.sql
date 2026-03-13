DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'patients'
          AND column_name = 'ihi'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE patients
            ALTER COLUMN ihi TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'patients'
          AND column_name = 'medicare_number'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE patients
            ALTER COLUMN medicare_number TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'consultations'
          AND column_name = 'soap_subjective'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE consultations
            ALTER COLUMN soap_subjective TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'consultations'
          AND column_name = 'soap_objective'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE consultations
            ALTER COLUMN soap_objective TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'consultations'
          AND column_name = 'soap_assessment'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE consultations
            ALTER COLUMN soap_assessment TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'consultations'
          AND column_name = 'soap_plan'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE consultations
            ALTER COLUMN soap_plan TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'consultations'
          AND column_name = 'clinical_notes'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE consultations
            ALTER COLUMN clinical_notes TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'social_history'
          AND column_name = 'notes'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE social_history
            ALTER COLUMN notes TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'allergies'
          AND column_name = 'reaction'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE allergies
            ALTER COLUMN reaction TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'allergies'
          AND column_name = 'notes'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE allergies
            ALTER COLUMN notes TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'medical_history'
          AND column_name = 'notes'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE medical_history
            ALTER COLUMN notes TYPE BYTEA
            USING NULL::BYTEA;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'family_history'
          AND column_name = 'notes'
          AND udt_name <> 'bytea'
    ) THEN
        ALTER TABLE family_history
            ALTER COLUMN notes TYPE BYTEA
            USING NULL::BYTEA;
    END IF;
END;
$$;
