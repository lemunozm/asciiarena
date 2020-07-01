
mod client;
mod server;

fn main()
{
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1
    {
        let sub_args: Vec<String> = args[1..].iter().cloned().collect();
        match sub_args[0].as_ref()
        {
            "client" => { client::run(sub_args); return; },
            "server" => { server::run(sub_args); return; },
            mode => println!("'{}' is not a valid mode", mode),
        }
    }
    println!("Usage: AsciiArena [client | server]")
}
