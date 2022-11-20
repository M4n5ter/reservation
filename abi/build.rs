use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        // .type_attribute("reservation.ReservationStatus", "#[derive(sqlx::Type)]")
        // .type_attribute("reservation.Reservation", "#[derive(sqlx::FromRow)]")
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo-fmt").output().unwrap();

    println!("cargo:rerun-if-changed=protos/reservation.proto");
}
