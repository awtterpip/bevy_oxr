## setup
install xbuild ```cargo install --git https://github.com/rust-mobile/xbuild```
run ```x doctor``` and install all required depencencies
download the [openxr loader](https://developer.oculus.com/downloads/package/oculus-openxr-mobile-sdk/) and put it in the runtime_libs/arm64-v8a folder

## how to run
run ```x devices```

and get the device name that looks something like this ```adb:1WD523S``` (probably a bit longer)
 
then ```run x run --release```
