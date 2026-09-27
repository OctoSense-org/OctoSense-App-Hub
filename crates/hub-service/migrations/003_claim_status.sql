-- A legacy app ID must be reviewed before it may be published by a new owner.
ALTER TABLE app_claims ADD COLUMN status TEXT NOT NULL DEFAULT 'review_required'
    CHECK (status IN ('active', 'review_required'));
