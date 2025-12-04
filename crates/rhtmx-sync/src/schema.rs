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

diesel::table! {
    _rhtmx_field_sync_log (id) {
        id -> Int8,
        entity -> Varchar,
        entity_id -> Varchar,
        field -> Varchar,
        value -> Nullable<Text>,
        action -> Varchar,
        version -> Int8,
        client_id -> Nullable<Varchar>,
        timestamp -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    _rhtmx_sync_log,
    _rhtmx_field_sync_log,
);
