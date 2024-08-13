use std::{fs, sync::RwLock};

use rust_grpc::grpc::vm_runtime::{vm_runtime_client::VmRuntimeClient, CreateRequest, ExecuteRequest};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tonic::{transport::Channel, Request};
use lazy_static::lazy_static;

use crate::tests::mock::Server;

mod mock;

lazy_static! {
    static ref SERVER: RwLock<Server> = RwLock::new(Server::new());
}

async fn init_real_server() {
    SERVER.write().unwrap().init_server().await;
}

#[tokio::test]
async fn test_create_and_execute_e2e() {
    init_real_server().await;

    // create client
    let channel = Channel::from_static("http://127.0.0.1:14003")
        .connect()
        .await
        .unwrap();
    let mut client = VmRuntimeClient::new(channel);

    // create vm instance
    let file_content = fs::read_to_string("./src/tests/10003.json").unwrap();
    let v: Value = serde_json::from_str(&file_content).unwrap();
    let content = v["code"].as_str().unwrap().to_string();
    let project_id: u64 = 10003;
    let exp_params = vec!("".to_string());

    let req = Request::new(CreateRequest {
        project_id,
        content,
        exp_params,
    });

    
    let response = client.create(req).await;
    match response {
        Ok(_) => assert!(true),
        Err(err) => {
            println!("failed to create vm instance, {:?}", err);
            assert!(false)
        }
    }

    // generate a proof
    let req = Request::new(ExecuteRequest { 
        project_id, 
        task_id: 0u64,
        client_id: "test_client_id".to_string(),
        sequencer_signature: "test_sequencer_sign".to_string(),
        datas: vec!["wasm log example".to_string()],
    });
    let response = client.execute(req).await;
    match response {
        Ok(_) => assert!(true),
        Err(err) => {
            println!("failed to executor vm instance, {:?}", err);
            assert!(false)
        }
    }
}

#[tokio::test]
async fn test_create_repeat_e2e() {
    init_real_server().await;

    // create client
    let channel = Channel::from_static("http://127.0.0.1:14003")
        .connect()
        .await
        .unwrap();
    let mut client = VmRuntimeClient::new(channel);

    // create vm instance
    let file_content = fs::read_to_string("./src/tests/10003.json").unwrap();
    let v: Value = serde_json::from_str(&file_content).unwrap();
    let content = v["code"].as_str().unwrap().to_string();
    let exp_params = vec!("".to_string());
    let project_id: u64 = 10003;

    let req = Request::new(CreateRequest {
        project_id,
        content: content.clone(),
        exp_params: exp_params.clone(),
    });

    
    let response = client.create(req).await;
    match response {
        Ok(_) => assert!(true),
        Err(err) => {
            println!("{}", err);
            assert!(false)
        },
    };

    let req_repeat = Request::new(CreateRequest {
        project_id,
        content: content.clone(),
        exp_params: exp_params.clone(), 
    });
    let response = client.create(req_repeat).await;
    match response {
        Ok(_) => (),
        Err(err) => {
            println!("{}", err);
            assert_eq!("DUP_ITEM_ERR", err.message());
        }
    }

}

#[tokio::test]
async fn test_executor_failed_e2e() {
    init_real_server().await;

        // create client
        let channel = Channel::from_static("http://127.0.0.1:14003")
        .connect()
        .await
        .unwrap();
    let mut client = VmRuntimeClient::new(channel);

    // create vm instance
    let file_content = fs::read_to_string("./src/tests/10003.json").unwrap();
    let v: Value = serde_json::from_str(&file_content).unwrap();
    let content = v["code"].as_str().unwrap().to_string();
    let exp_params = vec!("".to_string());
    let project_id: u64 = 10000;

    let req = Request::new(CreateRequest {
        project_id,
        content,
        exp_params,
    });

    
    let response = client.create(req).await;
    match response {
        Ok(_) => assert!(true),
        Err(err) => {
            println!("failed to create vm instance, {:?}", err);
            assert!(false)
        }
    }

    // datas is nil
    let req = Request::new(ExecuteRequest { 
        project_id, 
        task_id: 0u64,
        client_id: "test_client_id".to_string(),
        sequencer_signature: "test_sequencer_sign".to_string(),
        datas: vec![],
    });
    let response = client.execute(req).await;
    match response {
        Ok(_) => (),
        Err(err) => {
            assert_eq!("need datas", err.message())
        }
    }

    // project not found
    let req = Request::new(ExecuteRequest { 
        project_id: 99999, 
        task_id: 0u64,
        client_id: "test_client_id".to_string(),
        sequencer_signature: "test_sequencer_sign".to_string(),
        datas: vec!["wasm log example".to_string()],
    });
    let response = client.execute(req).await;
    match response {
        Ok(_) => (),
        Err(err) => {
            assert_eq!("no project", err.message())
        }
    }
}
