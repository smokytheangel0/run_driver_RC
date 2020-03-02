# run_driver_RC
flutter driver set up and run script for the Retrieval Collector or other location aware flutter app

# setup
you will need the rust toolchain to build this script \
https://www.rust-lang.org/tools/install

after building a binary using cargo build, copy the resulting binary from \
target/release/run_driver to the flutter project dir you wish to use it on \
it can then be run from there.

# dependencies
it is important that you have https://github.com/appium/io.appium.settings downloaded and built, and edit line 93  
or whichever line contains  ```Command::new("adb").arg("install")``` to include the path to the io.appium.settings apk  

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
