use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .type_attribute(
            "reservation.ReservationQuery",
            "#[derive(derive_builder::Builder)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.resource_id",
            "#[builder(setter(into), default)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.user_id",
            "#[builder(setter(into), default)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.status",
            "#[builder(setter(into), default)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.desc",
            "#[builder(setter(into), default)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.page",
            "#[builder(setter(into), default)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.page_size",
            "#[builder(setter(into), default)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.start",
            "#[builder(setter(into, strip_option), default)]",
        )
        .field_attribute(
            "reservation.ReservationQuery.end",
            "#[builder(setter(into, strip_option), default)]",
        )
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo-fmt").output().unwrap();

    println!("cargo:rerun-if-changed=protos/reservation.proto");
}
