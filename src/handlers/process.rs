use custom_logger as log;
use regex::bytes::Regex;
use std::collections::BTreeMap;

pub trait MetricsInterface {
    fn new() -> Self;
    async fn scrape(&self, node: String) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    fn get_all_metrics(
        &mut self,
        vec_metrics: Vec<String>,
    ) -> Result<MetricsData, Box<dyn std::error::Error>>;
}

pub struct Service {
    pub cpu_state: BTreeMap<isize, f64>,
}

#[derive(Debug, Clone)]
pub struct MetricsData {
    pub cpu: Vec<String>,
    pub memory: Vec<String>,
    pub network: Vec<String>,
    pub disk: Vec<String>,
    pub info: Vec<String>,
}

impl MetricsInterface for Service {
    fn new() -> Self {
        Service {
            cpu_state: BTreeMap::new(),
        }
    }

    async fn scrape(&self, node: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut all_metrics: Vec<String> = Vec::new();
        log::trace!("[scrape] server {}", node);
        let server_endpoint = format!("{}/metrics", node);
        let client = reqwest::Client::new();
        let res = client.get(server_endpoint).send().await;
        match res {
            Ok(data) => {
                let data_result = data.bytes().await?;
                let contents = String::from_utf8(data_result.to_vec())?;
                all_metrics.push(contents);
            }
            Err(e) => {
                println!("{}", e.to_string())
            }
        };
        Ok(all_metrics.clone())
    }

    fn get_all_metrics(
        &mut self,
        vec_metrics: Vec<String>,
    ) -> Result<MetricsData, Box<dyn std::error::Error>> {
        let md = MetricsData {
            cpu: get_cpu_metrics(&mut self.cpu_state, vec_metrics.clone())?,
            memory: get_memory_metrics(vec_metrics.clone())?,
            network: get_network_metrics(vec_metrics.clone())?,
            disk: get_disk_metrics(vec_metrics.clone())?,
            info: get_info_metrics(vec_metrics)?,
        };
        Ok(md)
    }
}

// utility functions

fn get_cpu_metrics(
    cpu_state: &mut BTreeMap<isize, f64>,
    vec_metrics: Vec<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut hm_filtered_metrics: BTreeMap<isize, f64> = Default::default();
    for v in vec_metrics.iter() {
        let lines: Vec<&str> = v.split("\n").collect();
        for line in lines.iter() {
            match line {
                x if x.contains("node_cpu_seconds_total")
                    && !x.contains("idle")
                    && !x.contains("iowait")
                    && !x.contains("steal") =>
                {
                    let re = Regex::new("cpu=\"([0-9]+)\",mode=\"[a-z]+\"}\\W([0-9e\\+\\.]+)")?;
                    for cap in re.captures_iter(line.as_bytes()) {
                        let s_cpu = String::from_utf8(cap[1].to_vec())?;
                        let cpu = s_cpu.parse::<isize>()?;
                        let s_value = String::from_utf8(cap[2].to_vec())?;
                        let value = s_value.parse::<f64>()?;
                        let mut lu_value = hm_filtered_metrics.get(&cpu).unwrap_or(&0.0).to_owned();
                        lu_value = lu_value + value;
                        hm_filtered_metrics.insert(cpu, lu_value);
                    }
                }
                &_ => {}
            };
        }
    }
    let mut vec_filtered_metrics = vec![];
    for (k, v) in hm_filtered_metrics.iter() {
        let current = cpu_state.get(&k).unwrap_or(&0.0);
        match current {
            x if x == &0.0 => vec_filtered_metrics.push(format!("{} 0.000%", k)),
            &_ => {
                let perc = ((v - current).abs() / current) * 100.0;
                vec_filtered_metrics.push(format!("{} {:.3}%", k, perc));
            }
        };
    }
    *cpu_state = hm_filtered_metrics.clone();
    Ok(vec_filtered_metrics)
}

fn get_memory_metrics(vec_metrics: Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut vec_filtered_metrics: Vec<String> = Vec::new();
    let mut total = 0.0;
    let mut avail = 0.0;
    for v in vec_metrics.iter() {
        let lines: Vec<&str> = v.split("\n").collect();
        for line in lines.iter() {
            match line {
                x if x.contains("node_memory_MemTotal_bytes") => {
                    let re = Regex::new("node_memory_MemTotal_bytes\\W([0-9e\\+\\.]+)")?;
                    for cap in re.captures_iter(line.as_bytes()) {
                        let s_total = String::from_utf8(cap[1].to_vec())?;
                        total = s_total.parse::<f64>()?;
                        vec_filtered_metrics.push(format!("total     {:.2}", total));
                    }
                }
                x if x.contains("node_memory_MemAvailable_bytes") => {
                    let re = Regex::new("node_memory_MemAvailable_bytes\\W([0-9e\\+\\.]+)")?;
                    for cap in re.captures_iter(line.as_bytes()) {
                        let s_avail = String::from_utf8(cap[1].to_vec())?;
                        avail = s_avail.parse::<f64>()?;
                        vec_filtered_metrics.push(format!("available {:.2}", avail));
                    }
                }
                &_ => {}
            };
        }
        // final calculations
        vec_filtered_metrics.push(format!(
            "% used    {:.2}",
            ((total - avail) / total) * 100.0
        ));
    }
    Ok(vec_filtered_metrics)
}

fn get_network_metrics(
    vec_metrics: Vec<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut hm_filtered_metrics: BTreeMap<String, f64> = BTreeMap::new();
    for v in vec_metrics.iter() {
        let lines: Vec<&str> = v.split("\n").collect();
        for line in lines.iter() {
            match line {
                x if x.contains("node_network_receive_bytes_total") => {
                    let re = Regex::new(
                        "node_network_receive_bytes_total\\{device=\"([a-zA-Z0-9]+)\"\\}\\W([0-9e\\+\\.]+)",
                    )?;
                    for cap in re.captures_iter(line.as_bytes()) {
                        let s_device = String::from_utf8(cap[1].to_vec())?;
                        let s_value = String::from_utf8(cap[2].to_vec())?;
                        let value = s_value.parse::<f64>()?;
                        hm_filtered_metrics.insert(format!("rx [{:10}]", s_device), value);
                    }
                }
                x if x.contains("node_network_transmit_bytes_total") => {
                    let re = Regex::new(
                        "node_network_transmit_bytes_total\\{device=\"([a-zA-Z0-9]+)\"\\}\\W([0-9e\\+\\.]+)",
                    )?;
                    for cap in re.captures_iter(line.as_bytes()) {
                        let s_device = String::from_utf8(cap[1].to_vec())?;
                        let s_value = String::from_utf8(cap[2].to_vec())?;
                        let value = s_value.parse::<f64>()?;
                        hm_filtered_metrics.insert(format!("tx [{:10}]", s_device), value);
                    }
                }

                &_ => {}
            };
        }
    }
    let vec_filtered_metrics = hm_filtered_metrics
        .iter()
        .map(|(k, v)| format!("{} {:.2}", k, v))
        .collect::<Vec<String>>();
    Ok(vec_filtered_metrics)
}

fn get_disk_metrics(vec_metrics: Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut hm_filtered_metrics: BTreeMap<String, f64> = BTreeMap::new();
    for v in vec_metrics.iter() {
        let lines: Vec<&str> = v.split("\n").collect();
        for line in lines.iter() {
            match line {
                x if x.contains("node_filesystem_size_bytes") => {
                    let re = Regex::new(
                        "node_filesystem_size_bytes\\{device=\"([0-9a-zA-Z/]*)\",device_error=\"[0-9a-zA-Z/]*\",fstype=\"[0-9a-zA-Z/]*\",mountpoint=\"[0-9a-zA-Z/]*\"\\}\\W([0-9e\\+\\.]*)",
                    )?;
                    for cap in re.captures_iter(line.as_bytes()) {
                        let s_device = String::from_utf8(cap[1].to_vec())?;
                        let s_value = String::from_utf8(cap[2].to_vec())?;
                        let value = s_value.parse::<f64>()?;
                        hm_filtered_metrics.insert(format!("total [{:15}]", s_device), value);
                    }
                }
                x if x.contains("node_filesystem_free_bytes") => {
                    let re = Regex::new(
                        "node_filesystem_free_bytes\\{device=\"([0-9a-zA-Z/]*)\",device_error=\"[0-9a-zA-Z/]*\",fstype=\"[0-9a-zA-Z/]*\",mountpoint=\"[0-9a-zA-Z/]*\"\\}\\W([0-9e\\+\\.]*)",
                    )?;
                    for cap in re.captures_iter(line.as_bytes()) {
                        let s_device = String::from_utf8(cap[1].to_vec())?;
                        let s_value = String::from_utf8(cap[2].to_vec())?;
                        let value = s_value.parse::<f64>()?;
                        hm_filtered_metrics.insert(format!("free  [{:15}]", s_device), value);
                    }
                }
                &_ => {}
            };
        }
    }
    let vec_filtered_metrics = hm_filtered_metrics
        .iter()
        .map(|(k, v)| format!("{} {:.2}", k, v))
        .collect::<Vec<String>>();
    Ok(vec_filtered_metrics)
}

fn get_info_metrics(vec_metrics: Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result: Vec<String> = vec![];
    for v in vec_metrics.iter() {
        let lines: Vec<&str> = v.split("\n").collect();
        for line in lines.iter() {
            match line {
                x if x.starts_with("node_uname_info") => {
                    let mut res: Vec<String> = line
                        .replace(",2", "-2")
                        .replace(",1", "-1")
                        .split(",")
                        .map(|x| format!("{}", x))
                        .collect();
                    result.append(&mut res);
                }
                x if x.starts_with("node_dmi_info") => {
                    let mut res: Vec<String> = line
                        .replace(",2", "-2")
                        .replace(",1", "-2")
                        .split(",")
                        .map(|x| format!("{}", x))
                        .collect();
                    result.append(&mut res);
                }

                &_ => {}
            };
        }
    }
    Ok(result)
}
