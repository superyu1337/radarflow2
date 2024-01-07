# radarflow2
A Web radar for CS2 using [memflow](https://github.com/memflow/memflow)

## How can I run this?
There is two ways to run this, the first way is using a KVM/QEMU setup to target a running VM to read memory out of it.
The second way is using pcileech hardware, like a PCIe Screamer.

### The KVM/QEMU method
First, you need to set up a virtual machine on linux using qemu.  
How to set up a VM on linux is way out of scope for this. You can find plenty of information online on how to do it.

Before you begin, install the necessary memflow plugins using memflowup from the *stable 0.2.0 channel!*   
The needed Plugins are `memflow-qemu` and `memflow-win32` 

Clone the repo on your vm host:  
`git clone https://github.com/superyu1337/radarflow2.git`

Run radarflow:   
`cargo run --release`

For an overview of CLI commands, run this:  
`cargo run --release -- --help`

### The pcileech method

Install your pcileech hardware in your target pc. On your attacking PC, install the necessary memflow plugins using memflowup from the *stable 0.2.0 channel!*  
The needed Plugins are `memflow-pcileech` and `memflow-win32`.

Furthermore, you need to install some libraries, depending on your attacking PC's OS.
```
On Windows you additionally need to supply the proprietary FTD3XX.dll.
It can be downloaded from the FTDI Website in the Application Library (DLL) column.

On Linux you need to check-out and compile the leechcore_ft601_driver_linux projectfrom the LeechCore-Plugins repository.
On Linux the leechcore_ft601_driver_linux.so filecurrently has to be placed in /usr/ or /usr/lib.
Alternatively LD_LIBRARY_PATH can be set to the containing path.
Check the dlopen documentation for all possible import paths.
```

Clone the repo on your attacking pc:  
`git clone https://github.com/superyu1337/radarflow2.git`

Run radarflow:   
`cargo run --release -- --connector pcileech`

For an overview of CLI commands, run this:  
`cargo run --release -- --help`

## Detection Status
VAC: ✅ (Undetected)  
FaceIt: ❓ (Unknown, could work with proper spoofing on pcileech method)  
ESEA: ❓ (Unknown, could work with proper spoofing on pcileech method)  
