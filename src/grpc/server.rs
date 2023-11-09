use std::{io::Write, collections::HashMap, sync::Arc, os::raw::c_char, ffi::{CString, CStr}};

use libloading::{Library, Symbol};
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
            instance = match Library::new(file.path().to_str().unwrap()) {
                Ok(ins) => ins,
                Err(e) => {
                    println!("instance error : {:?}", e);
                    return Err(Status::internal("please use xx.so"))
                },
            };
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

        if project == "" || param == "" {
            return Err(Status::invalid_argument("need project and param"))
        }

        let mut map = self.instances_map.lock().await;
        let instance = match map.get_mut(&project) {
            Some(instance) => instance,
            None => return Err(Status::not_found("no project")),
        };

        let receipt: String;
        unsafe {
            let lib_func: Symbol<unsafe extern fn(*const c_char) -> *mut c_char> = instance.get(b"w3b_prove").unwrap();
            let input = CString::new(param).unwrap();
            // TODO catch painc
            let output = lib_func(input.as_ptr());
            receipt = CStr::from_ptr(output).to_str().unwrap().to_owned();
        }

        Ok(Response::new(ExecuteResponse {result: receipt.as_bytes().to_vec()} ))
    }
}