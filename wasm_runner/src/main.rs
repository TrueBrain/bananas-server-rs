use std::error::Error;
use std::sync::Arc;
use futures::lock::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use wasmer::{Store, Module, Instance, imports, TypedFunction, Function, FunctionEnv, FunctionEnvMut, Memory, WasmPtr};

struct ProcessEnv {
    memory: Option<Memory>,
    reader: Arc<Mutex<OwnedReadHalf>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
}

fn read(mut ctx: FunctionEnvMut<ProcessEnv>, ptr: WasmPtr<u8>, length: i32) -> i32 {
    let (env, store) = ctx.data_and_store_mut();

    let memory_view = env.memory.as_ref().unwrap().view(&store);
    let reader = env.reader.clone();

    /* As we spawned on a blocking task, we can safely block. We use this to switch from sync to async context. */
    tokio::runtime::Handle::current().block_on(async move {
        let mut reader = reader.lock().await;

        /* Read the amount of bytes as requested; block till we have them. */
        let mut buf = vec![0; length as usize];
        let len = match reader.read_exact(&mut buf).await {
            Ok(n) => n,
            Err(_) => return -1,
        };

        /* Write the bytes back into the WASM buffer. */
        if len != 0 {
            match memory_view.write(ptr.offset() as u64, &buf[0..len]) {
                Ok(_) => {},
                Err(_) => return -1,
            }
        }

        len as i32
    })
}

fn write(mut ctx: FunctionEnvMut<ProcessEnv>, ptr: WasmPtr<u8>, length: i32) -> i32 {
    let (env, store) = ctx.data_and_store_mut();

    let memory_view = env.memory.as_ref().unwrap().view(&store);
    let writer = env.writer.clone();

    /* As we spawned on a blocking task, we can safely block. We use this to switch from sync to async context. */
    tokio::runtime::Handle::current().block_on(async move {
        let mut writer = writer.lock().await;

        let mut buf = vec![0; length as usize];
        match memory_view.read(ptr.offset() as u64, &mut buf) {
            Ok(_) => {},
            Err(_) => return -1,
        };

        match writer.write(&buf).await {
            Ok(n) => n as i32,
            Err(_) => -1,
        }
    })
}

fn console_log(mut ctx: FunctionEnvMut<ProcessEnv>, ptr: WasmPtr<u8>, length: i32) {
    let (env, store) = ctx.data_and_store_mut();
    let memory_view = env.memory.as_ref().unwrap().view(&store);

    /* Read the string from WASM memory. */
    let string = match ptr.read_utf8_string(&memory_view, length as u32) {
        Ok(s) => s,
        Err(_) => "(invalid string)".to_string(),
    };

    /* Log it to the console. */
    println!("worker: {}", string);
}

async fn process(socket: TcpStream, module: &Module) -> Result<(), Box<dyn Error>> {
    let (reader, writer) = socket.into_split();

    let mut store = Store::default();
    let ctx = FunctionEnv::new(&mut store, ProcessEnv {
        memory: None,
        reader: Arc::new(Mutex::new(reader)),
        writer: Arc::new(Mutex::new(writer)),
    });

    /* Create a new instance of the processor. */
    let import_object = imports! {
        "env" => {
            "console_log" => Function::new_typed_with_env(&mut store, &ctx, console_log),
            "read" => Function::new_typed_with_env(&mut store, &ctx, read),
            "write" => Function::new_typed_with_env(&mut store, &ctx, write),
        },
    };
    let instance = Instance::new(&mut store, &module, &import_object)?;

    /* Get the memory from the WASM, so we can read/write buffers. */
    {
        let mut ctx_mut = ctx.into_mut(&mut store);
        let (env, _) = ctx_mut.data_and_store_mut();
        env.memory = Some(instance.exports.get_memory("memory")?.clone());
    }

    let store = Arc::new(store);
    let instance = Arc::new(instance);

    /* Create a new (blocking) task which calls the connect() functions in WASM.
     * Ideally we wouldn't need this, but wasmer currently doesn't allow async
     * host functions. So to still have async code in our host functions, we
     * use "block_on", which requires things to be on a thread that can block. */
    tokio::task::spawn_blocking(move || -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut store = Arc::try_unwrap(store).unwrap();
        let connect: TypedFunction<(), ()> = instance.exports.get_typed_function(&mut store, "connect")?;

        connect.call(&mut store)?;

        Ok(())
    }).await?.map_err(|e| e as Box<dyn Error>)?;

    /* If WASM terminates, the connection handling comes to an end. */

    Ok(())
}

async fn listen() -> Result<(), Box<dyn Error>> {
    /* Load in the WASM application. */
    let wasm_bytes = std::fs::read("../test2.wasm")?;
    let store = Store::default();
    let module = Arc::new(Module::new(&store, wasm_bytes)?);

    /* Listen for incoming TCP connections. */
    let listener = TcpListener::bind("127.0.0.1:12345").await?;

    loop {
        /* Blocking wait for a new connection. */
        let (socket, _) = listener.accept().await?;

        /* Spawn a new task to process the new connection. */
        let module = module.clone();
        tokio::spawn(async move {
            match process(socket, &module).await {
                Ok(_) => {},
                Err(e) => println!("Failed to process connection: {}", e),
            }
        });
    }
}

#[tokio::main()]
async fn main() {
    listen().await.unwrap();
}
