use std::process::Command;
use std::net::UdpSocket;
use std::str;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};
use std::thread;

const BIND_ADDR: &str = "0.0.0.0:1234";
const CACHE_TTL: Duration = Duration::from_secs(300); //30*60);
const SLEEP_TIME: Duration = Duration::from_secs(1);
const REMOVE_BATCH_SIZE: i32 = 100;

fn init_nft() -> std::io::Result<()> {
    let _output = Command::new("nft")
	.arg("add").arg("table").arg("bridge").arg("filter")
	.output()
	.expect("Failed to create nft table.");

    let _output = Command::new("nft")
	.arg("add").arg("chain").arg("bridge").arg("filter").arg("forward")
	.arg("{type filter hook forward priority 0; }")
	.output()
	.expect("Failed to create nft chain.");
	
    Ok(())
}

fn list_nft() -> std::io::Result<HashMap<String,SystemTime>> {
    let mut map = HashMap::new();

    let output = Command::new("nft")
	.arg("list").arg("chain").arg("bridge").arg("filter").arg("forward")
	.output()?;

    if output.status.success() {
	for line in String::from_utf8_lossy(&output.stdout).lines() {
	    let parts: Vec<&str> = line.split_whitespace().collect();
	    if parts.len() == 4 && parts[0] == "ip" {
		let now = SystemTime::now();
		map.insert(String::from(parts[2]), now);
		// println!("Added: {}", parts[2]);
	    }
	}
    } else {
	println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(map)
}

fn list_handles_nft() -> std::io::Result<HashMap<String,String>> {
    let mut map = HashMap::new();

    let output = Command::new("nft")
	.arg("--handle").arg("--numeric")
	.arg("list").arg("chain").arg("bridge").arg("filter").arg("forward")
	.output()?;

    if output.status.success() {
	for line in String::from_utf8_lossy(&output.stdout).lines() {
	    let parts: Vec<&str> = line.split_whitespace().collect();
	    if parts.len() == 7 && parts[0] == "ip" {
		map.insert(String::from(parts[2]), String::from(parts[6]));
		// println!("IP: {}, Handle: {}", parts[2], parts[6]);
	    }
	}
    } else {
	println!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(map)
}

fn add_nft(ip: &str) -> std::io::Result<()> {
    let _output = Command::new("nft")
	.arg("add").arg("rule").arg("bridge").arg("filter").arg("forward")
	.arg("ip").arg("saddr").arg(ip).arg("drop")
	.output()
	.expect("Failed to create nft rule.");

    Ok(())
}

fn remove_nft(handle: &str) -> std::io::Result<()> {
    if handle != "" {
	let _output = Command::new("nft")
	    .arg("delete").arg("rule").arg("bridge").arg("filter").arg("forward")
	    .arg("handle").arg(handle)
	    .output()
	    .expect("Failed to remove nft rule.");
    }
    
    Ok(())
}

fn listen(blocks: Arc<Mutex<HashMap<String, SystemTime>>>) -> std::io::Result<()> {
    let socket = UdpSocket::bind(BIND_ADDR)?;
    println!("Listening on {}", BIND_ADDR);
    loop {
	let mut buf = [0; 1024];
	let (amt, _src) = socket.recv_from(&mut buf)?;
	let ip = str::from_utf8(&mut buf[..amt]).unwrap();
	
	let now = SystemTime::now();

	// Lock
	let mut block_list = blocks.lock().unwrap();
	
	match block_list.get(ip) {
	    Some(_) => 	println!("Renewing block on {}", ip),
	    None => println!("Blocking {}", ip),
	}

	add_nft(ip)?;

	block_list.insert(String::from(ip), now);

	// Unlock
	drop(block_list);
    }
}

fn prune(blocks: Arc<Mutex<HashMap<String, SystemTime>>>) -> std::io::Result<()> {
    // Clean up thread
    loop {
	// Prune list
	let now = SystemTime::now();
	// Lock
	let mut block_list = blocks.lock().unwrap();
	let mut remove_list: HashSet<String> = HashSet::new();
	for (ip,time) in block_list.iter() {
	    match now.duration_since(*time) {
		Ok(duration) => {
		    if duration > CACHE_TTL {
			remove_list.insert(ip.clone());
		    }
		},
		Err(_) => {},
	    }
	}
	if remove_list.len() > 0 {
	    let handles: HashMap<String,String> = list_handles_nft().unwrap();
	    let mut removed = 0;
	    println!("Handles: {}", handles.len());
	    for ip in remove_list.iter() {
		if removed <= REMOVE_BATCH_SIZE {
		    let time = block_list.remove(ip).unwrap_or(now);
		    let duration = now.duration_since(time).unwrap_or_default();
		    let mins = duration.as_secs() / 60;
		    match handles.get(ip) {
			None => println!("Handle not found for {}", ip),
			Some(h) => {
			    println!("Removing block on {} after {:.0} minutes (Handle: {})", ip, mins, h);
			    remove_nft(h).unwrap();
			}
		    }
		    removed = removed + 1;
		    if removed > REMOVE_BATCH_SIZE {
			println!("Removed {} blocks.", removed);
		    }
		}
	    }
	}
	// Unlock
	drop(block_list);
	thread::sleep(SLEEP_TIME);
    } // end loop
}

fn main() -> std::io::Result<()> {
    init_nft()?;

    let blocks_mutex = Arc::new(Mutex::new(list_nft()?));
    {
	println!("Loaded {} blocked IPs", blocks_mutex.lock().unwrap().len());
    }

    
    let prune_blocks = Arc::clone(&blocks_mutex);
    let prune_thread = thread::spawn(move || {
	prune(prune_blocks)
    });
    
    let listen_blocks = Arc::clone(&blocks_mutex);
    let listen_thread = thread::spawn(move || {
	listen(listen_blocks)
    });

    listen_thread.join().expect("Error in listening thread")?;
    prune_thread.join().expect("Error in pruning thread")?;
    
    Ok(())
    
}

