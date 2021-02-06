use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::process::Command;
use std::str::FromStr;

use actix_web::{App, get, HttpResponse, HttpServer, Responder};
use chrono::{NaiveDateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

const DHCP_LEASES_FILE_PATH: &str = "/var/db/dhcpd/dhcpd.leases";

#[get("/api/status")]
async fn index() -> impl Responder {
    // distinct by mac address and exclude unavailable lease
    let mut leases = BTreeMap::<String, DhcpLease>::new();
    for lease in read_dhcp_leases().await {
        if !lease.is_available() {
            continue;
        }
        leases.insert(lease.mac_address.to_string(), lease);
    }

    let mut response = BTreeMap::<String, MergedEntry>::new();
    // Insert leases
    for lease in leases.values() {
        response.insert(lease.mac_address.to_string(), MergedEntry {
            dhcp_lease: Some(lease.clone()),
            ndp_entries: Vec::new(),
        });
    }
    // Inset ndp entries
    for entry in retrieve_ndp_entries().await {
        response
            .entry(entry.mac_address.to_string())
            .or_insert_with(|| MergedEntry {
                ndp_entries: Vec::new(),
                dhcp_lease: None,
            })
            .ndp_entries.push(entry.clone());
    }
    HttpResponse::Ok().json(response)
}

#[derive(Debug, Serialize, Deserialize)]
struct MergedEntry {
    ndp_entries: Vec<NdpEntry>,
    dhcp_lease: Option<DhcpLease>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting");
    HttpServer::new(|| App::new().service(index))
        .bind("192.168.0.1:80")?
        .run()
        .await
}

async fn read_dhcp_leases() -> Vec<DhcpLease> {
    let leases_file = File::open(DHCP_LEASES_FILE_PATH).unwrap();
    let mut leases: Vec<DhcpLease> = Vec::new();
    let mut current_buf = String::new();
    let mut in_section = false;
    for line in BufReader::new(leases_file).lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.starts_with("lease") {
            in_section = true;
        }
        if in_section {
            current_buf.push_str(line);
        }
        if line.starts_with("}") {
            leases.push(DhcpLease::from_str(&current_buf).unwrap());
            in_section = false;
            current_buf.clear()
        }
    }
    leases
}

async fn retrieve_ndp_entries() -> Vec<NdpEntry> {
    let stdout = Command::new("ndp")
        .arg("-a")
        .output()
        .unwrap()
        .stdout;
    let stdout = String::from_utf8_lossy(&stdout);
    stdout
        .split("\n")
        .skip(1)
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| NdpEntry::from_str(line).unwrap())
        .collect()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DhcpLease {
    mac_address: String,
    ipv4_address: Ipv4Addr,
    end: String,
    host_name: Option<String>,
}

impl DhcpLease {
    fn is_available(&self) -> bool {
        let end = NaiveDateTime::parse_from_str(&self.end, "%Y/%m/%d %H:%M:%S").unwrap();
        end.timestamp() > Utc::now().timestamp()
    }
}

impl FromStr for DhcpLease {
    type Err = &'static str;

    /// Must be trimmed
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let address_regex = Regex::new(r"lease (.*?) \{").unwrap();
        let end_regex = Regex::new(r"ends . (.*?);").unwrap();
        let mac_address_regex = Regex::new(r"hardware ethernet (.*?);").unwrap();
        let host_regex = Regex::new("client-hostname \"(.*?)\";").unwrap();

        let ipv4_address =
            Ipv4Addr::from_str(
                &address_regex.captures_iter(&value).next().unwrap()[1]
            ).unwrap();
        let end = end_regex.captures_iter(&value).next().unwrap()[1].to_string();
        let mac_address = mac_address_regex
            .captures_iter(&value).next().unwrap()[1].to_string();
        let host_name = host_regex
            .captures_iter(&value).next().map(|cap| cap[1].to_string());
        Ok(Self {
            mac_address,
            host_name,
            ipv4_address,
            end,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NdpEntry {
    mac_address: String,
    ipv6_address: String,
}

impl FromStr for NdpEntry {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"([^ ]+)").unwrap();
        let mut matches = regex.captures_iter(s);
        let ipv6_address = matches.next().unwrap()[1].to_string();
        let mac_address = matches.next().unwrap()[1].to_string();
        Ok(NdpEntry {
            ipv6_address,
            mac_address,
        })
    }
}