use std::{io::Write, collections::HashMap, sync::Arc};

use libloading::Library;
use tempfile::NamedTempFile;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use super::vm_runtime::{vm_runtime_server::VmRuntime, CreateRequest, CreateResponse, ExecuteResponse, ExecuteRequest};


#[derive(Debug)]
pub struct Halo2GrpcServer {
    instances_map: Arc<Mutex<HashMap<String, Library>>>,
}

impl Default for Halo2GrpcServer {
    fn default() -> Self {
        Halo2GrpcServer { instances_map: Arc::new(Mutex::new(HashMap::<String, Library>::new())) }
    }
}

#[tonic::async_trait]
impl VmRuntime for Halo2GrpcServer {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>, Status> {
        let request = request.into_inner();

        let project = request.project;
        let content: Vec<u8> = request.content;
        let _exp_param = request.exp_param;

        let mut file = NamedTempFile::new().unwrap();
        match file.write_all(&content) {
            Ok(_) => {},
            Err(e) => return Err(Status::internal(format!("write temp file error: {}", e))),
        };

        let instance: Library;
        unsafe {
            instance = Library::new(file.path().to_str().unwrap()).unwrap();
        }

        let mut map = self.instances_map.lock().await;
        if let Some(_) = map.get(&project) {
            return Err(Status::already_exists("DUP_ITEM_ERR"));
        }

        map.insert(project, instance);

        Ok(Response::new(CreateResponse {instance_id: Uuid::new_v4().to_string()}))
    }

    async fn execute_operator(&self, request: Request<ExecuteRequest>) -> Result<Response<ExecuteResponse>, Status> {
        let request = request.into_inner();

        let project = request.project;
        let param = request.param;

        if param == "" {
            return Err(Status::invalid_argument("need param"))
        }

        // let connection = &mut db::pgdb::establish_connection();
        // let mut vm = db::models::Vm::new();
        // let vm_result = db::pgdb::get_vm_by_project(connection, &project);
        // match vm_result {
        //     Ok(v) => vm = v,
        //     Err(e) => return Err(Status::not_found(format!("{} not found", project))),
        // };

        // // param = {"private_input":"14", "public_input":"3,34", "receipt_type":"Snark"}
        // let v: Value = serde_json::from_str(&param).unwrap();
        // let private_input = v["private_input"].as_str().unwrap().to_string();
        // let public_input = v["public_input"].as_str().unwrap().to_string();
        // let receipt_type: Result<ProofType, _> = v["receipt_type"].as_str().unwrap().to_string().parse();
        // let receipt_type = receipt_type.unwrap();

        // let receipt = get_receipt(
        //     "name".to_string(),
        //     "w3b".to_string(),
        //     vm.image_id,
        //     private_input,
        //     public_input,
        //     receipt_type,
        // ).await.unwrap_or(String::from("get proof error"));
        
        Ok(Response::new(ExecuteResponse {result: "receipt".as_bytes().to_vec()} ))
    }
}