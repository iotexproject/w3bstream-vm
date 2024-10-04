use std::env;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    Connection, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;

use crate::db::models::{NewPoof, NewVm, Proof, Vm};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

pub fn run_migration(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");
}

pub fn create_vm<'a>(
    conn: &mut PgConnection,
    prj_name: &'a str,
    elf_str: &'a str,
    id_str: &'a str,
) -> Result<Vm, diesel::result::Error> {
    use crate::db::schema::vms;
    use crate::db::schema::vms::dsl::*;

    let new_vm = NewVm {
        project_name: prj_name,
        elf: elf_str,
        image_id: id_str,
    };

    match diesel::delete(vms.filter(project_name.eq(prj_name))).execute(conn) {
        Ok(_) => (),
        Err(err) => {
            return Err(err);
        }
    };

    diesel::insert_into(vms::table)
        .values(&new_vm)
        .returning(Vm::as_returning())
        .get_result(conn)
}

pub fn get_vm<'a>(conn: &mut PgConnection, id_str: &'a str) -> Result<Vm, diesel::result::Error> {
    use crate::db::schema::vms::dsl::*;

    // let results = vms.filter(image_id.eq(id_str)).limit(1).select(Vm::as_select()).load(conn).expect("Error loading vms");

    let vm = vms
        .filter(image_id.eq(id_str))
        .select(Vm::as_select())
        .first(conn)?;
    Ok(vm)
}

pub fn get_vm_by_project<'a>(
    conn: &mut PgConnection,
    project: &'a str,
) -> Result<Vm, diesel::result::Error> {
    use crate::db::schema::vms::dsl::*;

    // let results = vms.filter(image_id.eq(id_str)).limit(1).select(Vm::as_select()).load(conn).expect("Error loading vms");

    let vm = vms
        .filter(project_name.eq(project))
        .select(Vm::as_select())
        .first(conn)?;
    Ok(vm)
}

pub fn create_proof<'a>(
    conn: &mut PgConnection,
    project_id: &'a str,
    task_id: &'a str,
    client_id: &'a str,
    sequencer_sign: &'a str,
    image_id: &'a str,
    datas_input: &'a str,
    receipt_type: &'a str,
    status: &'a str,
) -> Proof {
    use crate::db::schema::proofs;

    let new_proof = NewPoof {
        project_id,
        task_id,
        client_id,
        sequencer_sign,
        image_id,
        datas_input,
        receipt_type,
        status,
    };

    diesel::insert_into(proofs::table)
        .values(&new_proof)
        .returning(Proof::as_returning())
        .get_result(conn)
        .expect("Error saving new proof")
}

pub fn update_proof_with_receipt<'a>(
    conn: &mut PgConnection,
    p: &'a Proof,
    r: &'a String,
) -> Proof {
    use crate::db::schema::proofs::dsl::*;

    diesel::update(proofs.filter(id.eq(p.id)))
        .set((receipt.eq(r), status.eq("succeeded")))
        .returning(Proof::as_returning())
        .get_result(conn)
        .expect("Error updating proof")
}

pub fn update_proof_status_with_receipt<'a>(
    conn: &mut PgConnection,
    p: &'a Proof,
    s: &'a String,
) -> Proof {
    use crate::db::schema::proofs::dsl::*;

    diesel::update(proofs.filter(id.eq(p.id)))
        .set(status.eq(s))
        .returning(Proof::as_returning())
        .get_result(conn)
        .expect("Error updating proof")
}
