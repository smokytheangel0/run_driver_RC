# run_driver_RC
flutter driver set up and run script for the Retrieval Collector or other location aware flutter apps

# setup
you will need the rust toolchain to build this script \
https://www.rust-lang.org/tools/install

after building a binary using cargo build, copy the resulting binary from \
target/release/run_driver to the flutter project dir you wish to use it on \
it can then be run from there.

# dependencies
it is important that you have https://github.com/appium/io.appium.settings downloaded and built, and edit line 93 \
or whichever line contains  ```Command::new("adb").arg("install")``` to include the path to the io.appium.settings apk \
which is usually at app/build/outputs/apk/debug/settings_apk-debug.apk relative to the root of the appium settings folder

it is also important that your flutter app has the flutter_driver dependency and that you have a test_driver folder\
with a main.dart (where the app is inited with enableFlutterDriverExtension), and main_test.dart (where your driver test code should reside)  

if this setup is not for you or you wish to use this with an already configured app, you can edit the line 144 (or the code 
``` 
let mut run = Command::new("flutter")
                    .arg("run")
                    .arg("-t")
                    .arg("test_driver/main.dart")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("failed to run flutter run");
```
)
to reflect the file used to init your app for driver to use

and line 292 (or the code
```
let mut driver = Command::new("flutter")
        .arg("drive")
        .arg(format!("--use-existing-app={}", attach_address))
        .arg("--target=test_driver/main.dart")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start flutter using existing app");
```
)
to reflect the main file you use to start your driver tests        

# understanding
the program has three arguments it can be run with
* debug
* reset
* shutdown

the way the program keeps the flutter process alive even after the end is reached \
is a bit of a hack and results in terminal I/O lockup after it the script is complete \
this will be fixed in the future, but it is not a dealbreaker at this time 

the debug argument allows flutter to print every message to stdout through this script \
this includes a step by step explanation of the drivers actions and attempts to find things 

the reset argument deletes the attach_address file used to attach to the same flutter process \
the script then reruns init and the flutter process from scratch 

the shutdown argument deletes the attach_address file and exits the process 

each of these arguments besides debug can be replicated by simply deleting attach_address 