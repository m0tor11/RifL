use std::fs::{File, OpenOptions};
use std::os::unix::net::UnixListener;
use std::io::{BufRead, BufReader};
use nix::mount::{mount, MsFlags};
use nix::sys::reboot::{reboot, RebootMode};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::io::Write;

fn fmount(qqq : &str){
    let tt = format!("/{}",qqq);
    let _= mount(Some(qqq), tt.as_str(),Some(qqq), MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC, None::<&str>);
}

fn sdown(x : &str){
    match x{
    "1" => { match reboot(RebootMode::RB_POWER_OFF) {
        Ok(_) => tokmsg(&format!("[init]shutdown")),
        Err(e) => tokmsg(&format!("[init]Init Error: {}", e))
    }}
    "2" => { let _ = reboot(RebootMode::RB_AUTOBOOT); tokmsg(&format!("[init]reboot"));}
    _ => tokmsg(&format!("[init]error{}",x))
    }
}


struct Service{
    name: String,
    child: Child,
    restart: bool,
}


fn new_process(x : &str, list : Arc<Mutex<Vec<Service>>>){
    let random_name = x.to_string();
    std::thread::spawn(move || {                                                                                                                 
    let parts: Vec<&str> = random_name.split_whitespace().collect();
    let cmd = parts[0];
    let args = &parts[1..];

    match std::process::Command::new(cmd)
        .args(args)
        .spawn()
        {
            Ok(ch) => {let child = Service {
                name: cmd.to_string(),
                child: ch,
                restart: false,
            };
            list.lock().unwrap().push(child);
            }
            Err(e) => {tokmsg(&format!("failed to spawn {}, {}",cmd,e)); return} 
        };

    });

}

fn tokmsg(msg : &str){
    if let Ok(mut file) = OpenOptions::new().write(true).open("/dev/kmsg"){
        let _ = writeln!(file, "[init] {}", msg);
    }
}

fn main() {
    tokmsg(&format!("[init]init starting, mounting..."));
    fmount("proc");
    let _= mount(Some("sysfs"), "/sys",Some("sysfs"), MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC, None::<&str>); 
    let _= mount(Some("devtmpfs"), "/dev",Some("devtmpfs"), MsFlags::MS_NOSUID , None::<&str>); 
    let _= mount(Some("tmpfs"), "/tmp",Some("tmpfs"), MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC, None::<&str>);
    let _= mount(Some("tmpfs"), "/run",Some("tmpfs"), MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC, None::<&str>); 
    let _= mount(Some("devpts"), "/dev/pts",Some("devpts"), MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC, None::<&str>);
    let _= mount(Some("tmpfs"), "/dev/shm",Some("tmpfs"), MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC, None::<&str>);
    


    let sock = "/run/init.sock";
    tokmsg(&format!("[init]making dirs..."));
    for dir in ["/proc", "/sys", "/dev", "/tmp", "/run", "/dev/pts", "/dev/shm"]{
        let _ =    std::fs::create_dir_all(dir);
    }
    tokmsg(&format!("[init]creating /run/init.sock ..."));
    let _ = std::fs::remove_file(sock);
    let listener = UnixListener::bind(sock).expect("bind");

    tokmsg(&format!("[init]staring /bin/sh"));
    std::thread::spawn(|| {
        let _ = std::process::Command::new("/bin/sh")
          .arg("-i")
          .status();
                        });




    let autorun = "/etc/init/autorun";



    tokmsg(&format!("starting comands from {}", autorun));
   


        let childs = Arc::new(Mutex::new(Vec::<Service>::new()));
    
    #[allow(clippy::never_loop)] 
    for _ in 1..5{
        let file = match File::open(autorun){
            Ok(f) => f,
            Err(e) => {tokmsg(&format!("[init]error in file: {}",e)); break},
        };

        
        let read = BufReader::new(file);

        for line in read.lines(){
            match line {
            Ok(l) => new_process(l.trim(), Arc::clone(&childs)),
            Err(e) => {tokmsg(&format!("[init]error in file: {}",e));}
        };
    }
        break;
}
    std::thread::spawn(move ||{
    for stream in listener.incoming(){
        match stream {
            Ok(stream) => {
                tokmsg("[init]stream ok");
                let mut reader = BufReader::new(stream);
                let mut l = String::new();

                let _ = reader.read_line(&mut l);
                let cmd = l.trim();
                sdown(cmd);
                
            },
            
            Err(e) => tokmsg(&format!("[init]stream error: {}", e)),
        }
    }
    });

let mut i = -1;
loop{
    i += 1;
    let mut childs_list = childs.lock().unwrap();
    
    childs_list.retain_mut(|ch| {
        match ch.child.try_wait(){
        Ok(Some(stat)) => {
            tokmsg(&format!("child exited: {}", stat));
            




            false
        }

        Ok(None) => true,
        Err(e) => {
            tokmsg(&format!("child exited with error: {}", e));





            false
                }
            }
        });


    std::thread::sleep(std::time::Duration::from_secs(1));
        
    }

}
