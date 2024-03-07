// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Int8,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
        message_id -> Nullable<Text>,
        project_id -> Nullable<Int8>,
        project_version -> Nullable<Text>,
        #[max_length = 4096]
        data -> Nullable<Varchar>,
        task_id -> Nullable<Text>,
    }
}

diesel::table! {
    proofs (id) {
        id -> Int4,
        image_id -> Varchar,
        private_input -> Varchar,
        public_input -> Varchar,
        receipt_type -> Varchar,
        receipt -> Nullable<Text>,
        status -> Varchar,
        create_at -> Timestamp,
    }
}

diesel::table! {
    task_state_logs (id) {
        id -> Int8,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
        task_id -> Nullable<Text>,
        state -> Int2,
        comment -> Nullable<Text>,
    }
}

diesel::table! {
    tasks (id) {
        id -> Int8,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
        task_id -> Nullable<Text>,
        message_id -> Nullable<Text>,
        project_id -> Nullable<Int8>,
        state -> Nullable<Int2>,
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

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    proofs,
    task_state_logs,
    tasks,
    vms,
);
