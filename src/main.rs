use std::path::{Path, PathBuf};
use tokio::time::{timeout, Duration};
use tokio::process::Command;
use windows_sys::Win32::System::Console::GetConsoleWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
use std::io;
use sysinfo ::{System};

fn is_ac_running() -> bool {
    let mut system = System::new_all();
    system.refresh_all();
    for (_pid, process) in system.processes() {
        if process.name().to_ascii_lowercase() == "acgui.exe" {
            return true;
        }
    }
    false
}

fn hide_console() {
  unsafe {
    let hwnd = GetConsoleWindow();
    if !hwnd.is_null() {
      ShowWindow(hwnd, SW_HIDE);
    }
  }
}

fn get_ifilter_bin() -> Result<(std::path::PathBuf, std::path::PathBuf), &'static str> {
    let program_files = Path::new("C:\\Program Files\\Digital Arts");
    let program_files_x86 = Path::new("C:\\Program Files (x86)\\Digital Arts");
    
    let digital_arts_path = (if program_files.exists() {
        Ok(program_files)
    } else if program_files_x86.exists() {
        Ok(program_files_x86)
    } else {
        Err("Digital Arts folder not found in Program Files or Program Files (x86)")
    })?;

    let acentry_exe = digital_arts_path.join("AC\\app\\fcbin\\acentry.exe");
    let accui_exe = digital_arts_path.join("AC\\app\\bin\\accui.exe");

    return Ok((acentry_exe, accui_exe));
}

// 低頻度でやるやつ
async fn kill_with_reregister(acentry: PathBuf, accui: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let mut stop_process = Command::new(acentry)
        .arg("-ReRegist")
        .spawn()?; // reregister を起動

    // 5秒待機
    let _ = timeout(Duration::from_secs(5), stop_process.wait()).await?;

    let _ = Command::new(accui)
        .arg("/acstop")
        .status()
        .await?; // 停止させる

    let _ = stop_process.kill().await?; // まだ生きてる場合殺す

    Ok(())
}

// 高頻度でやるやつ
async fn kill_pure(accui: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let _ = Command::new(accui)
        .arg("/acstop")
        .status()
        .await?; // 停止させる
    Ok(())
}

async fn run() -> Result<(), &'static str> {
    println!("Checking for DigitalArts@Cloud installation...");
    let (acentry, accui) = get_ifilter_bin()?;
    println!("✅ Successfully found DigitalArts@Cloud installation:");
    println!(" - ACEntry: {}", acentry.display());
    println!(" - ACCUI:   {}", accui.display());

    println!("\n");

    println!("⌛Starting bypass engine...");

    let _ =  kill_with_reregister(acentry.clone(), accui.clone()).await;

    println!("🔪🩸 Censorship was defeated");

    let accui_for_kill_pure = accui.clone();
    let kill_pure_task = tokio::spawn(async move {
        loop {
            if is_ac_running() {
                if let Err(e) = kill_pure(accui_for_kill_pure.clone()).await {
                    eprintln!("Error in kill_pure: {}", e);
                } 
            } else {
                println!("ACGUI is not running. Skipping pure kill.");
            }
            tokio::time::sleep(Duration::from_secs(10)).await; // 10秒ごとに実行
        }
    });
    let accui_for_kill_with_reregister = accui.clone();
    let kill_with_reregister_task = tokio::spawn(async move  {
        loop {
            if is_ac_running() {
                if let Err(e) = kill_with_reregister(acentry.clone(), accui_for_kill_with_reregister.clone()).await {
                    eprintln!("Error in kill_with_reregister: {}", e);
                }
            } else {
                println!("ACGUI is not running. Skipping reregister and kill.");
            }
            tokio::time::sleep(Duration::from_secs(60 * 10)).await;
        }
    });

    let hide_console_task = tokio::spawn(async move {
        println!("Hiding console window after 3 seconds...");
        hide_console();
        println!("Console window hidden.");
    });

    let _ = tokio::join!(kill_pure_task, kill_with_reregister_task, hide_console_task);

    Ok(())
}

#[tokio::main]
async fn main() {
    let result = run().await;
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        eprintln!("\nPress Enter to exit...");
        let _ = io::stdin().read_line(&mut String::new());
        std::process::exit(1);
    }
}