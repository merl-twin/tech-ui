use std::{
    io, fs,
    sync::mpsc::{channel,Receiver,Sender,RecvTimeoutError,TryRecvError},
    path::PathBuf,
    time::{SystemTime,Duration},
};


pub struct Resource {
    text: String,
    updates: Option<Receiver<String>>,
}
impl Resource {
    /*pub fn get_last(&self) -> &String {
        &self.text
    }*/
    pub fn get(&mut self) -> &String {
        if let Some(rx) = &self.updates {
            let mut drop_rx = false;
            loop {
                match rx.try_recv() {
                    Ok(t) => { self.text = t; },
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => { drop_rx = true; break },
                }
            }
            if drop_rx { self.updates = None; }
        }
        &self.text
    }
}

pub struct ResourceManager {
    sender: Option<Sender<ResourceInner>>,
    handle: Option<std::thread::JoinHandle<()>>,
}
impl Drop for ResourceManager {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(h) = self.handle.take() {
            h.join().ok();
        }
    }
}
impl ResourceManager {
    pub fn new(int: Duration) -> ResourceManager {
        let (tx,rx) = channel();
        ResourceManager {
            sender: Some(tx),
            handle: Some(std::thread::spawn(move || {
                let mut inners = Vec::new();
                loop {
                    match rx.recv_timeout(int) {
                        Ok(inner) => inners.push(inner),
                        Err(RecvTimeoutError::Disconnected) => break,
                        Err(RecvTimeoutError::Timeout) => {}, 
                    }
                    while let Ok(inner) = rx.try_recv() {
                        inners.push(inner);
                    }
                    for inner in &mut inners {
                        if let Err(e) = inner.check_update() {
                            log::warn!("Update failed for {:?}: {:?}",inner.path,e);
                        }
                    }
                }
            })),
        }
    }
    pub fn register(&self, fl: &str, updates: bool) -> Result<Resource,io::Error> {
        Ok(match updates {
            false => resource_no_updates(fl)?,
            true => {
                let (mut rc,inner) = recource_with_updates(fl)?;
                match &self.sender {
                    None => { rc.updates = None; },
                    Some(sender) => if let Err(_) = sender.send(inner) {
                        rc.updates = None;
                    },
                }
                rc
            }
        })
    }
    pub fn empty(&self) -> Resource {
        Resource {
            text: String::new(),
            updates: None,
        }
    }
}


fn recource_with_updates(fl: &str) -> Result<(Resource,ResourceInner),io::Error> {
    let path = PathBuf::from(fl);
    let tm = fs::metadata(&path)?.modified()?;
    let text = fs::read_to_string(&path)?;
    let (tx,rx) = channel();
    Ok((Resource { text, updates: Some(rx) }, ResourceInner { path, tm, sender: Some(tx) }))
}

fn resource_no_updates(fl: &str) -> Result<Resource,io::Error> {
    let path = PathBuf::from(fl);
    let text = fs::read_to_string(&path)?;
    Ok(Resource { text, updates: None })
}

struct ResourceInner {
    path: PathBuf,
    tm: SystemTime,
    sender: Option<Sender<String>>,
}
impl ResourceInner {    
    fn check_update(&mut self) -> Result<bool,io::Error> {
        let tm = fs::metadata(&self.path)?.modified()?;
        let res =  tm > self.tm;
        if res {            
            self.tm = tm;
            let text = fs::read_to_string(&self.path)?;
            match &self.sender {
                None => log::warn!("Resource is non-updatable: {:?}",self.path),
                Some(sender) => match sender.send(text) {
                    Err(_) => {
                        log::warn!("Resource can't be updated: {:?}",self.path);
                        self.sender = None;
                    },
                    Ok(()) => log::info!("Resource updated: {:?}",self.path),
                },
            }
        }
        Ok(res)
    }
}

