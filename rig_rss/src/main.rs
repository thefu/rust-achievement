use rig::providers::openai::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use reqwest;
use rss::Channel;
use tokio::time::{self, Duration};
use std::error::Error;
use regex::Regex;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct SummarizedRssItem {
    title: String,
    link: String,
    #[schemars(with = "String")]
    pub_date: DateTime<Utc>,
    summary: String,
    relevance_score: f32,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
struct RssSummary {
    items: Vec<SummarizedRssItem>,
    total_count: usize,
    extraction_time: String, // ISO 8601 formatted string
    overall_summary: String,
}

// 定义一个函数，用于美化打印RSS摘要信息
fn pretty_print_summary(summary: &RssSummary) {
    // 打印RSS摘要的标题
    println!("RSS Feed Summary:");
    // 打印总项目数
    println!("Total Items: {}", summary.total_count);
    // 打印提取时间
    println!("Extraction Time: {}", summary.extraction_time);
    // 打印顶级项目列表的标题
    println!("\nTop Items:");
    // 遍历摘要中的项目列表
    for (i, item) in summary.items.iter().enumerate() {
        // 打印项目编号和标题
        println!("{}. {}", i + 1, item.title);
        // 打印项目的链接
        println!("   Link: {}", item.link);
        // 打印项目的发布日期
        println!("   Published: {}", item.pub_date);
        // 打印项目的摘要
        println!("   Summary: {}", item.summary);
        // 打印项目的相关性得分，保留两位小数
        println!("   Relevance Score: {:.2}", item.relevance_score);
        // 打印空行以分隔不同项目
        println!();
    }
    // 打印整体摘要信息
    println!("Overall Summary: {}", summary.overall_summary);
}

// 异步函数，用于从给定的URL获取RSS订阅源
async fn fetch_rss_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    // 使用reqwest库发送HTTP GET请求到指定的URL
    // await关键字用于等待异步操作的完成
    // ?操作符用于传播错误，如果请求失败，将返回错误
    let response = reqwest::get(url).await?.text().await?;
    // 尝试将响应文本解析为Channel类型
    // parse方法用于将字符串解析为特定的数据结构
    // ?操作符用于传播错误，如果解析失败，将返回错误
    let channel = response.parse::<Channel>()?;
    // 如果一切顺利，返回解析后的Channel对象
    Ok(channel)
}

// 定义一个名为 sanitize_string 的函数，接受一个字符串切片作为输入，返回一个字符串
fn sanitize_string(input: &str) -> String {
    // 将输入字符串转换为可变的字符串类型
    let mut sanitized = input.to_string();
    // 将字符串中的换行符 "\n" 替换为空格 " "
    sanitized = sanitized.replace("\n", " ");
    // 将字符串中的回车符 "\r" 替换为空字符串（即删除回车符）
    sanitized = sanitized.replace("\r", "");
    // 将字符串中的双引号 '"' 替换为空字符串（即删除双引号）
    sanitized = sanitized.replace("\"", "");
    sanitized = sanitized.replace("’", "'"); // Replace any special quotes
    sanitized
}

// 异步函数，用于从RSS频道中提取摘要
async fn summarize_rss_feed(channel: Channel) -> Result<RssSummary, Box<dyn Error>> {
    // 创建一个OpenAI客户端
    let openai_client = Client::from_env();

    // 创建一个提取器，指定模型和前导文本
    let extractor = openai_client
        .extractor::<RssSummary>("gpt-4o-mini-2024-07-18")
        .preamble("You are an AI assistant specialized in summarizing RSS feeds. \
                   Your task is to analyze the RSS items, extract the most relevant information, \
                   and provide concise summaries. For each item, provide a brief summary and a \
                   relevance score from 0.0 to 1.0. Also, provide an overall summary of the feed.")
        .build();

    // 创建一个包含所有摘要的向量
    let rss_items = channel.items();
    let mut formatted_rss = String::new();

    // 创建一个包含所有摘要的向量
    let re_html = Regex::new(r"(?i)<[^>]*>").unwrap();
    let re_cdata = Regex::new(r"(?i)<!\[CDATA\[.*?\]\]>").unwrap();

    for (i, item) in rss_items.iter().enumerate() {
        let title = item.title().unwrap_or("").to_string();
        let link = item.link().unwrap_or("").to_string();
        let pub_date = item.pub_date().unwrap_or("").to_string();
        let description = item.description().unwrap_or("").to_string();

        // 提取摘要
        let clean_description = re_html.replace_all(&re_cdata.replace_all(&description, ""), "").to_string();
        let sanitized_description = sanitize_string(&clean_description);

        formatted_rss.push_str(&format!(
            "{}. Title: {}\nLink: {}\nDate: {}\nDescription: {}\n\n",
            i + 1,
            sanitize_string(&title),
            sanitize_string(&link),
            sanitize_string(&pub_date),
            sanitized_description
        ));
    }

    println!("Extracting summary from the RSS feed...\n");

    let rss_summary = extractor.extract(&formatted_rss).await?;

    Ok(rss_summary)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rss_url = "https://news.ycombinator.com/rss";
    let mut interval = time::interval(Duration::from_secs(3600)); // 1 hour interval

    loop {
        interval.tick().await;
        
        match fetch_rss_feed(rss_url).await {
            Ok(channel) => {
                match summarize_rss_feed(channel).await {
                    Ok(rss_summary) => {
                        pretty_print_summary(&rss_summary);
                    }
                    Err(e) => eprintln!("Error summarizing RSS feed: {}", e),
                }
            }
            Err(e) => eprintln!("Error fetching RSS feed: {}", e),
        }
    }
}