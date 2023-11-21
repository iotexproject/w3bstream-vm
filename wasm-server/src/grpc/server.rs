
use std::io::Read;
use std::{collections::HashMap, sync::Arc, i32::MAX};

use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use uuid::Uuid;
use flate2::read::ZlibDecoder;
use hex::FromHex;

use rust_grpc::grpc::vm_runtime::{vm_runtime_server::VmRuntime, CreateRequest, CreateResponse, ExecuteRequest, ExecuteResponse};

use crate::wasmtime::instance as wasm_instance;

pub struct WasmtimeGrpcServer {
    instances_map: Arc<Mutex<HashMap<u64, wasm_instance::Instance>>>,
}

impl Default for WasmtimeGrpcServer {
    fn default() -> Self {
        WasmtimeGrpcServer { instances_map: Arc::new(Mutex::new(HashMap::<u64, wasm_instance::Instance>::new())) }
    }
}

#[tonic::async_trait]
impl VmRuntime for WasmtimeGrpcServer {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<CreateResponse>, Status> {
        let request = request.into_inner();

        let project = request.project_id;
        let content = request.content;
        let compressed_data = Vec::from_hex(content).unwrap();
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut content = Vec::new();
        decoder.read_to_end(&mut content)?;
        let exp_param = request.exp_param;

        // exp_param = {"image_id":"RANGE_ID", "elf":"RANGE_ELF"}
        // if exp_param == "" {
        //     return Err(Status::invalid_argument("need exp_param"))
        // }

        let id = Uuid::new_v4();
        let instance = wasm_instance::new_instance_by_code(id, content).unwrap();

        let mut map = self.instances_map.lock().await;
        if let Some(_) = map.get(&project) {
            return Err(Status::already_exists("DUP_ITEM_ERR"));
        }

        map.insert(project, instance);
        
        Ok(Response::new(CreateResponse {}))
    }

    async fn execute_operator(&self, request: Request<ExecuteRequest>) -> Result<Response<ExecuteResponse>, Status> {
        let request = request.into_inner();

        let project = request.project_id;
        let param = request.param;

        if param == "" {
            return Err(Status::invalid_argument("need project and param"))
        }

        let mut map = self.instances_map.lock().await;
        let instance = match map.get_mut(&project) {
            Some(instance) => instance,
            None => return Err(Status::not_found("no project")),
        };

        let rid = (Uuid::new_v4().as_u128() % (MAX as u128)) as i32;
        {
            let mut res = instance.export_funcs.res.lock().unwrap();
            res.insert(rid, param.as_bytes().to_vec());
        }

        match instance.export_funcs.rt.instantiate() {
            Ok(_) => {},
            Err(_) => {
                let _ = instance.export_funcs.rt.drop_instantiate();
                return Err(Status::internal("vm runtime instantiate error"));
            },
        };

        let result = match instance.export_funcs.rt.call("start", rid) {
            Ok(result) => result,
            Err(_) => {
                let _ = instance.export_funcs.rt.drop_instantiate();
                return Err(Status::internal("vm runtime run call error"));
            },
        };
        
        Ok(Response::new(ExecuteResponse {result: result.to_le_bytes().to_vec()} ))
    }
}