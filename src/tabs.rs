use super::{
    classed,
    HtmlProducer,Block,
};

pub struct Tabs {
    tabs: Vec<Tab>,
}
pub struct Tab {
    pub name: String,
    pub count: usize,
    pub active: bool,
    pub href: String,
}
impl Tabs {
    pub fn new(_producer: &mut HtmlProducer, tabs: Vec<Tab>) -> Tabs {
        Tabs{ tabs: tabs }
    }
    pub fn set_active(&mut self, name: &str) {
        for tab in &mut self.tabs {
            if tab.name == name {
                tab.active = true;
            }
        }
    }
    /*fn push(mut self, tab: Tab) -> Tabs {
        self.tabs.push(tab);
        self
    }*/
    pub fn blocks(&self) -> Block {
        let mut bl = Block::new("tab_row");
        for tab in &self.tabs {
            let act = match tab.active {
                true => "tab_button_active",
                false => "tab_button",
            };        
            bl = bl.sub({
                let mut bl = Block::new(act).text({
                    let mut t = tab.name.clone();
                    match !tab.active && (tab.count > 0) {
                        true => { t += &classed("tab_count",tab.count); },
                        false => { t += &classed("tab_count_empty","&nbsp;"); },
                    }
                    t
                });
                if !tab.active {
                    bl = bl.onclick(format!("tabClicked(\"{}\");",tab.href));
                }
                bl
            });
        }
        bl = bl.sub(Block::new("tab_finish").text("<img width=1 height=1>"));
        Block::new("tabs").sub(bl)
    }
}
