use reqwest;
use serde_json;
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver},
};
use threadpool::ThreadPool;
use utils::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct SubInfo {
    pub surl: String,
    pub language: String,
    pub rate: String,
    pub svote: i64,
}

impl SubInfo {
    pub fn all(cid_hash: &str, limit: usize) -> Result<Vec<SubInfo>> {
        let url = format!(
            "http://sub.xmp.sandai.net:8000/subxl/{cid_hash}.json",
            cid_hash = cid_hash
        );
        let text = reqwest::get(&url)?.text()?;
        let json = serde_json::from_str::<serde_json::Value>(&text)?;
        let mut sub_info_list = json
            .get("sublist")
            .expect("Wrong response")
            .as_array()
            .expect("Wrong response")
            .into_iter()
            .filter(|x| !x.as_object().expect("Wrong response").is_empty())
            .map(|x| serde_json::from_value::<SubInfo>(x.clone()).expect("Wrong response"))
            .collect::<Vec<_>>();

        sub_info_list.sort_by_key(|x| x.svote);
        sub_info_list.reverse();

        sub_info_list = sub_info_list
            .into_iter()
            .take(limit)
            .collect::<Vec<SubInfo>>();

        Ok(sub_info_list)
    }

    pub fn download(&self) -> Result<String> {
        Ok(reqwest::get(&self.surl)?.text()?)
    }
}

#[derive(Debug)]
pub struct DownloadResult {
    pub response: Result<String>,
    pub target_path: PathBuf,
}

#[derive(Default)]
pub struct TaskRunner {
    pub pool: ThreadPool,
    pub results: Vec<Receiver<DownloadResult>>,
}

impl TaskRunner {
    pub fn execute(&mut self, sub_info: SubInfo, target_path: PathBuf) {
        let (sender, receiver) = channel::<DownloadResult>();
        self.pool.execute(move || {
            sender
                .send(DownloadResult {
                    response: sub_info.download(),
                    target_path,
                }).expect("Send download result failed")
        });
        self.results.push(receiver)
    }
}
