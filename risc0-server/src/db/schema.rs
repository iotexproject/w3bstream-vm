// @generated automatically by Diesel CLI.

diesel::table! {
    proofs (id) {
        id -> Int4,
        project_id -> Varchar,
        task_id -> Varchar,
        client_id -> Varchar,
        sequencer_sign -> Varchar,
        image_id -> Varchar,
        datas_input -> Varchar,
        receipt_type -> Varchar,
        receipt -> Nullable<Text>,
        status -> Varchar,
        create_at -> Timestamp,
    }
}

diesel::table! {
    vms (id) {
        id -> Int4,
        project_name -> Varchar,
        elf -> Text,
        image_id -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(proofs, vms,);
