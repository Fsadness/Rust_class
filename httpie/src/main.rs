//! main.rs
use anyhow::{anyhow, Result}; // 错误处理
use clap::Parser; // 命令行解析
use colored::*; // 命令终端多彩展示
use mime::Mime; // 处理mime类型
use reqwest::{header, Client, Response, Url}; // HTTP客户端
use std::{collections::HashMap, str::FromStr}; // HashMap and FromStr

/// A naive httpie implementation with Rust
#[derive(Parser, Debug)]
#[command(version = "1.0", author = "Fsadness <f.sadness@qq.com>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// Sub-commands: get / post
#[derive(Parser, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post), // Others
}

/// Get: feed get with an url and will retrieve the response
#[derive(Parser, Debug)]
struct Get {
    /// HTTP requested url
    #[arg(value_parser = parse_url)]
    url: String,
}

/// Post: feed post with an url and optional key = value pairs,
/// will retrieve the response after posting the data as JSON
#[derive(Parser, Debug)]
struct Post {
    /// HTTP requested url
    #[arg(value_parser = parse_url)]
    url: String,
    /// HTTP requested body
    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}

/// KvPair: turn key=value in argpharses into KvPair struct
#[derive(Clone, Debug)]
struct KvPair {
    k: String,
    v: String,
}

/// FromStr trait: Turn String into KvPair, used in str.parse()
impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

/// Check whether the url is leagal
fn parse_url(s: &str) -> Result<String> {
    // 使用reqwest::Url验证url字段是否可用
    let _url: Url = s.parse()?;
    Ok(s.into())
}

/// Check whether the KvPair is leagal
fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

/// get
async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

/// post
async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }

    let resp = client.post(&args.url).json(&body).send().await?;
    // println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

/// turn content-type into Mime
fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

/// print response status
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status)
}

/// print HTTP header
fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{} {:?}", name.to_string().green(), value);
    }

    print!("\n")
}

/// print HTTP body
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        // pretty print "application/json"
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        // directly output for others
        _ => println!("{}", body),
    }
}

/// print whole response
async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

/// main with asynchronous processing library using tokio
#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    // Add the HTTP header
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);
    // Generate an HTTP client
    let client = Client::new();
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };

    Ok(result)
}
