-- resevation change queue
CREATE TABLE rsvp.reservation_changes ( ID SERIAL NOT NULL, reservation_id UUID NOT NULL, op rsvp.reservation_update_type NOT NULL );
-- trigger for add/update/delete a reservation
CREATE
 OR REPLACE FUNCTION rsvp.reservations_trigger ( ) RETURNS TRIGGER AS $$ BEGIN
 IF
   -- update reservation_changes
  TG_OP = 'INSERT' THEN
   INSERT INTO rsvp.reservation_changes ( reservation_id, op )
  VALUES
   ( NEW.ID, 'create' );
   -- if status changed, update reservation_changes
  ELSIF TG_OP = 'UPDATE' THEN
   IF
    OLD.status <> NEW.status THEN
     INSERT INTO rsvp.reservation_changes ( reservation_id, op )
    VALUES
     ( NEW.ID, 'update' );

   END IF;
   -- update reservation_changes
   ELSIF TG_OP = 'DELETE' THEN
    INSERT INTO rsvp.reservation_changes ( reservation_id, op )
    VALUES
     ( OLD.ID, 'delete' );

   END IF;
   -- notify a channel called reservation_update
   NOTIFY reservation_update;
   RETURN NULL;

  END;
  $$ LANGUAGE plpgsql;
  CREATE TRIGGER reservations_trigger AFTER INSERT
  OR UPDATE
  OR DELETE ON rsvp.reservations FOR EACH ROW
 EXECUTE FUNCTION rsvp.reservations_trigger();
