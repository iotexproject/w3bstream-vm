use std::{collections::HashMap, ptr::copy, sync::{Arc, Mutex}};

use anyhow::Result;
use wasmtime::{Caller, Extern, AsContextMut, Engine, Linker, Store, Module, Instance as wasmInstance};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

pub struct Runtime {
    engin: Option<Engine>,
    module: Option<Module>,
    linker: Option<Linker<WasiCtx>>,
    store: Option<Store<WasiCtx>>,
    instance: Option<wasmInstance>,
}

impl Default for Runtime {
    fn default() -> Self {
        let engine = Engine::default();
        let linker = Linker::new(&engine);
        Self { engin: Some(engine), module: None, linker: Some(linker), store: None, instance: None }
    }
}

impl Runtime {
    fn creat(&mut self, code: Vec<u8>) -> Result<()> {
        let module = Module::new(&self.engin.as_mut().unwrap(), code)?;
        self.module = Some(module);

        wasmtime_wasi::add_to_linker(&mut self.linker.as_mut().unwrap(), |s| s).unwrap();

        return Ok(());
    }

    pub fn instantiate(&mut self) -> Result<()> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args().unwrap()
            .build();
        let store = Store::new(&self.engin.as_mut().unwrap(), wasi);
        self.store = Some(store);

        let instance = self.linker.as_mut().unwrap().instantiate(self.store.as_mut().unwrap(), &self.module.as_mut().unwrap())?;
        self.instance = Some(instance);

        return Ok(());
    }

    pub fn drop_instantiate(&mut self) -> Result<()> {
        self.instance = None;
        self.store = None;
        return Ok(());
    }

    pub fn call(&mut self, name: &str, rid: i32) -> Result<i32> {

        // self.instance.as_ref().unwrap().get_func(self.store.as_mut().unwrap(), name);

        let start_func = self.instance.as_ref().unwrap().get_typed_func::<i32, i32>(self.store.as_mut().unwrap(), name)?;
        let result = start_func.call(self.store.as_mut().unwrap(), rid);
        
        return result;
    }
}

trait ABILinker {
    fn link_abi(&mut self);
}

pub struct ExportFuncs {
    pub rt: Runtime,
    pub res: Arc<Mutex<HashMap<i32, Vec<u8>>>>,
}

impl ABILinker for ExportFuncs {
    fn link_abi(&mut self) {
        // let env_map: HashMap<String, Box<dyn Fn()>> = [
        //     ("ws_log", Box::new(ExportFuncs::log)), 
        //     ("ws_get_data", Box::new(ExportFuncs::get_data)),
        // ].iter().cloned().collect();
        // for (key, value) in &env_map {
        //     self.rt.linker.unwrap().func_wrap("env", key, value);
        // }

        // self.rt.linker.unwrap().func_wrap("env", "ws_log", ExportFuncs::log);
        // self.rt.linker.unwrap().func_wrap("env", "ws_get_data", ExportFuncs::get_data);

        let res = Arc::clone(&self.res);

        self.rt.linker.as_mut().unwrap().func_wrap("env", "ws_log", ExportFuncs::log).unwrap();
        // self.rt.linker.as_mut().unwrap().func_wrap("env", "ws_get_data", ExportFuncs::get_data).unwrap();
        self.rt.linker.as_mut().unwrap().func_wrap("env", "ws_get_data", move |mut caller: Caller<'_, WasiCtx>, rid: i32, vm_add_ptr: i32, vm_size_ptr: i32| -> i32 {
            println!("log: rid {}", rid);
            println!("log: add_ptr {}", vm_add_ptr);
            println!("log: size_ptr {}", vm_size_ptr);

            // let handle = thread::spawn(|| { res.lock().await };
            // let res = handle.join();
            // let mut data: Vec<u8> = Vec::new();
            // async {
                let res = res.lock().unwrap();
                let data = res.get(&rid).unwrap().clone();
            // };
            
            // let data = String::from("The input string").as_bytes().to_vec();
        
            let len = data.len();
    
            let alloc_func = match caller.get_export("alloc") {
                Some(Extern::Func(func)) => func,
                _ => {
                    println!("expected a function export named 'alloc'");
                    return 1;
                },
            };
            let alloc_func_typed = match alloc_func.typed::<i32, i32>(&caller) {
                Ok(func) => func,
                Err(_) => {
                    println!("function 'alloc' has a wrong type");
                    return 1;
                },
            };
    
            let mem_ptr = alloc_func_typed.call(&mut caller.as_context_mut(), len.try_into().unwrap()).unwrap();
    
            let memory = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => {
                    println!("failed to find host memory");
                    return 1;
                }
            };
    
            let pointer = unsafe { memory.data_ptr(&caller).add(mem_ptr as usize) };
            println!("pointer is {:?}", pointer);
            unsafe {
                copy(data.as_ptr(), pointer, len);
            }
                       
            let offset = vm_add_ptr as u32 as usize;
            let addr_as_bytes: [u8; std::mem::size_of::<usize>()] = unsafe {
                std::mem::transmute(mem_ptr as usize)
            };
    
            match memory.write(&mut caller, offset, &addr_as_bytes) {
                Ok(_) => {}
                _ => println!("failed to write add_ptr to host memory"),
            };
    
            let offset = vm_size_ptr as u32 as usize;
            let addr_as_bytes: [u8; std::mem::size_of::<usize>()] = unsafe {
                std::mem::transmute(len)
            };
            match memory.write(&mut caller, offset, &addr_as_bytes) {
                Ok(_) => {}
                _ => println!("failed to write size_ptr to host memory"),
            };
            0
        }).unwrap();

    }
}

impl ExportFuncs {
    fn new(rt: Runtime, res: Arc<Mutex<HashMap<i32, Vec<u8>>>>) -> Self {
        return  Self {rt, res};
    }

    // fn get_abi_map() -> &HashMap<String, Box<dyn Fn()>> {
    //     let map = [
    //         ("ws_log", Box::new(ExportFuncs::log)), 
    //         ("ws_get_data", Box::new(ExportFuncs::get_data)),
    //     ].iter().cloned().collect();
    //     for (key, value) in &map {
    //         println!("{}: {}", key, value);
    //     }
    // }

    fn log(mut caller: Caller<'_, WasiCtx>, log_level: i32, ptr: i32, size: i32) -> i32 {
        println!("log: logLevel {}", log_level);
        println!("log: ptr {}", ptr);
        println!("log: size {}", size);

        let memory = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => {
                println!("failed to find host memory");
                return 1;
            }
        };

        let offset = ptr as u32 as usize;
        let mut buffer: Vec<u8> = vec![0; size as usize];

        let byte3 = unsafe { 
            String::from_utf8(memory.data(&caller)[offset..][..size as usize]
            .to_vec()).unwrap()
        };
        println!("Result: {}", byte3);
        
        0
    }

    fn get_data(mut caller: Caller<'_, WasiCtx>, rid: i32, vm_add_ptr: i32, vm_size_ptr: i32) -> i32 {
        println!("log: rid {}", rid);
        println!("log: add_ptr {}", vm_add_ptr);
        println!("log: size_ptr {}", vm_size_ptr);
        // let data = self.res.get(rid);
        let data = String::from("The input string");
    
        let len = data.len();
        // let mem_ptr = alloc.call(&mut caller, len as u32).unwrap();

        let alloc_func = match caller.get_export("alloc") {
            Some(Extern::Func(func)) => func,
            _ => {
                println!("expected a function export named 'alloc'");
                return 1;
            },
        };
        let alloc_func_typed = match alloc_func.typed::<i32, i32>(&caller) {
            Ok(func) => func,
            Err(_) => {
                println!("function 'alloc' has a wrong type");
                return 1;
            },
        };

        // let alloc_func_typed: TypedFunc<u32, u32> = alloc_func.typed(caller).unwrap();
        let mem_ptr = alloc_func_typed.call(&mut caller.as_context_mut(), len.try_into().unwrap()).unwrap();
        println!("mem_ptr is {}", mem_ptr);

        // let alloc_func_typed = alloc_func.get_typed_func::<u32, u32>();
        // let result = alloc_func_typed.call(size)?;

        let memory = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => {
                println!("failed to find host memory");
                return 1;
            }
        };

        let pointer = unsafe { memory.data_ptr(&caller).add(mem_ptr as usize) };
        println!("pointer is {:?}", pointer);
        unsafe {
            let bytes = data.as_bytes();
            copy(bytes.as_ptr(), pointer, bytes.len());

            let byte3 = String::from_utf8(memory.data(&caller)[mem_ptr as usize..][..len as usize]
                .to_vec()).unwrap();
            println!("memeory write is : {}", byte3);
        }
        // _ = put_uint32_le( memory.data_mut( &caller), ptr, pointer as u32).unwrap();
        

       
        let offset = vm_add_ptr as u32 as usize;
        println!("add_ptr is {:?}", offset);
        let addr_as_bytes: [u8; std::mem::size_of::<usize>()] = unsafe {
            std::mem::transmute(mem_ptr as usize)
        };
        println!("pointer u8 is {:?}", addr_as_bytes);

        match memory.write(&mut caller, offset, &addr_as_bytes) {
            Ok(_) => {}
            _ => println!("failed to write add_ptr to host memory"),
        };

        let offset = vm_size_ptr as u32 as usize;
        println!("add_ptr is {:?}", offset);
        let addr_as_bytes: [u8; std::mem::size_of::<usize>()] = unsafe {
            std::mem::transmute(len)
        };
        match memory.write(&mut caller, offset, &addr_as_bytes) {
            Ok(_) => {}
            _ => println!("failed to write size_ptr to host memory"),
        };
        0
    }
}

pub struct Instance {
    pub id: uuid::Uuid,
    pub export_funcs: ExportFuncs,
}

pub fn new_instance_by_code(id: uuid::Uuid, code: Vec<u8>) -> Result<Instance> {
    let res = Arc::new(Mutex::new(HashMap::<i32, Vec<u8>>::new()));
    let rt: Runtime = Runtime::default();
    let export_funcs = ExportFuncs::new(rt, res);
    let mut instance = Instance { id, export_funcs };

    instance.export_funcs.link_abi();
    let _ = instance.export_funcs.rt.creat(code);

    return anyhow::Ok(instance)
}