-- if user_id is null, find all reservations within during for the resource.
-- if resource_id is null, find all reservations within during for the user.
-- if both are null, find all reservations within during.
-- if both are set, find all reservations within during for the user and resource.
CREATE OR REPLACE FUNCTION rsvp.query(
    uid TEXT,
    rid TEXT,
    during TSTZRANGE,
    status rsvp.reservation_status DEFAULT 'confirmed',
    page INTEGER DEFAULT 1,
    is_desc BOOL DEFAULT FALSE,
    page_size INTEGER DEFAULT 10
) RETURNS TABLE (LIKE rsvp.reservations) AS $$
DECLARE
    _sql TEXT;
BEGIN
    -- format the query based on the parameters
    _sql := format('SELECT * FROM rsvp.reservations WHERE %L @> timespan AND %s AND %s ORDER BY lower(timespan) %s LIMIT %L::INTEGER OFFSET %L::INTEGER',
    during,
    CASE WHEN status = 'unknown' THEN 'TRUE' ELSE 'status = ' || quote_literal(status) END,
    CASE
        WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
        WHEN uid IS NULL THEN 'resource_id = ' || quote_literal(rid)
        WHEN rid IS NULL THEN 'user_id = ' || quote_literal(uid)
        ELSE 'user_id = ' || quote_literal(uid) || ' AND resource_id = ' || quote_literal(rid)
    END,
    CASE
        WHEN is_desc THEN 'DESC'
        ELSE 'ASC'
    END,
    page_size,
    (page - 1) * page_size
    );
    -- log the sql
    RAISE NOTICE '%', _sql;
    -- execute the query
    RETURN QUERY EXECUTE _sql;

END;
$$ LANGUAGE plpgsql;
