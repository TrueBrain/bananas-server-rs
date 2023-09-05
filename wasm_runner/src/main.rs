use std::error::Error;
use std::sync::Arc;
use futures::Future;
use futures::lock::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use wasmtime::{Engine, Module, Store, Linker, Caller, Config, Extern};

struct ProcessEnv {
    reader: Arc<Mutex<OwnedReadHalf>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
}

fn console_log(mut caller: Caller<'_, ProcessEnv>, ptr: i32, length: i32) -> Box<dyn Future<Output = ()> + Send + '_> {
    Box::new(async move {
        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => panic!("failed to find WASM memory"),
        };

        let data = mem.data(&caller)[ptr as usize ..ptr as usize + length as usize].as_ref();
        let string = match std::str::from_utf8(data) {
            Ok(string) => string,
            Err(_) => "(invalid string)",
        };

        /* Log it to the console. */
        println!("worker: {}", string);

        ()
    })
}

fn read(mut caller: Caller<'_, ProcessEnv>, ptr: i32, length: i32) -> Box<dyn Future<Output = i32> + Send + '_> {
    Box::new(async move {
        let reader = caller.data().reader.clone();
        let mut reader = reader.lock().await;

        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        let mut data = mem.data_mut(&mut caller)[ptr as usize ..ptr as usize + length as usize].as_mut();

        match reader.read_exact(&mut data).await {
            Ok(n) => n as i32,
            Err(_) => -1i32,
        }
    })
}

fn write(mut caller: Caller<'_, ProcessEnv>, ptr: i32, length: i32) -> Box<dyn Future<Output = i32> + Send + '_> {
    Box::new(async move {
        let writer = caller.data().writer.clone();
        let mut writer = writer.lock().await;

        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        let data = mem.data(&caller)[ptr as usize ..ptr as usize + length as usize].as_ref();

        match writer.write(&data).await {
            Ok(n) => n as i32,
            Err(_) => -1i32,
        }
    })
}

async fn process(socket: TcpStream, engine: &Engine, module: &Module, linker: &Linker<ProcessEnv>) -> Result<(), Box<dyn Error>> {
    let (reader, writer) = socket.into_split();

    let mut store = Store::new(&engine, ProcessEnv {
        reader: Arc::new(Mutex::new(reader)),
        writer: Arc::new(Mutex::new(writer)),
    });
    let instance = linker.instantiate_async(&mut store, &module).await?;

    let connect = instance.get_typed_func::<(), ()>(&mut store, "connect")?;
    connect.call_async(&mut store, ()).await?;

    Ok(())
}

async fn listen() -> Result<(), Box<dyn Error>> {
    let wasm_bytes = std::fs::read("../test2.wasm")?;

    let engine = Engine::new(Config::new().async_support(true))?;
    let module = Module::new(&engine, wasm_bytes)?;

    let mut linker = Linker::new(&engine);
    linker.func_wrap2_async("env", "console_log", console_log)?;
    linker.func_wrap2_async("env", "read", read)?;
    linker.func_wrap2_async("env", "write", write)?;

    /* Listen for incoming TCP connections. */
    let listener = TcpListener::bind("127.0.0.1:12345").await?;

    loop {
        /* Blocking wait for a new connection. */
        let (socket, _) = listener.accept().await?;

        let module = module.clone();
        let linker = linker.clone();
        let engine = engine.clone();

        /* Spawn a new task to process the new connection. */
        tokio::spawn(async move {
            match process(socket, &engine, &module, &linker).await {
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
