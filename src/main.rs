use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use stopwatch::Stopwatch;

#[derive(Copy, Clone)]
struct Location {
    latitude: f64,
    longitude: f64,
}
struct Locations {
    location_list: Vec<Location>,
    index: usize,
}

fn main() {
    let mut DEBUG: bool = false;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "reset" {
            fs::remove_file("attach_address").expect("failed to remove attach address file");
            if args.len() > 2 {
                if args[2] == "debug" {
                    DEBUG = true;
                }
            }
        }
        if args[1] == "shutdown" {
            fs::remove_file("attach_address").expect("failed to remove attach address file");
            panic!("killed existing flutter process!");
        }
        if args[1] == "debug" {
            DEBUG = true;
        }
    }
    let total_stopwatch = Stopwatch::start_new();
    let init_stopwatch = Stopwatch::start_new();
    let app_name = "com.example.james_data".to_string();

    let mut LocationData = Locations {
        location_list: vec![
            Location {
                latitude: 47.5972,
                longitude: -122.6243,
            },
            Location {
                latitude: 47.5967,
                longitude: -122.6247,
            },
            Location {
                latitude: 47.5969,
                longitude: -122.6233,
            },
        ],
        index: 0,
    };

    //if there is a text file called attach address, skip flutter run and other init
    let mut attach_address = String::new();

    match File::open("attach_address") {
        Ok(mut file) => {
            file.read_to_string(&mut attach_address)
                .expect("failed to read attach address from existing file");
            println!("rust: using already running flutter process");
        }
        Err(_) => {
            //something in here is definitiely the cause of the stdin freezing on no error
            //on error it still freezes even when using already running flutter process
            Command::new("flutter")
                .arg("install")
                .output()
                .expect("failed to run flutter install to clear db");

            let appium_installs = Command::new("adb")
                .arg("shell")
                .arg("pm")
                .arg("list")
                .arg("packages")
                .arg("appium")
                .output()
                .expect("failed to search for appium")
                .stdout;

            if appium_installs.len() == 0 {
                Command::new("adb").arg("install").arg("/home/j0/Desktop/kod/flutter/io.appium.settings/app/build/outputs/apk/debug/settings_apk-debug.apk").output().expect("failed to install apium");
                Command::new("adb")
                    .arg("shell")
                    .arg("pm")
                    .arg("grant")
                    .arg("io.appium.settings")
                    .arg("android.permission.ACCESS_FINE_LOCATION")
                    .output()
                    .expect("failed to set appium fine location permission");
                Command::new("adb")
                    .arg("shell")
                    .arg("pm")
                    .arg("grant")
                    .arg("io.appium.settings")
                    .arg("android.permission.ACCESS_COARSE_LOCATION")
                    .output()
                    .expect("failed to set appium coarse location permission");
                Command::new("adb")
                    .arg("shell")
                    .arg("am")
                    .arg("start")
                    .arg("-W")
                    .arg("-n")
                    .arg("io.appium.settings/.Settings")
                    .arg("-a")
                    .arg("android.intent.action.MAIN")
                    .arg("-c")
                    .arg("android.intent.category.LAUNCHER")
                    .arg("-f")
                    .arg("0x10200000")
                    .output()
                    .expect("failed to run appium as launcher");
                Command::new("adb")
                    .arg("shell")
                    .arg("appops")
                    .arg("set")
                    .arg("io.appium.settings")
                    .arg("android:mock_location allow")
                    .output()
                    .expect("failed to set appium as location mocker");
            }
            println!(
                "rust: init took {}s to complete",
                init_stopwatch.elapsed_ms() / 1000
            );

            let (thread_sender, main_receiver) = mpsc::channel();
            //spawn a thread to leave flutter run on indefinitely
            let _thread = thread::spawn(move || {
                let run_stopwatch = Stopwatch::start_new();
                let mut attach_address = "".to_string();
                let mut run = Command::new("flutter")
                    .arg("run")
                    .arg("-t")
                    .arg("test_driver/main.dart")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("failed to run flutter run");

                let out = BufReader::new(
                    run.stdout
                        .take()
                        .expect("failed to create bufreader from run stdout"),
                );
                /*
                let err = BufReader::new(
                    run.stderr
                        .take()
                        .expect("failed to create bufreader from run stderr"),
                );
                */
                for line in out.lines() {
                    let read_line = line.expect("failed to unwrap line in run stdout printer");
                    if read_line.clone().contains("http") {
                        //split the attach address into a var
                        let attach_line: Vec<&str> = read_line.split(": ").collect();
                        attach_address = attach_line[1].trim().to_string();
                        //save to text file for future runs using the same process
                        let mut file = File::create("attach_address")
                            .expect("failed to create attach address file");
                        file.write_all(&format!("{}", attach_address).into_bytes())
                            .expect("failed to write attach address to file");

                        break;
                    }
                }
                /*this needs to run on a different
                //thread spawned before out.lines()
                for line in err.lines() {
                    println!("{}", line.unwrap());
                }
                */
                println!(
                    "rust: flutter run took {}s until it was complete",
                    run_stopwatch.elapsed_ms() / 1000
                );
                thread_sender
                    .send(attach_address)
                    .expect("failed to send attach address from flutter run thread");
                run.wait().expect("failed to await the end of the flutter process, which means the attach address file might have been prematurely deleted");
            });

            attach_address = main_receiver
                .recv()
                .expect("failed to return attach address from flutter run thead");

            if attach_address == "" {
                println!("rust: there was an error in the dart code!\nrust: please wait...");
                let mut run = Command::new("flutter")
                    .arg("run")
                    .arg("-t")
                    .arg("test_driver/main.dart")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("failed to run flutter run");

                let out = BufReader::new(
                    run.stdout
                        .take()
                        .expect("failed to create bufreader from run stdout"),
                );

                let err = BufReader::new(
                    run.stderr
                        .take()
                        .expect("failed to create bufreader from run stderr"),
                );

                let thread = thread::spawn(move || {
                    err.lines().for_each(|line| {
                        if DEBUG {
                            println!(
                                "err: {}",
                                line.expect("failed to unwrap line in error printer")
                            )
                        }
                    });
                });

                out.lines().for_each(|line| {
                    let read_line = line.expect("failed to unwrap line in stdout printer");

                    if DEBUG {
                        println!("out: {}", read_line);
                    }
                });

                thread.join().expect("failed to kill flutter run thread");
                panic!("DART ERROR!");
            }
        }
    };

    Command::new("adb")
        .arg("shell")
        .arg("pm")
        .arg("grant")
        .arg(format!("{}", app_name))
        .arg("android.permission.ACCESS_FINE_LOCATION")
        .output()
        .expect("failed to set app's fine location permission");
    Command::new("adb")
        .arg("shell")
        .arg("pm")
        .arg("grant")
        .arg(format!("{}", app_name))
        .arg("android.permission.ACCESS_COARSE_LOCATION")
        .output()
        .expect("failed to set app's coarse location permission");
    Command::new("adb")
        .arg("shell")
        .arg("pm")
        .arg("grant")
        .arg(format!("{}", app_name))
        .arg("android.permission.ACCESS_MOCK_LOCATION")
        .output()
        .expect("failed to set app's mock location permission");

    let location = LocationData.location_list[0];
    Command::new("adb")
        .arg("shell")
        .arg("am")
        .arg("start-foreground-service")
        .arg("--user")
        .arg("0")
        .arg("-n")
        .arg("io.appium.settings/.LocationService")
        .arg("--es")
        .arg("longitude")
        .arg(format!("{}", location.longitude))
        .arg("--es")
        .arg("latitude")
        .arg(format!("{}", location.latitude))
        .output()
        .expect("failed to set new locaiton");

    let driver_stopwatch = Stopwatch::start_new();
    let mut driver = Command::new("flutter")
        .arg("drive")
        .arg(format!("--use-existing-app={}", attach_address))
        .arg("--target=test_driver/main.dart")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start flutter using existing app");

    let out = BufReader::new(
        driver
            .stdout
            .take()
            .expect("failed to create bufreader from driver stdout"),
    );
    let err = BufReader::new(
        driver
            .stderr
            .take()
            .expect("failed to create bufreader from driver stderr"),
    );

    /*
    let (err_sender, err_reciever): (
        std::sync::mpsc::Sender<String>,
        std::sync::mpsc::Receiver<String>,
    ) = mpsc::channel();
    */

    let thread = thread::spawn(move || {
        for line in err.lines() {
            /*
            let shutdown_message = err_reciever.recv_timeout(Duration::from_secs(2));
            if shutdown_message.is_ok() {
                break;
            }
            */
            let read_line = line.expect("failed to get driver err line unwrapped");
            if DEBUG {
                println!("err: {}", read_line);
            }
        }
    });
    //this var is used to pretty print test failures
    let mut expected_lines = 6;
    for line in out.lines() {
        let read_line = line.expect("failed to unwrap line in stdout printer");

        if read_line.clone().contains("ready for next waypoint") {
            let location = next_location(&mut LocationData, DEBUG);
            Command::new("adb")
                .arg("shell")
                .arg("am")
                .arg("start-foreground-service")
                .arg("--user")
                .arg("0")
                .arg("-n")
                .arg("io.appium.settings/.LocationService")
                .arg("--es")
                .arg("longitude")
                .arg(format!("{}", location.longitude))
                .arg("--es")
                .arg("latitude")
                .arg(format!("{}", location.latitude))
                .output()
                .expect("failed to set new locaiton");
        } else if read_line.clone().contains("Exception") {
            println!("FATAL: {}", read_line);
            panic!("rust: caught a fatal error from dart!");
        } else if read_line.clone().contains("reset waypoints") {
            reset_location(&mut LocationData, DEBUG);
        } else {
            if expected_lines != 6 && expected_lines != 0 {
                println!("err: {}", read_line);
                expected_lines -= 1;
            }
            if DEBUG {
                println!("out: {}", read_line);
            } else if read_line.clone().contains("[E]") {
                println!("dart: {}", read_line);
                expected_lines -= 1;
            } else if read_line.clone().contains("+") {
                println!("dart: {}", read_line);
                expected_lines = 6;
            }
        }
    }
    // err_sender
    //     .send("shutdown".to_string())
    //     .expect("failed to send shutdown to err printer thread");
    driver.wait().expect("failed to wait for driver process");
    thread.join().expect("failed to join err printer thread");
    println!(
        "rust: driver took {}s to complete",
        driver_stopwatch.elapsed_ms() / 1000
    );
    Command::new("adb")
        .arg("shell")
        .arg("am")
        .arg("stopservice")
        .arg("io.appium.settings/.LocationService")
        .output()
        .expect("failed to stop appium location service");
    println!(
        "rust: total time was {} minutes",
        (total_stopwatch.elapsed_ms() as f64) / 1000.0 / 60.0
    );
}

fn next_location(location_data: &mut Locations, DEBUG: bool) -> Location {
    if DEBUG {
        println!("rust: changing location!");
    }
    if location_data.index == location_data.location_list.len() - 1 {
        location_data.index = 0;
    } else {
        location_data.index += 1;
    }
    return location_data.location_list[location_data.index];
}

fn reset_location(location_data: &mut Locations, DEBUG: bool) {
    if DEBUG {
        println!("rust: reset locations!");
    }
    location_data.index = 0;
}
