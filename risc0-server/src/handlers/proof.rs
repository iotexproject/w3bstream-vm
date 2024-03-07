use std::env;
use dotenvy::dotenv;
use anyhow::Result;

use crate::{
    core::prove::{bonsai_prove, generate_proof_with_elf, RiscReceipt},
    db::{self},
    model::models::ProofType,
    tools::parse_elf_from_str,
};

// pub fn async_generate_proof(
//     template_name: &String,
//     image_id: &String,
//     private_input: &String,
//     public_input: &String,
//     receipt_type: &ProofType,
// ) -> Result<String, String> {
//     let proof = get_proof_from_db(image_id, private_input, public_input, &receipt_type.to_string());
//     match proof {
//         Ok(p) => {
//             if let Some(_) = p.receipt {
//                 return Ok(String::from("receipt exist"));
//             } else {
//                 return Ok(String::from("generating proof, please try later."));
//             }
//         }
//         Err(e) => {
//             println!("image_id and param not exist: {}", e);
//             let template_name_clone = template_name.clone();
//             let image_id_clone = image_id.clone();
//             let private_clone = private_input.clone();
//             let pub_clone = public_input.clone();
//             let re_clone = receipt_type.clone();
//             tokio::task::spawn(get_receipt(
//                     "name".to_string(),
//                     template_name_clone,
//                     image_id_clone,
//                     private_clone,
//                     pub_clone,
//                     re_clone,
//                 )
//             );
//             return Ok(String::from("generating proof"));
//         }
//     }
// }

// pub fn validate_generate_proof_params(
//     pri_input: &String,
//     pub_input: &String,
// ) -> Result<bool, String> {
//     if pri_input.chars().count() == 0 {
//         return Err(String::from("private input can't be null."));
//     }

//     let pri_vec: Result<Vec<u64>, _> = pri_input
//         .split(",")
//         .map(|s| s.trim().parse::<u64>())
//         .collect();
//     match pri_vec {
//         Ok(v) => v,
//         Err(e) => {
//             println!("private input parse error, Error: {:?}", e);
//             return Err(format!("private input parse error, Error: {:?}", e));
//         }
//     };

//     if pub_input.chars().count() > 0 {
//         let pub_ver: Result<Vec<u64>, _> = pub_input
//             .split(",")
//             .map(|s| s.trim().parse::<u64>())
//             .collect();
//         match pub_ver {
//             Ok(v) => v,
//             Err(e) => {
//                 println!("public input parse error, Error: {:?}", e);
//                 return Err(format!("public input parse error, Error: {:?}", e));
//             }
//         };
//     }
//     Ok(true)
// }

// pub fn get_proof_from_db(
//     image_id: &String,
//     private_input: &String,
//     public_input: &String,
//     receipt_type: &String,
// ) -> Result<Proof, diesel::result::Error> {
//     let connection = &mut db::pgdb::establish_connection();
//     let proof = db::pgdb::get_receipt_with_id_params(
//         connection,
//         image_id,
//         private_input,
//         public_input,
//         &receipt_type,
//     );
//     proof
// }

pub async fn get_receipt(
    image_id: String,
    input_datas: String,
    receipt_type: ProofType,
) -> Result<String, String> {
    let connection = &mut db::pgdb::establish_connection();
    let proof = db::pgdb::create_proof(
        connection,
        &image_id,
        &input_datas,
        "public_input",
        &receipt_type.to_string(),
        "generating",
    );
    let mut vm = db::models::Vm::new();
    let vm_result = db::pgdb::get_vm(connection, &image_id);
    match vm_result {
        Ok(v) => vm = v,
        Err(e) => println!("image_id parse error: {}", e),
    }
    let elf_cont: Vec<u8> = parse_elf_from_str(&vm.elf);

    let receipt: Result<RiscReceipt>;
    match receipt_type {
        ProofType::Stark => {
            receipt = generate_proof_with_elf(&input_datas, &elf_cont);
        }
        ProofType::Snark => {
            dotenv().ok();

            let bonsai_url = env::var("BONSAI_URL").expect("BONSAI_URL must be set");
            let bonsai_key = env::var("BONSAI_KEY").expect("BONSAI_KEY must be set");
            // TODO
            receipt = tokio::task::spawn_blocking(move || { 
                bonsai_prove(
                    &input_datas,
                    &elf_cont,
                    bonsai_url,
                    bonsai_key,
               )
            }).await.unwrap();
        }
    }

    match receipt {
        Ok(r) => {
            let receipt_str = serde_json::to_string(&r).unwrap();
            let _ = db::pgdb::update_proof_with_receipt(connection, &proof, &receipt_str);
            return Ok(receipt_str);
        }
        Err(e) => {
            println!("generate proof error, Error: {:?}", e);
            let _ = db::pgdb::update_proof_status_with_receipt(
                connection,
                &proof,
                &"failed".to_string(),
            );
            return Err(format!("generate proof error, Error: {:?}", e));
        }
    }
}

// websocket
// pub async fn handle_socket(mut socket: WebSocket) {
//     while let Some(msg) = socket.recv().await {
//         if let Ok(msg) = msg {
//             let (receipt, control) = process_message(msg).await;
//             if control.is_break() {
//                 return;
//             }
//             if let Some(r) = receipt {
//                 if socket
//                     .send(Message::Text(r))
//                     .await
//                     .is_err() {
//                     println!("client disconnected");
//                 }
//             } else {
//                 println!("No name");
//             }
//         } else {
//             println!("client abruptly disconnected");
//             return;
//         }
//     }
// }

// async fn process_message(msg: Message) -> (Option<String>, ControlFlow<(), ()>) {
//     match msg {
//         Message::Text(t) => {
//             println!("Server got message {}. ", t);

//             let v: Value = serde_json::from_str(&t).unwrap();

//             // TODO validate
//             let id_str = v["image_id"].as_str().unwrap().to_string();
//             let private_input = v["private_input"].as_str().unwrap().to_string();
//             let public_input = v["public_input"].as_str().unwrap().to_string();
//             let receipt_type: Result<ProofType, _> = v["receipt_type"].as_str().unwrap().to_string().parse();
//             let receipt_type = receipt_type.unwrap();

//             let proof = get_proof_from_db(
//                 &id_str,
//                 &private_input,
//                 &public_input,
//                 &receipt_type.to_string()
//             );

//             let receipt = match proof {
//                 Ok(p) => {
//                     if let Some(r) = p.receipt {
//                         println!("receipt exist");
//                         r
//                     } else {
//                         String::from("generating proof, please try later.")
//                     }
//                 }
//                 Err(e) => {
//                     println!("image_id and param not exist: {}", e);
//                     get_receipt(
//                         id_str,
//                         private_input,
//                         public_input,
//                         receipt_type,
//                     ).await.unwrap_or(String::from("get proof error"))
//                 }
//             };

//             return (Some(receipt), ControlFlow::Continue(()));
//         }
//         Message::Binary(d) => {
//             println!(">>> {} sent {} bytes: {:?}", "who", d.len(), d);
//             let receipt = String::from("not support bytes");
//             return (Some(receipt), ControlFlow::Continue(()));
//         }
//         Message::Close(c) => {
//             if let Some(cf) = c {
//                 println!(
//                     ">>> {} sent close with code {} and reason `{}`",
//                     "who", cf.code, cf.reason
//                 );
//             } else {
//                 println!(
//                     ">>> {} somehow sent close message without CloseFrame",
//                     "who"
//                 );
//             }
//             return (None, ControlFlow::Break(()));
//         }
//         Message::Pong(v) => {
//             println!(">>> {} sent pong with {:?}", "who", v);
//         }
//         // You should never need to manually handle Message::Ping, as axum's websocket library
//         // will do so for you automagically by replying with Pong and copying the v according to
//         // spec. But if you need the contents of the pings you can see them here.
//         Message::Ping(v) => {
//             println!(">>> {} sent ping with {:?}", "who", v);
//         }
//     }
//     (None, ControlFlow::Continue(()))
// }

