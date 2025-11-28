// @generated automatically by Diesel CLI.

diesel::table! {
    _rhtmx_sync_log (id) {
        id -> Int8,
        entity -> Varchar,
        entity_id -> Varchar,
        action -> Varchar,
        data -> Nullable<Text>,
        version -> Int8,
        client_id -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}
