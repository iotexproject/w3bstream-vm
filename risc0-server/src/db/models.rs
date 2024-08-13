use crate::db::schema::proofs;
use crate::db::schema::vms;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = vms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Vm {
    pub id: i32,
    pub project_name: String,
    pub elf: String,
    pub image_id: String,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            id: 0,
            project_name: "init".to_string(),
            elf: "init".to_string(),
            image_id: "init".to_string(),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = vms)]
pub struct NewVm<'a> {
    pub project_name: &'a str,
    pub elf: &'a str,
    pub image_id: &'a str,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = proofs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Proof {
    pub id: i32,
    pub project_id: String,
    pub task_id: String,
    pub client_id: String,
    pub sequencer_sign: String,
    pub image_id: String,
    pub datas_input: String,
    pub receipt_type: String,
    pub receipt: Option<String>,
    pub status: String,
}

impl Proof {
    pub fn new() -> Self {
        Proof {
            id: 0,
            project_id: "0".to_string(),
            task_id: "0".to_string(),
            client_id: "init".to_string(),
            sequencer_sign: "init".to_string(),
            image_id: "init".to_string(),
            datas_input: "init".to_string(),
            receipt_type: "init".to_string(),
            receipt: Some("init".to_string()),
            status: "init".to_string(),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = proofs)]
pub struct NewPoof<'a> {
    pub project_id: &'a str,
    pub task_id: &'a str,
    pub client_id: &'a str,
    pub sequencer_sign: &'a str,
    pub image_id: &'a str,
    pub datas_input: &'a str,
    pub receipt_type: &'a str,
    pub status: &'a str,
}
