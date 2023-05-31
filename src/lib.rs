pub mod resources;

mod tabs;
mod table;

pub use tabs::{Tab,Tabs};

pub use table::{TableBuilder,TableDrawer,RowRef,TableError,SoftColumn};

#[derive(Debug)]
pub struct Style {
    name: &'static str,
    opts: Vec<(&'static str, &'static str)>,
}
impl Style {
    pub fn new(style: &'static str) -> Style {
        Style {
            name: style,
            opts: Vec::new(),
        }
    }
    pub fn duplicate(&self,style: &'static str) -> Style {
        Style {
            name: style,
            opts: self.opts.clone(),
        }
    }
    pub fn opt(mut self, opt: &'static str, value: &'static str) -> Style {
        let mut idx = None;
        for (i,(o,_)) in self.opts.iter().enumerate() {
            if *o == opt { idx = Some(i); break }
        }
        match idx {
            Some(i) => self.opts[i] = (opt,value),
            None => self.opts.push((opt,value)),
        }
        self
    }
}
impl ToString for Style {
    fn to_string(&self) -> String {
        match self.opts.len() {
            0 => String::new(),
            _ => {
                let mut s = format!(".{} {{",self.name);
                for (o,v) in &self.opts {
                    s += "    ";
                    s += o;
                    s += ": ";
                    s += v;
                    s += ";\n"
                }
                s += "}";
                s
            },
        }
    }
}

pub fn classed<T: ToString>(class: &'static str, t: T) -> String {
    format!("<span class='{}'>{}</span>",class,t.to_string())
}

#[derive(Debug)]
pub struct Block {
    class: &'static str,
    onclick: Option<String>,
    id: Option<String>,
    text: Option<String>,
    subs: Vec<Block>,
}
impl Block {
    pub fn new(class: &'static str) -> Block {
        Block {
            class,
            onclick: None,
            id: None,
            text: None,
            subs: Vec::new(),
        }
    }
    pub fn id<T: ToString>(mut self, id: T) -> Block {
        self.id = Some(id.to_string());
        self
    }
    pub fn onclick<T: ToString>(mut self, onclick: T) -> Block {
        self.onclick = Some(onclick.to_string());
        self
    }
    pub fn text<T: ToString>(mut self, t: T) -> Block {
        self.text = Some(t.to_string());
        self
    }
    pub fn sub(mut self, s: Block) -> Block {
        self.subs.push(s);
        self
    }
    pub fn sub_mut(&mut self, s: Block) {
        self.subs.push(s);
    }
}
impl ToString for Block {
    fn to_string(&self) -> String {
        let t = match &self.text {
            Some(t) => t,
            None => "",
        };
        let s = match self.subs.len() {
            0 => String::new(),
            _ => {
                let mut s = String::new();
                for b in &self.subs {
                    s += &b.to_string();
                    s += "\n"
                }
                s
            },
        };
        match (&self.id, &self.onclick) {
            (None,None) => format!("<div class='{}'>{}{}</div>",self.class,t,s),
            (None, Some(oc)) => format!("<div class='{}' onclick='{}'>{}{}</div>",self.class,oc,t,s),
            (Some(id),None) => format!("<div id='{}' class='{}'>{}{}</div>",id,self.class,t,s),
            (Some(id), Some(oc)) => format!("<div id='{}' class='{}' onclick='{}'>{}{}</div>",id,self.class,oc,t,s),
        }
    }
}


#[derive(Debug,Default)]
pub struct HtmlProducer {
    title: String,
    scripts: Vec<String>,
    styles: Vec<Style>,
    blocks: Vec<Block>,

    tables: TableDrawer,
    
    css: String,
    js: String,
}
impl ToString for HtmlProducer {
    fn to_string(&self) -> String {
        let mut style = self.css.clone();
        style += "\n";
        for s in &self.styles {
            style += &s.to_string();
            style += "\n";
        }
        let mut body = String::new();
        for b in &self.blocks {
            body += &b.to_string();
            body += "\n";
        }
        let mut script = self.js.clone();
        script += "\n";
        for scr in &self.scripts {
            script += scr;
            script += "\n";
        }
        format!("<html>\n<head>\n<title>\n{}\n</title>\n<style>\n{}\n</style>\n<script>\n{}\n</script>\n</head>\n<body>\n{}\n</body>\n</html>\n",self.title,style,script,body)
    }
}
impl HtmlProducer {
    pub fn with_title<T: ToString>(mut self, t: T) -> HtmlProducer {
        self.title = t.to_string();
        self
    }
    pub fn with_styles(mut self, css: &String) -> HtmlProducer {
        if self.css != "" { self.css += "\n"; }
        self.css += css;        
        self
    }
    pub fn with_scripts(mut self, js: &String) -> HtmlProducer {
        if self.js != "" { self.js += "\n"; }
        self.js += js;        
        self
    }
    
    pub fn push_script<T: ToString>(&mut self, t: T) {
        self.scripts.push(t.to_string());
    }
    pub fn push_style(&mut self, s: Style) {
        self.styles.push(s);
    }
    pub fn push_block(&mut self, b: Block) {
        self.blocks.push(b);
    }
    
    pub fn drawer(&mut self) -> &mut TableDrawer {
        &mut self.tables
    }
    pub fn add_tables(&mut self, tb: &TableBuilder) {        
        let s = tb.styles(&self.tables);
        if s != "" {
            if self.css != "" { self.css += "\n"; }
            self.css += &s;
        }
    }
}
