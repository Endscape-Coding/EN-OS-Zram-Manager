use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::process;
use std::{thread, time};
use sysinfo::{Components, System};
use is_root::is_root;
use colored::*;
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
#[allow(non_camel_case_types)]
enum Algoritm {
    lz4,
    zstd1,
    zstd2,
    zstd3,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, value_enum)]
    alg: Option<Algoritm>,

    #[arg(short, long)]
    gb: Option<f64>,

    #[arg(long)]
    install: bool,

    #[arg(long)]
    uninstall: bool,
}

const LOGO: &str = r#" _____ _   _        ___  ____    _________                   __  __
| ____| \ | |      / _ \/ ___|  |__  /  _ \ __ _ _ __ ___   |  \/  | __ _ _ __   __ _  __ _  ___ _ __
|  _| |  \| |_____| | | \___ \    / /| |_) / _` | '_ ` _ \  | |\/| |/ _` | '_ \ / _` |/ _` |/ _ \ '__|
| |___| |\  |_____| |_| |___) |  / /_|  _ < (_| | | | | | | | |  | | (_| | | | | (_| | (_| |  __/ |
|_____|_| \_|      \___/|____/  /____|_| \_\__,_|_| |_| |_| |_|  |_|\__,_|_| |_|\__,_|\__, |\___|_|
                                                                                      |___/

"#;

// Дополнительные функции

fn run_cmd(cmd: &str, args: &[&str]) -> io::Result<()> {
    let cmd_status = Command::new(cmd).args(args).status()?;

    match cmd_status.success() {
        true => Ok(()),
        false => Err(std::io::Error::other(
            format!("Ошибка в выполнении команды {}", cmd),
        )),
    }
}

fn check_memory(sys: &mut System) -> f64 {
    sys.refresh_memory();

    sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0
}

fn check_cpu() -> f64 {
    let path = "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq";
    let path = Path::new(&path);

    match path.exists() {
        true => {
            let khz = fs::read_to_string(path)
            .unwrap()
            .trim()
            .parse::<f64>()
            .unwrap_or(0.0);


            khz / 1000000.0
        }
        false => 2.0,
    }
}

fn check_install() -> bool {
    let path = "/etc/systemd/system/zram.service";
    let path = Path::new(&path);
    match path.exists() {
        true => match fs::metadata(path) {
            Ok(data) => !matches!(data.len(), 0),
            Err(e) => {
                println!("Error {e}");
                false
            }
        },
        false => false,
    }
}

fn count() -> (String, f64) {
    let mut sys = System::new();
    let _components = Components::new_with_refreshed_list();
    sys.refresh_all();

    let memory_size = check_memory(&mut sys);
    let cpu = check_cpu();
    let mut gb: f64;
    let alg: String;
    if memory_size <= 2.0 {
        if cpu <= 2.0 {
            gb = memory_size;
            alg = "lz4".to_string();
        } else {
            gb = memory_size;
            alg = "zstd1".to_string();
        }
    } else if memory_size <= 4.0 {
        if cpu <= 2.0 {
            gb = memory_size / 1.5;
            alg = "lz4".to_string();
        } else {
            gb = memory_size / 1.5;
            alg = "zstd2".to_string();
        }
    } else if memory_size <= 8.0 {
        if cpu <= 2.0 {
            gb = memory_size / 2.0;
            alg = "lz4".to_string();
        } else {
            gb = memory_size / 2.0;
            alg = "zstd2".to_string();
        }
    } else if memory_size <= 16.0 {
        if cpu <= 2.0 {
            gb = memory_size / 2.0;
            alg = "lz4".to_string();
        } else {
            gb = memory_size / 2.0;
            alg = "zstd3".to_string();
        }
    } else if cpu <= 2.0 {
        gb = memory_size / 3.0;
        alg = "lz4".to_string();
        gb = gb.min(16.0);
    } else {
        gb = memory_size / 2.5;
        alg = "zstd3".to_string();
        gb = gb.min(16.0);
    }

    (alg, gb)
}

// Основные фукции

fn zram_install(alg: &str, gb: f64) -> io::Result<()> {
    let service_path = "/etc/systemd/system/zram.service";
    let service_path = Path::new(&service_path);
    let config = format!(
        r#"
[Unit]
Description=ZRAM Configuration
DefaultDependencies=no
Before=swap.target
After=local-fs.target
Conflicts=shutdown.target
ConditionVirtualization=!container

[Service]
Type=oneshot
ExecStart=/usr/bin/zram-manager --gb {:.1} --alg {}
RemainAfterExit=yes
TimeoutSec=30
StandardOutput=journal
Restart=no
User=root

[Install]
WantedBy=multi-user.target
"#,
        gb, alg
    );

    fs::write(service_path, config)?;

    run_cmd("systemctl", &["daemon-reload"])?;
    run_cmd("systemctl", &["enable", "zram.service"])?;
    run_cmd("systemctl", &["start", "zram.service"])?;

    println!("{}", "Zram has been installed!".green());
    Ok(())
}

fn zram_on(alg: &str, gb: f64) -> io::Result<()> {
    println!("Zram-start");
    let _bytes = (gb * 1024.0 * 1024.0 * 1024.0) as u64;

    run_cmd("modprobe", &["zram", "num_devices=1"])?;

    let _ = Command::new("swapoff").arg("/dev/zram0").status();

    fs::write("/sys/block/zram0/reset", "1")?;

    let zram_path_alg = "/sys/block/zram0/comp_algorithm";
    let zram_path_str = "/sys/block/zram0/max_comp_streams";
    let zram_path_dsk = "/sys/block/zram0/disksize";

    let mut sys = System::new();
    let _components = Components::new_with_refreshed_list();
    sys.refresh_all();
    let cores = sys.cpus().len().max(1);

    let algoritm = match alg {
        "lz4" => "lz4",
        "zstd1" => "zstd level=1",
        "zstd2" => "zstd level=2",
        "zstd3" => "zstd level=3",
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Неверный алгоритм! Доступные варианты смотрите в zram-manager -h"
            ));
        }
    };

    fs::write(zram_path_alg, algoritm)?;
    fs::write(zram_path_str, cores.to_string())?;
    fs::write(zram_path_dsk, _bytes.to_string())?;

    run_cmd("mkswap", &["/dev/zram0"])?;
    run_cmd("swapon", &["/dev/zram0", "-p", "100"])?;

    Ok(())
}

fn zram_uninstall() -> io::Result<()> {
    println!("{}", "Uninstalling..".yellow());

    run_cmd("systemctl", &["daemon-reload"])?;
    run_cmd("systemctl", &["stop", "zram.service"])?;
    run_cmd("systemctl", &["disable", "zram.service"])?;

    let path = "/etc/systemd/system/zram.service";
    let path = Path::new(&path);
    match path.exists() {
        true => {
            fs::remove_file(path)?;
            Ok(())
        }
        false => {
            Ok(())
        }
    }

}

fn zram_info() {
    let wait = time::Duration::from_secs(1);
    let mut sys = System::new();
    let _components = Components::new_with_refreshed_list();
    let _ = Command::new("clear").status();
    loop {
        let install = check_install();
        print!("\x1B[?25l");
        print!("\x1B[H");
        println!("{}", LOGO.magenta());

        println!("{}", "ZRAM INFO".blue());

        if install {
            println!("{}", "ZRam service installed\n".green());
            let _ = Command::new("zramctl").status();
        } else {
            println!("{}", "ZRam service is not installed\n".red());
        }

        sys.refresh_all();
        let usage_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let cpu_usage = sys.global_cpu_usage() as i32;

        println!("\n{} {:.1}%", "CPU Usage: ".red(), cpu_usage);
        println!("{}{:.1} GB", "Memory Usage: ".red(), usage_memory);

        println!(
            "{}",
            "\n\nFor install use: sudo zram-manager --install".yellow()
        );
        println!("\n{}", "Ctrl+c to exit".cyan());
        io::stdout().flush().unwrap();

        thread::sleep(wait)
    }
}


fn main() {
    let _ = Command::new("clear").status();
    println!("{}", LOGO.magenta());
    println!("{}", "By Endscape".blue());
    let install_check = check_install();
    let args = Args::parse();

    if args.install {
        if is_root() {
         println!("{}", "Processing installation...".red());
        } else {
            println!("{}", "Run program with sudo!".red());
            process::exit(0);
        }

        let (alg_final, gb_final) = match (&args.alg, args.gb) {
            (Some(a), Some(g)) => (format!("{:?}", a), g),
            _ => count(),
        };

        if install_check {
            println!(
                "{}",
                "ZRam service already installed, reinstalling...".yellow()
            );
        } else {
            println!("{}", "Installing ZRam Service...".yellow());
        }

        let mut sys = System::new();
        sys.refresh_all();
        let memory_size = check_memory(&mut sys);

        println!(
            "Memory size: {memory_size:.2} GB, CPU freq: {} GHz",
            check_cpu()
        );

        let _ = zram_install(&alg_final, gb_final);
    } else if args.uninstall {
        let _ = zram_uninstall();
        println!("{}", "Service uninstalled successfully!".green());
    } else if let (Some(alg), Some(gb)) = (&args.alg, args.gb) {
        let algstr = format!("{:?}", alg);
        let _ = zram_on(&algstr, gb);
    } else {
        zram_info();
    }
}
