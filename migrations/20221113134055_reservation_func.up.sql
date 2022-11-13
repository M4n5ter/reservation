-- if user_id is null, find all reservations within during for the resource
-- if resource_id is null, find all reservations within during for the user
-- if both are null, find all reservations within during
-- if both set, find all reservations within during for the resource and user
CREATE
 OR REPLACE FUNCTION rsvp.query ( uid TEXT, rid TEXT, during TSTZRANGE ) RETURNS TABLE (LIKE rsvp.reservations) AS $$
 BEGIN

 END;
 $$ LANGUAGE plpgsql;
